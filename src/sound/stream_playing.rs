//! `sound::stream_playing` (0x40C4D5): the music stream's playing flag (`dword_506960`,
//! `g_soundStream.playing`) compared with 1: zero set when it plays, carry when it is 0.
//!
//! In:  nothing
//! Out: the flags of `cmp dword_506960, 1`; every register kept
//!
//! Callers: `sub_46F7AD` (`jz`), `sub_47F314` (`jnz`); `sound::step_car_stream` has it
//! inlined. Self-test: the original and the Rust version on the flag 0, 1, 2 and -1, both
//! flags compared.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stream_playing", rva(0x40C4D5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLAYING_RVA: usize = rva(0x506960); // g_soundStream.playing

/// The Rust version: (zero, carry) as the original's `cmp` leaves them.
pub fn stream_playing() -> (bool, bool) {
    let playing: u32 = unsafe { game_read(PLAYING_RVA) };
    (playing == 1, playing < 1)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (zero, carry) = stream_playing();
    r.set_zero(zero);
    r.set_carry(carry);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let vars = [at(PLAYING_RVA)];
    let saved = capture(&vars);
    for playing in [0u32, 1, 2, 0xFFFF_FFFF] {
        unsafe { game_write(PLAYING_RVA, playing) };
        let mut o = Regs::SENTINEL;
        unsafe { call_original(&mut o) };
        let theirs = (crate::audio::buffer_play::zero(&o), o.carry());
        let mine = stream_playing();
        t.check(
            theirs == mine && o.preserved_except(&Regs::SENTINEL, 0),
            || format!("playing {playing:#x}: original (zero, carry) {theirs:?}, rust {mine:?}"),
        );
    }
    restore(&vars, &saved);
    t.finish()
}
