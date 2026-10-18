//! `sound::send_music_command` (0x409BF6): sends the audio thread command 0xB with the
//! registers as they came, and does not wait.
//!
//! In:  the registers go into the command
//! Out: every register preserved
//! `pusha; push 0xB; call; pop eax; popa`. Callers: `sound::play_music`, `stop_music`, `set_music_volume` (whose Rust versions send it themselves).
//! A call in its first five bytes: private copy (0xB bytes, the call relocated), manual
//! patch. See `sound/plain_sender.rs` for shadow mode and the self-test.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::sound::plain_sender::{SEND, dispatch_plain, test_plain};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::send_music_command", rva(0x409BF6), thunk, &ORIGINAL)
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

const COMMAND: u32 = 0xB;
const ENTRY: [u8; 8] = [0x60, 0x6A, 0x0B, 0xE8, 0x47, 0x00, 0x00, 0x00];

// ---- Rust version ----

/// `call(callee, command, registers)` for each call the original makes.
pub fn send_music_command(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, COMMAND, r);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, send_music_command);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_plain(
        &mut t,
        0x409BF6,
        0xB,
        &ENTRY,
        &[3],
        &[SEND],
        send_music_command,
    );
    t.finish()
}
