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

    /// Read raw byte stream from cava configured for raw stdout output.
    fn run_cava_reader() -> anyhow::Result<()> {
        let temp_cfg = "/tmp/amane_cava_config";
        let config_content = format!(
            "[general]\nbars = {}\n[output]\nmethod = raw\nraw_target = /dev/stdout\ndata_format = binary\nbit_format = 8bit\n",
            CAVA_BAR_COUNT
        );
        let _ = std::fs::write(temp_cfg, config_content);

        let mut child: Child = Command::new("cava")
            .arg("-p")
            .arg(temp_cfg)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let mut stdout = match child.stdout.take() {
            Some(s) => s,
            None => return Ok(()),
        };

        let mut buf = vec![0u8; CAVA_BAR_COUNT];
        let mut last_activity = Instant::now();

        while let Ok(()) = stdout.read_exact(&mut buf) {
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

        let _ = child.kill();
        let _ = child.wait();
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
