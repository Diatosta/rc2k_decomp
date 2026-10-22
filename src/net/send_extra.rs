//! `net::send_extra` (0x4643FA): "send x", the player and the step logged, and the message
//! 'XGSM' (0x4D534758) with nine dwords of the car's block (+0x64 .. +0x84; not read)
//! built in `unk_71C4D0` and sent (`net::send_race_message`).
//!
//! In:  nothing
//! Out: as `send_race_message` leaves them (in: eax the last dword, ebx the message, ecx
//!      its bytes, the rest kept)
//!
//! See `net/mod.rs` and `net::send_inputs` (the shared body). One caller, `net::send_turn`
//! (step 1 of 256). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose car block and message are the test's and whose calls go to noting
//! stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::net::send_inputs::{Message, send_fields, test_send};
use crate::net::{Part, dispatch_net, net_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::send_extra", rva(0x4643FA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const EXTRA: Message = Message {
    text: 0x4DE0E7, // "send x"
    kind: rc2k_core::net::race::Kind::Extra,
};

/// The Rust version.
pub fn send_extra(r: &mut Regs, part: Part) -> Result<(), Refused> {
    // Converted: `rc2k_core::net::race::Kind::pack` (in `send_fields`).
    send_fields(r, &EXTRA, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, send_extra);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_extra)
}

fn self_test() -> (usize, usize) {
    test_send(
        &HOOK,
        0x4643FA,
        0xA4,
        &[0x60, 0x8D, 0x05, 0xE7, 0xE0, 0x4D, 0x00],
        0x9E,
        &EXTRA,
    )
}
