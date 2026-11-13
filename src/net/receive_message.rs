//! Takes one message from DirectPlay (room for 0x100 bytes); the size is 0 if there is none
//! or it is too big. Returns the sender (0 a system message) and the size; the bytes are in
//! `receive_data`. No check for a missing object: the caller makes it.

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
