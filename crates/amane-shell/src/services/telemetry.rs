use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::FileExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

static RUNNING: AtomicBool = AtomicBool::new(false);
static TELEMETRY_STATE: LazyLock<ArcSwap<TelemetrySnapshot>> =
    LazyLock::new(|| ArcSwap::from_pointee(TelemetrySnapshot::new()));

/// Immutable snapshot of system telemetry and active anomalies.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetrySnapshot {
    pub cpu_percent: u8,
    pub mem_percent: u8,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub battery_percent: Option<u8>,
    pub battery_charging: bool,
    pub battery_present: bool,
    pub cpu_stalled: bool,
    pub mem_stalled: bool,
    pub culprit_pid: Option<i32>,
    pub culprit_name: Option<String>,
    pub culprit_delta_percent: f32,
    pub anomaly_active: bool,
    pub timestamp: Instant,
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetrySnapshot {
    pub fn new() -> Self {
        Self {
            cpu_percent: 0,
            mem_percent: 0,
            mem_used_mb: 0,
            mem_total_mb: 0,
            battery_percent: None,
            battery_charging: false,
            battery_present: false,
            cpu_stalled: false,
            mem_stalled: false,
            culprit_pid: None,
            culprit_name: None,
            culprit_delta_percent: 0.0,
            anomaly_active: false,
            timestamp: Instant::now(),
        }
    }

    /// Returns true if any anomalous resource spike or stall threshold has tripped.
    pub fn is_anomalous(&self) -> bool {
        self.anomaly_active
            || self.cpu_percent >= 85
            || self.mem_percent >= 80
            || self.cpu_stalled
            || self.mem_stalled
            || (self.battery_present
                && !self.battery_charging
                && self.battery_percent.unwrap_or(100) <= 20)
    }

    /// Returns true if critical threshold has tripped (red danger color).
    pub fn is_critical(&self) -> bool {
        self.cpu_percent >= 95
            || self.mem_percent >= 90
            || (self.battery_present
                && !self.battery_charging
                && self.battery_percent.unwrap_or(100) <= 10)
    }
}

pub struct TelemetryService;

impl TelemetryService {
    /// Read latest immutable telemetry snapshot lock-free.
    pub fn read() -> Arc<TelemetrySnapshot> {
        TELEMETRY_STATE.load_full()
    }

    /// Spawns background telemetry poller and Linux PSI sentinel.
    pub fn start() {
        if RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        thread::Builder::new()
            .name("amane-telemetry".to_string())
            .spawn(telemetry_worker_loop)
            .expect("failed to spawn amane-telemetry thread");
    }

    #[cfg(test)]
    pub fn set_for_test(snapshot: TelemetrySnapshot) {
        TELEMETRY_STATE.store(Arc::new(snapshot));
    }
}

/// Persistent cached file descriptors for zero-allocation stack pread operations.
struct ProcfsReader {
    stat_file: Option<File>,
    meminfo_file: Option<File>,
    psi_cpu_file: Option<File>,
    psi_mem_file: Option<File>,
    bat_cap_file: Option<File>,
    bat_status_file: Option<File>,
    battery_present: bool,
    prev_busy_ticks: u64,
    prev_total_ticks: u64,
    attribution_cache: Option<(Instant, Option<i32>, Option<String>, f32)>,
    anomaly_active: bool,
    tripped_at: Option<Instant>,
    recovery_start: Option<Instant>,
}

impl ProcfsReader {
    fn new() -> Self {
        let stat_file = File::open("/proc/stat").ok();
        let meminfo_file = File::open("/proc/meminfo").ok();
        let psi_cpu_file =
            init_psi_trigger(Path::new("/proc/pressure/cpu"), b"some 200000 1000000");
        let psi_mem_file =
            init_psi_trigger(Path::new("/proc/pressure/memory"), b"some 100000 1000000");

        let (bat_cap_file, bat_status_file, battery_present) = find_battery_files();

        Self {
            stat_file,
            meminfo_file,
            psi_cpu_file,
            psi_mem_file,
            bat_cap_file,
            bat_status_file,
            battery_present,
            prev_busy_ticks: 0,
            prev_total_ticks: 0,
            attribution_cache: None,
            anomaly_active: false,
            tripped_at: None,
            recovery_start: None,
        }
    }

