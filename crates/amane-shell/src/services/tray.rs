use std::thread;
use std::time::Duration;

use amane::{Argument, Bus, Method, Service};

/// Represents an external application registered via StatusNotifierItem (SNI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayItem {
    pub service: String,
    pub path: String,
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub status: String,
}

impl TrayItem {
    /// Maps the application ID or icon name to a recognizable Nerd Font glyph.
    pub fn glyph(&self) -> &'static str {
        let needle = format!("{} {}", self.id, self.icon_name).to_lowercase();
        if needle.contains("discord") || needle.contains("vesktop") {
            "󰙯"
        } else if needle.contains("steam") {
            "󰓓"
        } else if needle.contains("telegram") {
            "󰔁"
        } else if needle.contains("spotify") {
            "󰓇"
        } else if needle.contains("obs") {
            "󰐌"
        } else if needle.contains("fcitx") || needle.contains("input") {
            "󰌌"
        } else if needle.contains("vlc") {
            "󰕼"
        } else if needle.contains("dropbox") {
            "󰇣"
        } else if needle.contains("nextcloud") {
            "󰍡"
        } else if needle.contains("torrent") || needle.contains("transmission") {
            "󰇚"
        } else if needle.contains("slack") {
            "󰒱"
        } else if needle.contains("element") || needle.contains("matrix") {
            "󰘑"
        } else if needle.contains("flameshot") {
            "󰹑"
        } else if needle.contains("keepass")
            || needle.contains("bitwarden")
            || needle.contains("pass")
        {
            "󰌋"
        } else {
            "󰀻"
        }
    }
}

#[derive(Default)]
pub struct TrayService {
    pub items: Vec<TrayItem>,
    pub watcher_active: bool,
}

impl Service for TrayService {
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }

    fn listen() {
        // Event-driven via StatusNotifierWatcher server started by TrayService::start()
    }
}

impl TrayService {
    /// Starts the background D-Bus StatusNotifierWatcher service.
    pub fn start() {
        thread::spawn(|| {
            Self::run_watcher_server();
        });
    }

    fn run_watcher_server() {
        let bus = Bus::session();
        let methods = bus.methods("/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher");

        let owned_kde = bus.own("org.kde.StatusNotifierWatcher");
        let _owned_fd = bus.own("org.freedesktop.StatusNotifierWatcher");

        if owned_kde {
            let mut tray = TrayService::write();
            tray.watcher_active = true;
        }

        for method in methods {
            Self::handle_method(&method);
        }
    }

    fn handle_method(method: &Method) {
        let bus = Bus::session();
        match method.name() {
            "RegisterStatusNotifierItem" => {
                if let Some(arg) = method.arguments().first() {
                    let service_or_path = arg.text().to_string();
                    if !service_or_path.is_empty() {
                        let (service, path) = if service_or_path.starts_with('/') {
                            (service_or_path.clone(), service_or_path)
                        } else {
                            (service_or_path, "/StatusNotifierItem".to_string())
                        };

                        Self::register_item(&service, &path);
                        bus.emit(
                            "/StatusNotifierWatcher",
                            "org.kde.StatusNotifierWatcher",
                            "StatusNotifierItemRegistered",
                            &[Argument::from(service.as_str())],
                        );
                    }
                }
                method.reply(&[]);
            }
            "RegisterStatusNotifierHost" => {
                bus.emit(
                    "/StatusNotifierWatcher",
                    "org.kde.StatusNotifierWatcher",
                    "StatusNotifierHostRegistered",
                    &[],
                );
                method.reply(&[]);
            }
            _ => {
                method.reply(&[]);
            }
        }
    }

    fn register_item(service: &str, path: &str) {
        let bus = Bus::session();
        let id = bus
            .property(service, path, "org.kde.StatusNotifierItem", "Id")
            .text()
            .to_string();
        let title = bus
            .property(service, path, "org.kde.StatusNotifierItem", "Title")
            .text()
            .to_string();
        let icon_name = bus
            .property(service, path, "org.kde.StatusNotifierItem", "IconName")
            .text()
            .to_string();
        let status = bus
            .property(service, path, "org.kde.StatusNotifierItem", "Status")
            .text()
            .to_string();

        let item = TrayItem {
            service: service.to_string(),
            path: path.to_string(),
            id: if id.is_empty() {
                service.to_string()
            } else {
                id
            },
            title: if title.is_empty() {
                service.to_string()
            } else {
                title
            },
            icon_name,
            status,
        };

        let mut tray = TrayService::write();
        if let Some(pos) = tray
            .items
            .iter()
            .position(|i| i.service == service && i.path == path)
        {
            tray.items[pos] = item;
        } else {
            tray.items.push(item);
        }
    }

