use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;

pub struct IpcCall {
    pub(crate) name: String,

    pub(crate) arguments: Vec<String>,
}

type Handler = fn(&[String]) -> String;

#[derive(Default)]
pub struct IpcHandlers {
    pub by_name: HashMap<String, Handler>,
}

impl IpcCall {
    pub fn new(name: &str, arguments: &[String]) -> Self {
        Self {
            name: String::from(name),
            arguments: arguments.to_vec(),
        }
    }

    // the name goes on the first line, then each argument on its own line
    pub fn write(&self, writer: &mut impl Write) -> io::Result<()> {
        writeln!(writer, "{}", self.name)?;

        for argument in &self.arguments {
            writeln!(writer, "{argument}")?;
        }

        Ok(())
    }

    // reads until the sender closes its side, which marks the end of the call
    pub(crate) fn read(reader: impl Read) -> io::Result<Self> {
        // Enforce 64 KiB ceiling to protect against unbounded memory allocation
        let limited = reader.take(65536);
        let mut lines = BufReader::new(limited).lines();

        let Some(name) = lines.next() else {
            return Err(io::ErrorKind::UnexpectedEof.into());
        };

        let name = name?;
        if name.len() > 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "IPC command name exceeds length limit",
            ));
        }

        let mut arguments = Vec::new();
        for line in lines {
            let arg = line?;
            if arg.len() > 4096 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "IPC argument exceeds length limit",
                ));
            }
            if arguments.len() >= 64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Too many IPC arguments",
                ));
            }
            arguments.push(arg);
        }

        Ok(Self { name, arguments })
    }
}

impl IpcHandlers {
    pub fn insert(&mut self, name: &str, handler: Handler) {
        self.by_name.insert(String::from(name), handler);
    }

    // Fast-path byte opcodes (0x01..=0x1F) for zero-allocation IPC dispatch
    pub fn run_name_and_args(&self, name: &str, args: &[String]) -> String {
        let Some(handler) = self.by_name.get(name) else {
            return format!("no handler named {name}");
        };

        handler(args)
    }

    // the reply is whatever the handler returns, so an unknown name answers in words too
    pub fn run(&self, call: &IpcCall) -> String {
        self.run_name_and_args(&call.name, &call.arguments)
    }
}

pub const OP_LAUNCHER: u8 = 0x01;
pub const OP_TERMINAL_MGR: u8 = 0x02;
pub const OP_DISPLAY_CFG: u8 = 0x03;
pub const OP_CONTROL_CTR: u8 = 0x04;
pub const OP_NETWORK: u8 = 0x05;
pub const OP_AUDIO: u8 = 0x06;
pub const OP_BLUETOOTH: u8 = 0x07;
pub const OP_TRAY: u8 = 0x08;
pub const OP_POWERMENU: u8 = 0x09;
pub const OP_CALENDAR: u8 = 0x0A;
pub const OP_SCREEN_REC: u8 = 0x0B;
pub const OP_RELOAD: u8 = 0x0C;
pub const OP_SIDECAR: u8 = 0x0D;
pub const OP_PRESENTATION: u8 = 0x0E;
pub const OP_WIDGETS: u8 = 0x0F;
pub const OP_OSD_VOL_UP: u8 = 0x10;
pub const OP_OSD_VOL_DOWN: u8 = 0x11;
pub const OP_OSD_VOL_MUTE: u8 = 0x12;
pub const OP_OSD_BRIGHT_UP: u8 = 0x13;
pub const OP_OSD_BRIGHT_DN: u8 = 0x14;
pub const OP_START_MENU: u8 = 0x15;
pub const OP_ACTION_CENTER: u8 = 0x16;
pub const OP_OVERVIEW: u8 = 0x17;
pub const OP_SETTINGS: u8 = 0x18;