    /// Zero-allocation parse of `/proc/stat` for aggregate CPU percentage.
    fn sample_cpu(&mut self) -> u8 {
        let Some(file) = &self.stat_file else {
            return 0;
        };

        let mut buf = [0u8; 2048];
        let Ok(bytes_read) = file.read_at(&mut buf, 0) else {
            return 0;
        };
        if bytes_read == 0 {
            return 0;
        }

        let slice = &buf[..bytes_read];
        let Some((busy, total)) = parse_proc_stat_cpu_line(slice) else {
            return 0;
        };

        if self.prev_total_ticks == 0 {
            self.prev_busy_ticks = busy;
            self.prev_total_ticks = total;
            return 0;
        }

        let busy_delta = busy.saturating_sub(self.prev_busy_ticks);
        let total_delta = total.saturating_sub(self.prev_total_ticks);
        self.prev_busy_ticks = busy;
        self.prev_total_ticks = total;

        (busy_delta * 100)
            .checked_div(total_delta)
            .map(|v| v.clamp(0, 100) as u8)
            .unwrap_or(0)
    }

    /// Zero-allocation parse of `/proc/meminfo` for memory percentage and MB usage.
    fn sample_memory(&self) -> (u8, u64, u64) {
        let Some(file) = &self.meminfo_file else {
            return (0, 0, 0);
        };

        let mut buf = [0u8; 4096];
        let Ok(bytes_read) = file.read_at(&mut buf, 0) else {
            return (0, 0, 0);
        };
        if bytes_read == 0 {
            return (0, 0, 0);
        }

        let slice = &buf[..bytes_read];
        let total_kb = parse_meminfo_field(slice, b"MemTotal:");
        let avail_kb = parse_meminfo_field(slice, b"MemAvailable:");

        if total_kb == 0 {
            return (0, 0, 0);
        }

        let used_kb = total_kb.saturating_sub(avail_kb);
        let percent = ((used_kb * 100) / total_kb).clamp(0, 100) as u8;
        let used_mb = used_kb / 1024;
        let total_mb = total_kb / 1024;

        (percent, used_mb, total_mb)
    }

    /// Zero-allocation read of battery capacity and charging status.
    fn sample_battery(&self) -> (Option<u8>, bool) {
        if !self.battery_present {
            return (None, false);
        }

        let cap = self.bat_cap_file.as_ref().and_then(|file| {
            let mut buf = [0u8; 16];
            let n = file.read_at(&mut buf, 0).ok()?;
            if n > 0 {
                parse_u64_from_ascii(&buf[..n]).map(|v| v.clamp(0, 100) as u8)
            } else {
                None
            }
        });

        let charging = self
            .bat_status_file
            .as_ref()
            .map(|file| {
                let mut buf = [0u8; 32];
                if let Ok(n) = file.read_at(&mut buf, 0)
                    && n > 0
                {
                    let status_bytes = &buf[..n];
                    status_bytes.starts_with(b"Charging")
                } else {
                    false
                }
            })
            .unwrap_or(false);

        (cap, charging)
    }

