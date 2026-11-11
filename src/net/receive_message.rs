//! `net::receive_message` (0x419722): one message taken from DirectPlay: its size
//! (`received_size`) set to 0x100, `Receive` (`IDirectPlay4A` +0x64: the sender into
//! `received_from`, the addressee into `received_to`, flags 1 DPRECEIVE_ALL, the bytes into
//! `receive_data`) through `g_dplay`; the size made 0 if it failed (no message, or one too
//! big for the buffer).
//!
//! Returns `Received`: the sender (0 a system message) and the size (0 none); the bytes
//! (`receive_data`) are not in `Received`. The original leaves the processor flags on the
//! method's result, which its one caller does not read (it tests the size next).
//! Caller: `net::take_messages`. `Receive` from Rust (`net::dplay`). Refuses
//! nothing (no check of `g_dplay`: its caller makes it).

use crate::net::owner::Net;

/// The buffer's size.
pub const ROOM: u32 = 0x100;
const _: () = assert!(ROOM as usize == crate::net::dp_session::MESSAGE_MOST);

/// A message taken (`size` 0: none).
#[derive(Clone, Copy, Debug)]
pub struct Received {
    pub from: u32,
    pub size: u32,
}
pub fn receive_message(net: &Net) -> Received {
    net.vars.received_size.set(ROOM);
    let mut data = [0u8; ROOM as usize];
    let r = crate::net::dp_objects::receive(net.vars.dplay.get(), &mut data);
    if let Some((from, to)) = r.from_to {
        net.vars.received_from.set(from);
        net.vars.received_to.set(to);
    }
    if let Some(size) = r.size {
        net.vars.received_size.set(size);
    }
    if r.result == 0 {
        net.vars
            .receive_data
            .write_at(
                0,
                &data[..r.size.unwrap_or(0) as usize],
                "the received message",
            )
            .unwrap_or_else(|e| crate::fatal::raise::end("net::receive_message", e));
    }
    let result = r.result;
    if result != 0 {
        net.vars.received_size.set(0);
    }
    Received {
        from: net.vars.received_from.get(),
        size: net.vars.received_size.get(),
    }
}
