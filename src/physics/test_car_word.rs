//! `physics::test_car_word` (0x44133F): eax = (`dword_719FE4` rol cl) & 0xFFFF, then
//! `cmp eax, 1`: CF when that is 0, ZF when it is 1. `dword_719FE4` is the work copy's
//! +0xC (`drive/car.rs`); what its bits are is not known yet.
//!
//! In:  cl the rotation
//! Out: eax the masked word; CF, ZF as said (callers read CF: `jnb`); the rest kept
//!
//! Callers (8): `sub_4410C8` and `sub_4411EC` (under `sub_4454D2`), four each, with cl
//! 5, 9, 0xE, ...
//! Shadow mode: `physics/flag_result.rs` (CF and ZF compared).
//! Self-test: a copy of the original on a test dword, rotations 0 .. 0x27 (the count is
//! taken mod 32) on several words.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_read, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::test_car_word", rva(0x44133F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WORD_RVA: usize = rva(0x719FE4); // dword_719FE4: g_car + 0xC

/// The Rust core on the word itself.
pub fn test_word(word: u32, cl: u8) -> FlagResult {
    let eax = word.rotate_left(u32::from(cl & 31)) & 0xFFFF;
    FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry: eax == 0,
        zero: Some(eax == 1),
    }
}

/// The Rust version.
pub fn test_car_word(cl: u8) -> FlagResult {
    test_word(unsafe { game_read(WORD_RVA) }, cl)
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(test_car_word(r.ecx as u8))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
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
            address: 0x44133F,
            len: 0x10,
            entry: &[0xA1, 0xE4, 0x9F, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x719FE4, at as u32)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for w in [
        0u32,
        1,
        0x0001_0000,
        0x8000_0000,
        0x0800_0000,
        0xFFFF_0000,
        0x1234_5678,
        u32::MAX,
    ] {
        for cl in 0u8..0x28 {
            unsafe { at.write_volatile(w) };
            let before = Regs {
                ecx: 0xABCD_EF00 | u32::from(cl),
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = test_word(w, cl);
            t.check(mine.matches(&before, &ro), || {
                format!(
                    "word {w:#x}, cl {cl:#x}: original eax {:#x} carry {} zero {}; rust {mine:?}",
                    ro.eax,
                    ro.carry(),
                    crate::audio::buffer_play::zero(&ro)
                )
            });
        }
    }
    drop(word);
    t.finish()
}