    /// Attribution engine: identifies highest CPU delta or RSS consumer.
    /// Uses a non-blocking 250ms sampling window and caches results for 2.0s.
    fn attribute_culprit(&mut self, is_mem_pressure: bool) -> (Option<i32>, Option<String>, f32) {
        let now = Instant::now();
        if let Some((cached_at, pid, ref name, delta)) = self.attribution_cache
            && now.duration_since(cached_at) < Duration::from_millis(2000)
        {
            return (pid, name.clone(), delta);
        }

        if is_mem_pressure {
            let (pid, name, rss_mb) = find_top_memory_culprit();
            self.attribution_cache = Some((now, pid, name.clone(), rss_mb));
            return (pid, name, rss_mb);
        }

        let (pid, name, delta_cpu) = find_top_cpu_culprit_two_point();
        self.attribution_cache = Some((now, pid, name.clone(), delta_cpu));
        (pid, name, delta_cpu)
    }
}

fn init_psi_trigger(path: &Path, trigger: &[u8]) -> Option<File> {
    use std::fs::OpenOptions;
    let mut file = OpenOptions::new().read(true).write(true).open(path).ok()?;
    file.write_all(trigger).ok()?;
    Some(file)
}

fn find_battery_files() -> (Option<File>, Option<File>, bool) {
    let base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("BAT") {
                let cap_path = entry.path().join("capacity");
                let status_path = entry.path().join("status");

                let cap_file = File::open(&cap_path).ok();
                let status_file = File::open(&status_path).ok();

                if cap_file.is_some() || status_file.is_some() {
                    return (cap_file, status_file, true);
                }
            }
        }
    }
    (None, None, false)
}

/// Parses the aggregate "cpu " line from `/proc/stat`.
pub fn parse_proc_stat_cpu_line(data: &[u8]) -> Option<(u64, u64)> {
    if !data.starts_with(b"cpu ") && !data.starts_with(b"cpu  ") {
        return None;
    }

    let mut line_end = 0;
    while line_end < data.len() && data[line_end] != b'\n' {
        line_end += 1;
    }
    let line = &data[..line_end];

    let mut iter = line
        .split(|&b| b == b' ' || b == b'\t')
        .filter(|s| !s.is_empty());
    // skip the "cpu" token
    let _ = iter.next()?;

    let mut busy: u64 = 0;
    let mut total: u64 = 0;

    for (i, token) in iter.enumerate() {
        let val = parse_u64_from_ascii(token).unwrap_or(0);
        total += val;
        // 0: user, 1: nice, 2: system, 3: idle, 4: iowait, 5: irq, 6: softirq, 7: steal
        if i == 0 || i == 1 || i == 2 || i == 5 || i == 6 || i == 7 {
            busy += val;
        }
        if i >= 9 {
            break;
        }
    }

    Some((busy, total))
}

/// Parses a numeric value from `/proc/meminfo` given a field prefix (e.g. `b"MemTotal:"`).
pub fn parse_meminfo_field(data: &[u8], field: &[u8]) -> u64 {
    let mut cursor = 0;
    while cursor < data.len() {
        let mut line_end = cursor;
        while line_end < data.len() && data[line_end] != b'\n' {
            line_end += 1;
        }
        let line = &data[cursor..line_end];

        if line.starts_with(field) {
            let rest = &line[field.len()..];
            for token in rest
                .split(|&b| b == b' ' || b == b'\t')
                .filter(|s| !s.is_empty())
            {
                if let Some(val) = parse_u64_from_ascii(token) {
                    return val;
                }
            }
        }
        cursor = line_end + 1;
    }
    0
}

/// Zero-allocation integer parser from byte slice.
pub fn parse_u64_from_ascii(slice: &[u8]) -> Option<u64> {
    let trimmed = slice.trim_ascii();
    if trimmed.is_empty() {
        return None;
    }
    let mut acc: u64 = 0;
    for &b in trimmed {
        if b.is_ascii_digit() {
            acc = acc.checked_mul(10)?.checked_add((b - b'0') as u64)?;
        } else {
            break;
        }
    }
    Some(acc)
}

