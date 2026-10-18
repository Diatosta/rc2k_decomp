//! `marks::move_pong_bat` (0x4B82B0): the pong bat's place kept (`dword_7686AC`,
//! `dword_7686B0`), then `marks::pong_bat_back`. The keys' moves between (up with
//! `byte_4F6B94` and `byte_4F6B91`, down with `byte_4F6B84`, none with `byte_4F6B86`, held
//! to `flt_7691E6` .. ) are jumped over by the original's two `jmp short`s: the bat
//! never moves (section 16).
//!
//! In:  nothing
//! Out: as `pong_bat_back` leaves them (eax / ebx the bat's place going in)
//!
//! Caller: `marks::run_pong`. The call is its Rust version. Self-test: the original and
//! the Rust version on balls and bats by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::pong;
use crate::marks::pong_bat_back::pong_bat_back;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::move_pong_bat", rva(0x4B82B0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn move_pong_bat(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(pong::BAT[0]);
        g.ebx = game_read(pong::BAT[1]);
        game_write(pong::SAVED_BAT[0], g.eax);
        game_write(pong::SAVED_BAT[1], g.ebx);
        pong_bat_back(&g)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { move_pong_bat(r) }))
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
