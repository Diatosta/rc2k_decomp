//! `marks::pong_bat_back` (0x4B8215): the pong bat put back where it was before its move
//! (`dword_7686AC`, `dword_7686B0`) when it overlaps the ball (`marks::pong`'s four
//! compares).
//!
//! In:  nothing
//! Out: eax the last compare's status (& 0x700), or with the bat put back eax / ebx its
//!      place's bits
//!
//! Caller: `marks::move_pong_bat`. Self-test: the original and the Rust version on balls
//! and bats by lot; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `pong_bat_back_plain`, through
//! `rc2k_core::marks::pong::Court::bat_back` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::pong;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::marks::pong::Back;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::pong_bat_back", rva(0x4B8215), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pong_bat_back(entry: &Regs) -> Regs {
    let mut g = *entry;
    let (over, status) = pong::overlap();
    g.eax = status;
    if over {
        unsafe {
            g.eax = game_read(pong::SAVED_BAT[0]);
            g.ebx = game_read(pong::SAVED_BAT[1]);
            game_write(pong::BAT[0], g.eax);
            game_write(pong::BAT[1], g.ebx);
        }
    }
    g
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::pong::Court::bat_back`
pub unsafe fn pong_bat_back_plain(entry: &Regs) -> Regs {
    let mut g = *entry;
    match pong::with_court(|c| c.bat_back()) {
        Back::Kept(status) => g.eax = status,
        Back::PutBack([x, y]) => (g.eax, g.ebx) = (x, y),
    }
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pong_bat_back_plain(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    pong::dispatch_court(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    pong::test_court(
        &mut t,
        "marks::pong_bat_back",
        call_original,
        8,
        &|case| unsafe { pong::test_values(case) },
        &rust,
    );
    t.finish()
}
