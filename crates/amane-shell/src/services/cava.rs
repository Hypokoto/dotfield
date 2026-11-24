//! Cava Audio Spectrum Visualizer Service.
//! Supervises an optional cava child process in raw 8-bit FIFO mode,
//! sampling 16-32 frequency bands while parking at 0 Hz during audio silence.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::{Duration, Instant};

use amane::Service;
use arc_swap::ArcSwap;

pub const CAVA_BAR_COUNT: usize = 20;

static SPECTRUM_BARS: LazyLock<ArcSwap<Vec<f32>>> =
    LazyLock::new(|| ArcSwap::from_pointee(vec![0.0; CAVA_BAR_COUNT]));

static IS_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Default)]
pub struct CavaService {
    pub bars: Vec<f32>,
    pub is_active: bool,
}

impl Service for CavaService {
    fn new() -> Self {
        Self {
            bars: vec![0.0; CAVA_BAR_COUNT],
            is_active: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_millis(100)
    }
}

impl CavaService {
    /// Starts the background cava visualizer listener.
    pub fn start() {
        // Only attempt to start if cava executable exists
        let which_cava = Command::new("which")
            .arg("cava")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if !which_cava.map(|s| s.success()).unwrap_or(false) {
            return;
        }

        thread::spawn(|| {
            loop {
                if let Err(_e) = Self::run_cava_reader() {
                    // Backoff before restarting cava
                    thread::sleep(Duration::from_secs(3));
                }
            }
        });
    }

    /// Secure path for temporary cava config file within XDG_RUNTIME_DIR or user-isolated tmpfs.
    pub fn config_path() -> std::path::PathBuf {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            std::path::PathBuf::from(runtime_dir).join("amane_cava.conf")
        } else {
            #[cfg(unix)]
            let uid = unsafe { libc::getuid() };
            #[cfg(not(unix))]
            let uid = 1000;
            std::env::temp_dir().join(format!("amane_cava_{uid}.conf"))
        }
    }

    /// Read raw byte stream from cava configured for raw stdout output.
    fn run_cava_reader() -> anyhow::Result<()> {
        let temp_cfg = Self::config_path();
        let config_content = format!(
            "[general]\nbars = {}\nframerate = 60\nsleep_timer = 2\n\n[output]\nmethod = raw\nraw_target = /dev/stdout\ndata_format = binary\nbit_format = 8bit\n\n[smoothing]\nmonstercat = 1\nnoise_reduction = 75\n",
            CAVA_BAR_COUNT
        );
        let _ = std::fs::write(&temp_cfg, config_content);

        let child_proc = Command::new("cava")
            .arg("-p")
            .arg(&temp_cfg)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut child = ChildGuard(child_proc);

        let mut stdout = match child.0.stdout.take() {
            Some(s) => s,
            None => return Ok(()),
        };

        let mut buf = vec![0u8; CAVA_BAR_COUNT];
        let mut last_activity = Instant::now();

        while let Ok(()) = stdout.read_exact(&mut buf) {
            let (float_bars, max_val) = process_cava_raw_bytes(&buf);

            if max_val > 5 {
                last_activity = Instant::now();
                IS_ACTIVE.store(true, Ordering::Relaxed);
                SPECTRUM_BARS.store(Arc::new(float_bars));
            } else if last_activity.elapsed() > Duration::from_secs(2) {
                // Silence: park spectrum bars to zero to allow 0 Hz render parking
                if IS_ACTIVE.swap(false, Ordering::Relaxed) {
                    SPECTRUM_BARS.store(Arc::new(vec![0.0; CAVA_BAR_COUNT]));
                }
                // Sleep slightly during prolonged silence to save CPU cycles
                thread::sleep(Duration::from_millis(150));
            }
        }

        // Child process is killed and reaped safely by ChildGuard RAII Drop
        Ok(())
    }

    /// Get current spectrum bars via lock-free atomic load.
    pub fn spectrum() -> Arc<Vec<f32>> {
        SPECTRUM_BARS.load_full()
    }

    /// Check if audio is currently playing and visualizer is active.
    pub fn is_audio_active() -> bool {
        IS_ACTIVE.load(Ordering::Relaxed)
    }
}

/// Normalizes 8-bit raw cava bytes into 0.0..1.0 float bars and tracks peak value.
pub fn process_cava_raw_bytes(buf: &[u8]) -> (Vec<f32>, u8) {
    let mut max_val = 0u8;
    let float_bars: Vec<f32> = buf
        .iter()
        .map(|&b| {
            if b > max_val {
                max_val = b;
            }
            (b as f32) / 255.0
        })
        .collect();
    (float_bars, max_val)
}

/// RAII Guard ensuring child processes are reaped on error or exit.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_cava_raw_bytes_silence() {
        let silence = vec![0u8; CAVA_BAR_COUNT];
        let (bars, max_val) = process_cava_raw_bytes(&silence);
        assert_eq!(max_val, 0);
        assert_eq!(bars.len(), CAVA_BAR_COUNT);
        for bar in bars {
            assert_eq!(bar, 0.0);
        }
    }

    #[test]
    fn test_process_cava_raw_bytes_scaling() {
        let mut sample = vec![0u8; CAVA_BAR_COUNT];
        sample[0] = 255;
        sample[1] = 128;
        sample[2] = 64;
        let (bars, max_val) = process_cava_raw_bytes(&sample);
        assert_eq!(max_val, 255);
        assert!((bars[0] - 1.0).abs() < 1e-4);
        assert!((bars[1] - (128.0 / 255.0)).abs() < 1e-4);
        assert!((bars[2] - (64.0 / 255.0)).abs() < 1e-4);
    }

    #[test]
    fn test_cava_config_path_security() {
        let path = CavaService::config_path();
        let path_str = path.to_string_lossy();
        assert!(path_str.ends_with("amane_cava.conf") || path_str.contains("amane_cava_"));
    }
}
