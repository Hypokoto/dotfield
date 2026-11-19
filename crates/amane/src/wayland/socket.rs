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

    // the event loop only wakes this up when a client is waiting, so accept must never block
    listener
        .set_nonblocking(true)
        .expect("failed to make ipc socket non-blocking");

    listener
}

fn answer(mut stream: UnixStream, handlers: &IpcHandlers) {
    // a client that never finishes sending must not freeze the shell
    let timeout = Some(Duration::from_millis(500));

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