/// Discovers top active processes and computes delta ticks over a 250ms non-blocking window.
fn find_top_cpu_culprit_two_point() -> (Option<i32>, Option<String>, f32) {
    let snap1 = take_proc_cpu_snapshot();
    if snap1.is_empty() {
        return (None, None, 0.0);
    }

    // 250ms delta window on this background worker thread
    thread::sleep(Duration::from_millis(250));

    let snap2 = take_proc_cpu_snapshot();
    let mut max_delta = 0u64;
    let mut best_pid = None;
    let mut best_name = None;

    for (pid, comm, ticks2) in snap2 {
        if let Some((_, _, ticks1)) = snap1.iter().find(|(p, _, _)| *p == pid) {
            let delta = ticks2.saturating_sub(*ticks1);
            if delta > max_delta {
                max_delta = delta;
                best_pid = Some(pid);
                best_name = Some(comm);
            }
        }
    }

    let approx_cpu_pct = if max_delta > 0 {
        // Across 250ms, 100 ticks/sec standard USER_HZ -> 25 ticks = 100% of 1 core
        (max_delta as f32 / 25.0 * 100.0).clamp(1.0, 100.0)
    } else {
        0.0
    };

    (best_pid, best_name, approx_cpu_pct)
}

/// Scans `/proc/[pid]/stat` for active processes, capturing cumulative ticks (`utime + stime`).
fn take_proc_cpu_snapshot() -> Vec<(i32, String, u64)> {
    let mut results = Vec::with_capacity(32);
    let proc_dir = Path::new("/proc");

    if let Ok(entries) = fs::read_dir(proc_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(pid_str) = file_name.to_str() else {
                continue;
            };
            let Ok(pid) = pid_str.parse::<i32>() else {
                continue;
            };

            let stat_path = entry.path().join("stat");
            let mut buf = [0u8; 512];
            let Ok(mut file) = fs::File::open(&stat_path) else {
                continue;
            };
            let Ok(bytes_read) = file.read(&mut buf) else {
                continue;
            };
            if bytes_read == 0 {
                continue;
            }

            if let Some((comm, ticks)) = parse_proc_stat_pid(&buf[..bytes_read]) {
                results.push((pid, comm, ticks));
            }
        }
    }

    // Sort descending by cumulative ticks to focus on high-activity PIDs
    results.sort_by_key(|a| std::cmp::Reverse(a.2));
    results.truncate(32);
    results
}

/// Parse `comm` and `utime + stime` from `/proc/[pid]/stat` bytes.
pub fn parse_proc_stat_pid(data: &[u8]) -> Option<(String, u64)> {
    // Format: "pid (comm) state ppid pgrp session tty_nr tpgid flags minflt cminflt majflt cmajflt utime stime ..."
    let open_paren = data.iter().position(|&b| b == b'(')?;
    let close_paren = data.iter().rposition(|&b| b == b')')?;
    if close_paren <= open_paren {
        return None;
    }

    let comm = String::from_utf8_lossy(&data[open_paren + 1..close_paren]).to_string();
    let after_paren = &data[close_paren + 1..];

    let mut fields = after_paren.split(|&b| b == b' ').filter(|s| !s.is_empty());
    // fields after close_paren:
    // 0: state
    // 1: ppid
    // 2: pgrp
    // 3: session
    // 4: tty_nr
    // 5: tpgid
    // 6: flags
    // 7: minflt
    // 8: cminflt
    // 9: majflt
    // 10: cmajflt
    // 11: utime (field 14 in man proc)
    // 12: stime (field 15 in man proc)
    for _ in 0..11 {
        fields.next()?;
    }

    let utime_bytes = fields.next()?;
    let stime_bytes = fields.next()?;

    let utime = parse_u64_from_ascii(utime_bytes)?;
    let stime = parse_u64_from_ascii(stime_bytes)?;

    Some((comm, utime + stime))
}