    /// Triggers the primary Activate action on the tray item.
    pub fn activate(index: usize) {
        let (service, path) = {
            let tray = TrayService::read();
            if let Some(item) = tray.items.get(index) {
                (item.service.clone(), item.path.clone())
            } else {
                return;
            }
        };

        Bus::session().call(
            &service,
            &path,
            "org.kde.StatusNotifierItem",
            "Activate",
            &[Argument::from(0), Argument::from(0)],
        );
    }

    /// Triggers the secondary/ContextMenu action on the tray item.
    pub fn context_menu(index: usize) {
        let (service, path) = {
            let tray = TrayService::read();
            if let Some(item) = tray.items.get(index) {
                (item.service.clone(), item.path.clone())
            } else {
                return;
            }
        };

        Bus::session().call(
            &service,
            &path,
            "org.kde.StatusNotifierItem",
            "ContextMenu",
            &[Argument::from(0), Argument::from(0)],
        );
    }

    /// Terminates the background application associated with the tray item.
    /// Looks up the owning process PID via D-Bus session bus and sends SIGTERM.
    /// If the process is still running after 2.5 seconds, escalates to SIGKILL.
    pub fn quit(index: usize) {
        let service = {
            let mut tray = TrayService::write();
            if index < tray.items.len() {
                let item = tray.items.remove(index);
                item.service
            } else {
                return;
            }
        };

        thread::spawn(move || {
            let bus = Bus::session();
            let pid_val = bus.call(
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                "GetConnectionUnixProcessID",
                &[Argument::from(service.as_str())],
            );

            let pid = pid_val.number() as u32;
            if pid > 1 {
                // Send SIGTERM for graceful exit
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }

                // Watchdog: after 2.5 seconds, escalate to SIGKILL if process remains stuck
                thread::sleep(Duration::from_millis(2500));
                let is_alive = unsafe { libc::kill(pid as i32, 0) == 0 };
                if is_alive {
                    unsafe {
                        libc::kill(pid as i32, libc::SIGKILL);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_service_default() {
        let default = TrayService::default();
        assert!(default.items.is_empty());
        assert!(!default.watcher_active);
    }

    #[test]
    fn test_tray_quit_out_of_bounds() {
        // Calling quit on an empty or out-of-bounds index should safely no-op
        TrayService::quit(999);
    }

    #[test]
    fn test_tray_quit_removes_item() {
        let mut tray = TrayService::write();
        tray.items.push(TrayItem {
            service: ":1.999999".into(),
            path: "/StatusNotifierItem".into(),
            id: "test-app".into(),
            title: "Test App".into(),
            icon_name: "test".into(),
            status: "Active".into(),
        });
        let len_before = tray.items.len();
        drop(tray);

        TrayService::quit(len_before - 1);

        let tray_after = TrayService::read();
        assert_eq!(tray_after.items.len(), len_before - 1);
    }

    #[test]
    fn test_tray_item_glyphs() {
        let discord = TrayItem {
            service: "org.kde.StatusNotifierItem-1-1".into(),
            path: "/StatusNotifierItem".into(),
            id: "discord".into(),
            title: "Discord".into(),
            icon_name: "discord".into(),
            status: "Active".into(),
        };
        assert_eq!(discord.glyph(), "󰙯");

        let steam = TrayItem {
            service: "org.kde.StatusNotifierItem-1-2".into(),
            path: "/StatusNotifierItem".into(),
            id: "steam".into(),
            title: "Steam".into(),
            icon_name: "steam_tray_mono".into(),
            status: "Active".into(),
        };
        assert_eq!(steam.glyph(), "󰓓");

        let unknown = TrayItem {
            service: "org.kde.StatusNotifierItem-1-3".into(),
            path: "/StatusNotifierItem".into(),
            id: "custom-app".into(),
            title: "Custom".into(),
            icon_name: "app-icon".into(),
            status: "Active".into(),
        };
        assert_eq!(unknown.glyph(), "󰀻");
    }
}
