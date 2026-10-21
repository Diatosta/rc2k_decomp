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
    kind: rc2k_core::net::race::Kind::Position,
};

/// The Rust version.
pub fn send_position(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::race::Kind::pack` (in `send_fields`).
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