/// Scans `/proc/[pid]/statm` for highest RSS memory consumer.
fn find_top_memory_culprit() -> (Option<i32>, Option<String>, f32) {
    let mut best_pid = None;
    let mut best_name = None;
    let mut max_rss_pages: u64 = 0;

    let proc_dir = Path::new("/proc");
    if let Ok(entries) = fs::read_dir(proc_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(pid_str) = file_name.to_str() else {
                continue;
            };
            let Ok(pid) = pid_str.parse::<i32>() else {
                continue;
            };

            let statm_path = entry.path().join("statm");
            let mut buf = [0u8; 128];
            let Ok(mut file) = fs::File::open(&statm_path) else {
                continue;
            };
            let Ok(bytes_read) = file.read(&mut buf) else {
                continue;
            };
            if bytes_read == 0 {
                continue;
            }

            // statm: "size resident shared text lib data dt"
            let mut fields = buf[..bytes_read]
                .split(|&b| b == b' ')
                .filter(|s| !s.is_empty());
            let _ = fields.next(); // size
            if let Some(rss_bytes) = fields.next()
                && let Some(rss) = parse_u64_from_ascii(rss_bytes)
                && rss > max_rss_pages
            {
                max_rss_pages = rss;
                best_pid = Some(pid);
            }
        }
    }

    if let Some(pid) = best_pid {
        let comm_path = format!("/proc/{pid}/comm");
        best_name = fs::read_to_string(comm_path)
            .ok()
            .map(|s| s.trim().to_string());
    }

    // Standard 4KB page -> MB
    let rss_mb = (max_rss_pages * 4) as f32 / 1024.0;
    (best_pid, best_name, rss_mb)
}

