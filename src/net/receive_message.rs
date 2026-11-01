//! `net::receive_message` (0x419722): one message taken from DirectPlay: its size
//! (`dword_5AB280`) set to 0x100, `Receive` (`IDirectPlay4A` +0x64: the sender into
//! `dword_5AB278`, the addressee into `unk_5AB27C`, flags 1 DPRECEIVE_ALL, the bytes into
//! `unk_52A968`) through `g_dplay`; the size made 0 if it failed (no message, or one too
//! big for the buffer).
//!
//! In:  nothing
//! Out: `Received`: the sender (eax; 0 a system message), the bytes (ebx, `unk_52A968`), the
//!      size (ecx; 0 none). The original keeps edx, esi, edi, ebp and leaves the flags of
//!      `test eax, eax` on the method's result, which its one caller does not read (it tests
//!      ecx next).
//! Caller: `net::take_messages`. `Receive` from Rust (`net::dplay`). Refuses
//! nothing (no check of `g_dplay`: its caller makes it).

use crate::net::store::NET;

/// The buffer's size.
pub const ROOM: u32 = 0x100;
const _: () = assert!(ROOM as usize == crate::net::dp_session::MESSAGE_MOST);

/// A message taken (`size` 0: none).
#[derive(Clone, Copy, Debug)]
pub struct Received {
    pub from: u32,
    pub bytes: u32,
    pub size: u32,
}

/// The Rust version.
pub fn receive_message() -> Received {
    NET.received_size.set(ROOM);
    let mut data = [0u8; ROOM as usize];
    let r = crate::net::dp_objects::receive(NET.dplay.get(), &mut data);
    if let Some((from, to)) = r.from_to {
        NET.received_from.set(from);
        NET.received_to.set(to);
    }
    if let Some(size) = r.size {
        NET.received_size.set(size);
    }
    if r.result == 0 {
        NET.receive_data
            .write_at(
                0,
                &data[..r.size.unwrap_or(0) as usize],
                "the received message",
            )
            .unwrap_or_else(|e| crate::fatal::raise::end("net::receive_message", e));
    }
    let result = r.result;
    if result != 0 {
        NET.received_size.set(0);
    }
    Received {
        from: NET.received_from.get(),
        bytes: (NET.receive_data.addr()),
        size: NET.received_size.get(),
    }
}
