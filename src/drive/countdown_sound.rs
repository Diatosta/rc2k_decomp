//! `drive::countdown_sound` (0x40EC9C): for a listed car (eax 0 or 1) the count down's
//! sound ebx played on its channel (`sound::play_sound`: eax + ebx, ebx and edx -0x6F,
//! ecx, edi, ebp 0). All in `pusha` / `popa`.
//!
//! In:  eax the car's sound slot, ebx the sound
//! Out: every register kept; no flags read
//!
//! Caller: `drive::countdown_cue`.
//! Shadow mode runs the original alone (a sound command).
//! Self-test: none (a sound command; `sound::play_sound`'s own test covers it).

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::countdown_sound", rva(0x40EC9C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::countdown::beep_sound`.
pub fn countdown_sound(slot: u32, sound: u32) {
    let Some(beep) = rc2k_core::drive::countdown::beep_sound(slot, sound) else {
        return;
    };
    let play = Regs {
        eax: beep.sound,
        ebx: beep.volume,
        edx: beep.volume,
        ecx: 0,
        edi: 0,
        ebp: 0,
        ..Regs::default()
    };
    crate::sound::play_sound::play_sound_in_game(&play);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    countdown_sound(r.eax, r.ebx);
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("a sound command (sound::play_sound's test covers it)")
}