/// Main background telemetry worker loop.
fn telemetry_worker_loop() {
    let mut reader = ProcfsReader::new();

    loop {
        // Construct poll fds for PSI events
        let mut poll_fds = [
            libc::pollfd {
                fd: reader
                    .psi_cpu_file
                    .as_ref()
                    .map(|f| f.as_raw_fd())
                    .unwrap_or(-1),
                events: libc::POLLPRI,
                revents: 0,
            },
            libc::pollfd {
                fd: reader
                    .psi_mem_file
                    .as_ref()
                    .map(|f| f.as_raw_fd())
                    .unwrap_or(-1),
                events: libc::POLLPRI,
                revents: 0,
            },
        ];

        let poll_count = if reader.psi_cpu_file.is_some() || reader.psi_mem_file.is_some() {
            // Wait up to 5000ms for PSI stall triggers
            unsafe { libc::poll(poll_fds.as_mut_ptr(), 2, 5000) }
        } else {
            thread::sleep(Duration::from_millis(5000));
            0
        };

        let cpu_stalled = poll_count > 0 && (poll_fds[0].revents & libc::POLLPRI != 0);
        let mem_stalled = poll_count > 0 && (poll_fds[1].revents & libc::POLLPRI != 0);

        let cpu_percent = reader.sample_cpu();
        let (mem_percent, mem_used_mb, mem_total_mb) = reader.sample_memory();
        let (battery_percent, battery_charging) = reader.sample_battery();

        let tripped_now = cpu_percent >= 85
            || mem_percent >= 80
            || cpu_stalled
            || mem_stalled
            || (reader.battery_present
                && !battery_charging
                && battery_percent.unwrap_or(100) <= 20);

        let now = Instant::now();
        if tripped_now {
            if !reader.anomaly_active {
                reader.anomaly_active = true;
                reader.tripped_at = Some(now);
            }
            reader.recovery_start = None;
        } else if reader.anomaly_active {
            let recovered_level = cpu_percent <= 75
                && mem_percent <= 72
                && !cpu_stalled
                && !mem_stalled
                && (!reader.battery_present
                    || battery_charging
                    || battery_percent.unwrap_or(100) > 20);

            if recovered_level {
                let rec_start = reader.recovery_start.get_or_insert(now);
                let recovery_duration = now.duration_since(*rec_start);
                let min_display_met = reader
                    .tripped_at
                    .map(|t| now.duration_since(t) >= Duration::from_secs(3))
                    .unwrap_or(true);

                if recovery_duration >= Duration::from_secs(3) && min_display_met {
                    reader.anomaly_active = false;
                    reader.tripped_at = None;
                    reader.recovery_start = None;
                }
            } else {
                reader.recovery_start = None;
            }
        }

        let needs_attribution = reader.anomaly_active;
        let (culprit_pid, culprit_name, culprit_delta_percent) = if needs_attribution {
            reader.attribute_culprit(mem_stalled || (mem_percent >= 80 && cpu_percent < 85))
        } else {
            (None, None, 0.0)
        };

        let snapshot = TelemetrySnapshot {
            cpu_percent,
            mem_percent,
            mem_used_mb,
            mem_total_mb,
            battery_percent,
            battery_charging,
            battery_present: reader.battery_present,
            cpu_stalled,
            mem_stalled,
            culprit_pid,
            culprit_name,
            culprit_delta_percent,
            anomaly_active: reader.anomaly_active,
            timestamp: Instant::now(),
        };

        let changed = {
            let current = TELEMETRY_STATE.load();
            current.cpu_percent != snapshot.cpu_percent
                || current.mem_percent != snapshot.mem_percent
                || current.cpu_stalled != snapshot.cpu_stalled
                || current.mem_stalled != snapshot.mem_stalled
                || current.battery_percent != snapshot.battery_percent
                || current.battery_charging != snapshot.battery_charging
                || current.anomaly_active != snapshot.anomaly_active
                || current.culprit_pid != snapshot.culprit_pid
        };

        if changed {
            TELEMETRY_STATE.store(Arc::new(snapshot));
            // Wake Amane layer shell if an anomalous state tripped or cleared
            amane::request_frame();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_proc_stat_cpu_line() {
        let sample =
            b"cpu  2457056 40 442829 17561402 4619080 109459 36653 0 0 0\ncpu0 324263 4 56336";
        let (busy, total) = parse_proc_stat_cpu_line(sample).expect("failed to parse");
        // busy = 2457056 + 40 + 442829 + 109459 + 36653 + 0 = 3046037
        // total = busy + 17561402 (idle) + 4619080 (iowait) = 25226519
        assert_eq!(busy, 3046037);
        assert_eq!(total, 25226519);
    }

    #[test]
    fn test_parse_meminfo_field() {
        let sample = b"MemTotal:        7432876 kB\nMemFree:          488296 kB\nMemAvailable:    3057288 kB\n";
        let total = parse_meminfo_field(sample, b"MemTotal:");
        let avail = parse_meminfo_field(sample, b"MemAvailable:");
        assert_eq!(total, 7432876);
        assert_eq!(avail, 3057288);
    }

    #[test]
    fn test_parse_proc_stat_pid() {
        let sample = b"1234 (rustc) R 1000 1234 1000 0 -1 4194304 100 0 0 0 250 50 10 20";
        let (comm, ticks) = parse_proc_stat_pid(sample).expect("failed to parse pid stat");
        assert_eq!(comm, "rustc");
        assert_eq!(ticks, 300); // 250 + 50
    }

    #[test]
    fn test_telemetry_snapshot_thresholds() {
        let mut snap = TelemetrySnapshot::new();
        assert!(!snap.is_anomalous());
        assert!(!snap.is_critical());

        snap.cpu_percent = 86;
        assert!(snap.is_anomalous());
        assert!(!snap.is_critical());

        snap.cpu_percent = 96;
        assert!(snap.is_anomalous());
        assert!(snap.is_critical());

        snap.cpu_percent = 20;
        snap.battery_present = true;
        snap.battery_charging = false;
        snap.battery_percent = Some(15);
        assert!(snap.is_anomalous());
        assert!(!snap.is_critical());

        snap.battery_percent = Some(8);
        assert!(snap.is_critical());
    }
}
