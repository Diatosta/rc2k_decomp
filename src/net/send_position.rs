//! `net::send_position` (0x464201): "send p", the player and the step logged, and the
//! message 'PGSM' (0x4D534750) with the car's place (+0x134 .. +0x13C), its matrix (+0xB4
//! .. +0xE0) and 25 more dwords of its block (speeds, gear, engine, ...) built in
//! `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: as `send_race_message` leaves them (in: eax the last dword, ebx the message, ecx
//!      its bytes, the rest kept)
//!
//! See `net/mod.rs` and `net::send_inputs` (the shared body). One caller, `net::send_turn`
//! (step 2 of 32). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose car block and message are the test's and whose calls go to noting
//! stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::send_inputs::{Message, send_fields, test_send};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_position", rva(0x464201), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const POSITION: Message = Message {
    text: 0x4DE0E0, // "send p"
    tag: 0x4D53_4750,
    fields: &[
        0x134, 0x138, 0x13C, 0xB4, 0xB8, 0xBC, 0xC0, 0xC4, 0xC8, 0xCC, 0xD0, 0xD4, 0xD8, 0xDC,
        0xE0, 0x198, 0x19C, 0x1A0, 0x1A4, 0x1A8, 0x1AC, 0x184, 0x188, 0x18C, 0x98, 0x94, 0x1D4,
        0x1D8, 0x1B8, 0x1BC, 0x600, 0x604, 0x608, 0x60C, 0x638, 0x63C, 0x640, 0x644, 0x648, 0x1EC,
    ],
};

/// The Rust version.
pub fn send_position(r: &mut Regs, part: Part) -> Result<(), Refused> {
    send_fields(r, &POSITION, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_position);
}

fn self_test() -> (usize, usize) {
    test_send(
        &HOOK,
        0x464201,
        0x1F9,
        &[0x60, 0x8D, 0x05, 0xE0, 0xE0, 0x4D, 0x00],
        0x1F3,
        &POSITION,
    )
}
