//! `sound::send_command_7` (0x409B8C): command 7 to the audio thread, then the end of
//! the turn, then `sound::channels_in_use` (whose flags `popa` / `retn` throw away).
//!
//! In:  the registers go into the commands
//! Out: every register preserved (`pusha` / `popa`)
//! Command 7 with the registers as they came; command 0xA (`sound::send_end_of_turn`)
//! with eax 7 (the `pop eax` of the push), the rest as they came. Callers:
//! `race::reset_race_sounds` (0x40D37E) and 0x409BA9.
//! A call in its first five bytes: private copy (0x15 bytes, the three calls relocated),
//! manual patch. See `sound/plain_sender.rs` for shadow mode and the self-test.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::sound::plain_sender::{SEND, dispatch_plain, test_plain};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::send_command_7", rva(0x409B8C), thunk, &ORIGINAL)
    .private_copy(0x15)
    .relocate_calls(&[3, 9, 0xE])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const END_OF_TURN: usize = 0x409BE0;
const CHANNELS: usize = 0x409BA1;
const ENTRY: [u8; 8] = [0x60, 0x6A, 0x07, 0xE8, 0xB1, 0x00, 0x00, 0x00];

// ---- Rust version ----

pub fn send_command_7(r: &Regs, call: &mut dyn FnMut(usize, u32, &Regs)) {
    call(SEND, 7, r);
    let after = Regs { eax: 7, ..*r };
    call(END_OF_TURN, 0, &after);
    call(CHANNELS, 0, &after);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, send_command_7);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_plain(
        &mut t,
        0x409B8C,
        0x15,
        &ENTRY,
        &[3, 9, 0xE],
        &[SEND, END_OF_TURN, CHANNELS],
        send_command_7,
    );
    t.finish()
}
