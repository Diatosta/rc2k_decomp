//! `sound::send_command_8` (0x409BB4): sends the audio thread command 0x8 with the
//! registers as they came, and does not wait.
//!
//! In:  the registers go into the command
//! Out: every register preserved
//! `pusha; push 8; call; pop eax; popa`. Callers: five (`sound::apply_settings`, 0x43A6F4, 0x43A735, 0x467F1E, 0x467F42).
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch. See `sound/plain_sender.rs` for shadow mode and the self-test.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::sound::plain_sender::{SEND, dispatch_plain, test_plain};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::send_command_8", rva(0x409BB4), thunk, &ORIGINAL)
    .private_copy(0xB)
    .relocate_calls(&[3])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COMMAND: u32 = 0x8;
const ENTRY: [u8; 8] = [0x60, 0x6A, 0x08, 0xE8, 0x89, 0x00, 0x00, 0x00];

// ---- Rust version ----

/// `call(callee, command, registers)` for each call the original makes.
pub fn send_command_8(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, COMMAND, r);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, send_command_8);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_plain(&mut t, 0x409BB4, 0xB, &ENTRY, &[3], &[SEND], send_command_8);
    t.finish()
}