pub fn opcode_to_name(opcode: u8) -> Option<(&'static str, &'static [&'static str])> {
    match opcode {
        OP_LAUNCHER => Some(("launcher", &[])),
        OP_TERMINAL_MGR => Some(("terminal-manager", &[])),
        OP_DISPLAY_CFG => Some(("display-config", &[])),
        OP_CONTROL_CTR => Some(("control-center", &[])),
        OP_NETWORK => Some(("network", &[])),
        OP_AUDIO => Some(("audio", &[])),
        OP_BLUETOOTH => Some(("bluetooth", &[])),
        OP_TRAY => Some(("tray", &[])),
        OP_POWERMENU => Some(("powermenu", &[])),
        OP_CALENDAR => Some(("calendar", &[])),
        OP_SCREEN_REC => Some(("screen-recorder", &[])),
        OP_RELOAD => Some(("reload", &[])),
        OP_SIDECAR => Some(("sidecar", &[])),
        OP_PRESENTATION => Some(("presentation", &[])),
        OP_WIDGETS => Some(("widgets", &[])),
        OP_OSD_VOL_UP => Some(("osd", &["volume-up"])),
        OP_OSD_VOL_DOWN => Some(("osd", &["volume-down"])),
        OP_OSD_VOL_MUTE => Some(("osd", &["volume-mute"])),
        OP_OSD_BRIGHT_UP => Some(("osd", &["brightness-up"])),
        OP_OSD_BRIGHT_DN => Some(("osd", &["brightness-down"])),
        OP_START_MENU => Some(("start-menu", &[])),
        OP_ACTION_CENTER => Some(("action-center", &[])),
        OP_OVERVIEW => Some(("overview", &[])),
        OP_SETTINGS => Some(("settings", &[])),
        _ => None,
    }
}

pub fn name_to_opcode(name: &str) -> Option<u8> {
    match name {
        "launcher" => Some(OP_LAUNCHER),
        "terminal-manager" | "terminals" | "topology" => Some(OP_TERMINAL_MGR),
        "display-config" => Some(OP_DISPLAY_CFG),
        "control-center" => Some(OP_CONTROL_CTR),
        "network" => Some(OP_NETWORK),
        "audio" => Some(OP_AUDIO),
        "bluetooth" => Some(OP_BLUETOOTH),
        "tray" => Some(OP_TRAY),
        "powermenu" => Some(OP_POWERMENU),
        "calendar" => Some(OP_CALENDAR),
        "screen-recorder" => Some(OP_SCREEN_REC),
        "reload" => Some(OP_RELOAD),
        "sidecar" => Some(OP_SIDECAR),
        "presentation" => Some(OP_PRESENTATION),
        "widgets" => Some(OP_WIDGETS),
        "volume-up" => Some(OP_OSD_VOL_UP),
        "volume-down" => Some(OP_OSD_VOL_DOWN),
        "volume-mute" => Some(OP_OSD_VOL_MUTE),
        "brightness-up" => Some(OP_OSD_BRIGHT_UP),
        "brightness-down" => Some(OP_OSD_BRIGHT_DN),
        "start-menu" | "start" => Some(OP_START_MENU),
        "action-center" => Some(OP_ACTION_CENTER),
        "overview" => Some(OP_OVERVIEW),
        "settings" | "setting" => Some(OP_SETTINGS),
        _ => None,
    }
}

pub fn ipc_socket() -> PathBuf {
    // every wayland session sets this, and it is private to the logged in user
    let runtime_directory =
        env::var_os("XDG_RUNTIME_DIR").expect("failed to find socket: XDG_RUNTIME_DIR is not set");

    PathBuf::from(runtime_directory).join("amane.sock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_mappings() {
        assert_eq!(name_to_opcode("launcher"), Some(OP_LAUNCHER));
        assert_eq!(name_to_opcode("terminal-manager"), Some(OP_TERMINAL_MGR));
        assert_eq!(name_to_opcode("terminals"), Some(OP_TERMINAL_MGR));
        assert_eq!(name_to_opcode("topology"), Some(OP_TERMINAL_MGR));
        assert_eq!(name_to_opcode("sidecar"), Some(OP_SIDECAR));
        assert_eq!(name_to_opcode("presentation"), Some(OP_PRESENTATION));
        assert_eq!(name_to_opcode("display-config"), Some(OP_DISPLAY_CFG));
        assert_eq!(name_to_opcode("widgets"), Some(OP_WIDGETS));
        assert_eq!(name_to_opcode("start-menu"), Some(OP_START_MENU));
        assert_eq!(name_to_opcode("start"), Some(OP_START_MENU));
        assert_eq!(name_to_opcode("action-center"), Some(OP_ACTION_CENTER));
        assert_eq!(name_to_opcode("overview"), Some(OP_OVERVIEW));
        assert_eq!(name_to_opcode("settings"), Some(OP_SETTINGS));
        assert_eq!(name_to_opcode("setting"), Some(OP_SETTINGS));
        assert_eq!(name_to_opcode("unknown-cmd"), None);

        assert_eq!(opcode_to_name(OP_LAUNCHER), Some(("launcher", &[][..])));
        assert_eq!(opcode_to_name(OP_SIDECAR), Some(("sidecar", &[][..])));
        assert_eq!(
            opcode_to_name(OP_PRESENTATION),
            Some(("presentation", &[][..]))
        );
        assert_eq!(opcode_to_name(OP_WIDGETS), Some(("widgets", &[][..])));
        assert_eq!(opcode_to_name(OP_START_MENU), Some(("start-menu", &[][..])));
        assert_eq!(
            opcode_to_name(OP_ACTION_CENTER),
            Some(("action-center", &[][..]))
        );
        assert_eq!(opcode_to_name(OP_OVERVIEW), Some(("overview", &[][..])));
        assert_eq!(opcode_to_name(OP_SETTINGS), Some(("settings", &[][..])));
        let (osd, args) = opcode_to_name(OP_OSD_VOL_UP).unwrap();
        assert_eq!(osd, "osd");
        assert_eq!(args, &["volume-up"]);
    }

    #[test]
    fn test_ipc_handlers_run_name_and_args() {
        let mut handlers = IpcHandlers::default();
        handlers.insert("test", |_| "ok".to_string());
        assert_eq!(handlers.run_name_and_args("test", &[]), "ok");
        assert_eq!(
            handlers.run_name_and_args("missing", &[]),
            "no handler named missing"
        );
    }

    #[test]
    fn test_ipc_call_read_limits() {
        // Valid small command
        let input = b"launcher\narg1\narg2\n";
        let call = IpcCall::read(&input[..]).unwrap();
        assert_eq!(call.name, "launcher");
        assert_eq!(call.arguments, vec!["arg1", "arg2"]);

        // Oversized argument (>4096 bytes) rejected
        let large_arg = "a".repeat(5000);
        let oversized = format!("command\n{large_arg}\n");
        assert!(IpcCall::read(oversized.as_bytes()).is_err());

        // Too many arguments (>64 arguments) rejected
        let mut many_args = "command\n".to_string();
        for i in 0..70 {
            many_args.push_str(&format!("arg{i}\n"));
        }
        assert!(IpcCall::read(many_args.as_bytes()).is_err());
    }
}
