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
//! Caller: `net::take_messages`. `Receive` from Rust (`net::dplay`, section 15). Refuses
//! nothing (no check of `g_dplay`: its caller makes it).

use crate::net::dplay::{RECEIVE, method};
use crate::net::lobby::{DPLAY, a, g, p};

/// The buffer's size.
pub const ROOM: u32 = 0x100;
/// DPRECEIVE_ALL.
const RECEIVE_ALL: u32 = 1;
/// dword_5AB278: the sender; unk_5AB27C: the addressee; dword_5AB280: the size;
/// unk_52A968: the bytes, 0x100.
pub const FROM: usize = 0x5AB278;
const TO: usize = 0x5AB27C;
const SIZE: usize = 0x5AB280;
const DATA: usize = 0x52A968;

/// A message taken (`size` 0: none).
#[derive(Clone, Copy, Debug)]
pub struct Received {
    pub from: u32,
    pub bytes: u32,
    pub size: u32,
}

/// The Rust version.
pub fn receive_message() -> Received {
    p(SIZE, ROOM);
    let result = unsafe {
        method(
            g(DPLAY),
            RECEIVE,
            &[a(FROM), a(TO), RECEIVE_ALL, a(DATA), a(SIZE)],
        )
    };
    if result != 0 {
        p(SIZE, 0);
    }
    Received {
        from: g(FROM),
        bytes: a(DATA),
        size: g(SIZE),
    }
}
