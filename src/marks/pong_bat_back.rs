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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::pong;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { pong_bat_back(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &pong::regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = pong::regions();
    let mut vars = regions.clone();
    vars.extend([pong::STEPS_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..8u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                pong::test_values(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
