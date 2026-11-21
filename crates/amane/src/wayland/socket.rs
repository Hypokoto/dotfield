use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::Duration;

use smithay_client_toolkit::reexports::calloop::{
    Interest, LoopHandle, Mode, PostAction, generic::Generic,
};

use crate::ipc::{IpcCall, IpcHandlers, ipc_socket, opcode_to_name};

use super::WaylandState;

pub fn insert(handle: &LoopHandle<'static, WaylandState>, handlers: IpcHandlers) {
    let source = Generic::new(listen(), Interest::READ, Mode::Level);

    handle
        .insert_source(source, move |_, listener, state| {
            // a failed call only loses that call, the shell keeps running
            if let Ok((stream, _)) = listener.accept() {
                answer(stream, &handlers);

                state.relocate_plain_overlays();
                state.request_changed_frames();
            }

            Ok(PostAction::Continue)
        })
        .expect("failed to insert ipc socket");
}

fn listen() -> UnixListener {
    let path = ipc_socket();

    // a socket that still answers belongs to another running shell
    if UnixStream::connect(&path).is_ok() {
        panic!("failed to listen: amane is already running");
    }

    // what is left is a leftover file from a shell that did not exit cleanly
    let _ = fs::remove_file(&path);

    let listener = UnixListener::bind(&path).expect("failed to create ipc socket");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }

    // the event loop only wakes this up when a client is waiting, so accept must never block
    listener
        .set_nonblocking(true)
        .expect("failed to make ipc socket non-blocking");

    listener
}

#[cfg(target_os = "linux")]
fn verify_peer_uid(stream: &UnixStream) -> bool {
    use std::os::unix::io::AsRawFd;

    #[repr(C)]
    struct UCred {
        pid: i32,
        uid: u32,
        gid: u32,
    }

    unsafe extern "C" {
        fn getsockopt(
            sockfd: std::ffi::c_int,
            level: std::ffi::c_int,
            optname: std::ffi::c_int,
            optval: *mut std::ffi::c_void,
            optlen: *mut u32,
        ) -> std::ffi::c_int;
        fn getuid() -> u32;
    }

    let mut cred = UCred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<UCred>() as u32;
    // SOL_SOCKET = 1, SO_PEERCRED = 17 on Linux
    let ret = unsafe {
        getsockopt(
            stream.as_raw_fd(),
            1,
            17,
            std::ptr::from_mut(&mut cred).cast(),
            &mut len,
        )
    };
    if ret == 0 {
        cred.uid == unsafe { getuid() }
    } else {
        false
    }
}

fn answer(mut stream: UnixStream, handlers: &IpcHandlers) {
    #[cfg(target_os = "linux")]
    if !verify_peer_uid(&stream) {
        return;
    }

    // A client that never finishes sending must not freeze the shell.
    // Local Unix socket data transfers in microseconds; 25ms protects the event loop from stalling.
    let timeout = Some(Duration::from_millis(25));

    if stream.set_read_timeout(timeout).is_err() {
        return;
    }

    let mut first_byte = [0u8; 1];
    if stream.read_exact(&mut first_byte).is_err() {
        return;
    }

    let b = first_byte[0];
    if b < 0x20 {
        // Fast-path byte opcode
        if let Some((name, args)) = opcode_to_name(b) {
            let args_vec: Vec<String> = args.iter().map(|&s| s.to_string()).collect();
            let _ = handlers.run_name_and_args(name, &args_vec);
            // Write 1-byte ACK
            let _ = stream.write_all(&[0x00]);
            let _ = stream.flush();
        } else {
            let _ = stream.write_all(b"err: unknown opcode\n");
            let _ = stream.flush();
        }
        return;
    }

    // Fallback: line-based string protocol
    let reader = std::io::Cursor::new(first_byte).chain(&mut stream);
    let Ok(call) = IpcCall::read(reader) else {
        return;
    };

    let reply = handlers.run(&call);

    let _ = stream.write_all(reply.as_bytes());
    let _ = stream.flush();
}
