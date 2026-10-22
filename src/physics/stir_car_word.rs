//! `physics::stir_car_word` (0x440F2E): `dword_719FE4` (the work copy's +0xC, the word
//! `physics::test_car_word` tests) = `scene::noise_step` of itself.
//!
//! In:  nothing
//! Out: eax the new word; the rest kept (`noise_step` keeps edx); no flags read
//!
//! Caller: `sub_43B635`, its first instruction: every physics step.
//! Self-test: a copy of the original on a test word (its call to `noise_step` kept),
//! 64 words.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::scene::next_noise::noise_step;
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::stir_car_word", rva(0x440F2E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORD_RVA: usize = rva(0x719FE4); // dword_719FE4

/// The Rust core on the word at `at`: the new word.
pub unsafe fn stir_word_at(at: u32) -> u32 {
    unsafe {
        let p = at as *mut u32;
        let w = noise_step(p.read_unaligned());
        p.write_unaligned(w);
        w
    }
}

/// The Rust version.
pub unsafe fn stir_car_word() -> u32 {
    unsafe { stir_word_at(game_addr(WORD_RVA) as u32) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { stir_car_word() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(WORD_RVA) as u32, 4)],
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut word = Box::new(0u32);
    let at = &mut *word as *mut u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x440F2E,
            len: 0x10,
            entry: &[0xA1, 0xE4, 0x9F, 0x71, 0x00],
            calls: &[5],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x719FE4, at as u32)],
        2,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let mut w = 0x1234_5678u32;
    for _ in 0..64 {
        w = w.wrapping_mul(0x9E37_79B9).rotate_left(7) ^ 0x0040_0006;
        unsafe { at.write_volatile(w) };
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = unsafe { at.read_volatile() };
        unsafe { at.write_volatile(w) };
        let mine = unsafe { stir_word_at(at as u32) };
        let ok = theirs == mine
            && unsafe { at.read_volatile() } == theirs
            && Left::nothing()
                .with(EAX, mine)
                .matches(&Regs::SENTINEL, &ro);
        t.check(ok, || {
            format!(
                "word {w:#x}: original {theirs:#x} (eax {:#x}); rust {mine:#x}",
                ro.eax
            )
        });
    }
    drop((copy, word));
    t.finish()
}
