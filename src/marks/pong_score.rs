//! `marks::pong_score` (0x4B8382): the pong score: the best (`dword_7687CC`) raised to the
//! hits (`dword_7687D0`) when they reach it; the place 0x100, 0x384; the hits (low byte,
//! kept) as three `marks::pong_digit`s (hundreds, tens, ones), 0x20 on, the best (low
//! byte, kept) the same.
//!
//! In:  nothing
//! Out: eax / edx the best's ones, ebx 10; the rest as the digits leave them
//!
//! Caller: `marks::run_pong`. The digits through `part` (1), in the game their Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose digit
//! calls go to a stand-in, against the Rust version, hits and bests by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::marks::pong;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pong_score", rva(0x4B8382), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `pong_digit`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A number of the score as three digits (the digits keep every register: the registers
/// set for each as the original's divisions leave them).
unsafe fn number(g: &mut Regs, at: usize, part: &mut Part) -> Result<(), Refused> {
    unsafe {
        let (n, [hundreds, tens, ones]) = rc2k_core::marks::pong::score_digits(game_read(at));
        game_write(at, n);
        [g.eax, g.edx, g.ebx] = [hundreds, n % 100, 100];
        part(1, g)?;
        [g.eax, g.edx, g.ebx] = [tens, ones, 10];
        part(1, g)?;
        g.eax = ones;
        part(1, g)?;
    }
    Ok(())
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::pong::{best, score_digits}`
pub unsafe fn pong_score(entry: &Regs, mut part: Part) -> Result<Regs, Refused> {
    use rc2k_core::marks::pong::{SCORE_AT, SCORE_GAP, best};
    let mut g = *entry;
    unsafe {
        let hits: u32 = game_read(pong::HITS_RVA);
        let was: u32 = game_read(pong::BEST_RVA);
        if best(hits, was) != was {
            game_write(pong::BEST_RVA, hits);
        }
        game_write(pong::DIGIT_AT[0], SCORE_AT[0]);
        game_write(pong::DIGIT_AT[1], SCORE_AT[1]);
        number(&mut g, pong::HITS_RVA, &mut part)?;
        game_write(
            pong::DIGIT_AT[0],
            game_read::<u32>(pong::DIGIT_AT[0]).wrapping_add(SCORE_GAP),
        );
        number(&mut g, pong::BEST_RVA, &mut part)?;
    }
    Ok(g)
}

/// In the game: `pong_digit`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::pong_digit as pd;
    *r = unsafe { pd::pong_digit(r, &mut pd::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { pong_score(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_7687D0
    let entry: &[u8] = &[0xA1, 0xD0, 0x87, 0x76, 0x00];
    let copy = match copy(
        0x4B8382,
        0x96,
        entry,
        &[0x3E, 0x4E, 0x55, 0x79, 0x89, 0x90],
        &[(0x4B8418, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = [
        pong::HITS_RVA,
        pong::BEST_RVA,
        pong::DIGIT_AT[0],
        pong::DIGIT_AT[1],
    ]
    .map(at)
    .to_vec();
    let saved = capture(&regions);
    for case in 0..5u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&regions, &saved);
                game_write(pong::HITS_RVA, [0u32, 7, 123, 0x1FF, 99][case as usize]);
                game_write(pong::BEST_RVA, [5u32, 7, 200, 0x101, 255][case as usize]);
            },
            &|r| unsafe { pong_score(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
