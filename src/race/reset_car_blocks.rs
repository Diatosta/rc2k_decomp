//! `race::reset_car_blocks` (0x43658A): the eight car blocks set for a race: +0x25C 1 for
//! the race's cars (`g_raceCars`) else 0, +0x38 1, +0x8C `dword_601430` x 100, 22 more
//! dwords 0; each block's +0x24 its kind (`g_raceCarKinds` and the seven after).
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block; the rest preserved
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` (0x43623A) and its four set-ups
//! (`sub_43645B`, `sub_43647F`, `race::set_up_race_cars`, `sub_4364BD`). Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern, 0 .. 9 and 0x80000000 cars, both versions, put back.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_car_blocks", rva(0x43658A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_blocks(r) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &block_regions(), &rust);
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
    let kept = saved_regions();
    let saved = capture(&kept);
    let regions = block_regions();
    for (k, cars) in [0u32, 1, 2, 5, 8, 9, 0x8000_0000].into_iter().enumerate() {
        check_state(
            &mut t,
            &format!("{cars:#x} cars"),
            call_original,
            Regs::SENTINEL,
            &|| prepare(0x1357_9BDF ^ k as u32, 0, 1, cars, [0, 0]),
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
