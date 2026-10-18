//! `marks::hexagons_step` (0x4BCAD0): the steps since the hexagons were last looked at
//! (`dword_814714` less `dword_84460C`, low byte) into `dword_81622E`, `dword_84460C` the
//! clock `dword_814714`; `marks::expire_hexagons`; then `dword_84460C` moved on by those
//! steps (low byte) and back into `dword_814714`.
//!
//! In:  nothing
//! Out: as `expire_hexagons` leaves them, eax the new `dword_84460C` before its masking
//!
//! Caller: `sub_4BAC87` (0x4BAF7D). The call is `expire_hexagons`' Rust version.
//! Self-test: the original and the Rust version on the game's items (as
//! `expire_hexagons`'), clocks by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::expire_hexagons::{self, LATEST_RVA, STEPS_RVA, expire_hexagons, test_items};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::hexagons_step", rva(0x4BCAD0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CLOCK_RVA: usize = rva(0x814714); // dword_814714

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn hexagons_step(entry: &Regs) -> Regs {
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        let steps = game_read::<u32>(LATEST_RVA).wrapping_sub(clock) & 0xFF;
        game_write(LATEST_RVA, clock);
        game_write(STEPS_RVA, steps);
        let mut g = expire_hexagons(&Regs {
            eax: steps,
            ..*entry
        });
        g.eax = game_read::<u32>(LATEST_RVA).wrapping_add(game_read(STEPS_RVA));
        game_write(LATEST_RVA, g.eax & 0xFF);
        game_write(CLOCK_RVA, g.eax & 0xFF);
        g
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { hexagons_step(r) }))
}

fn regions() -> Vec<crate::service::harness::Region> {
    let mut r = expire_hexagons::regions();
    r.extend([CLOCK_RVA, LATEST_RVA, STEPS_RVA].map(at));
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.push((game_addr(rva(0x844A10)) as u32, 0x400));
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                test_items(case);
                game_write(CLOCK_RVA, case * 3 + 1);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
