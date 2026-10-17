//! `race::set_up_race_cars` (0x4364AD): `race::reset_car_blocks`,
//! `race::drop_absent_player`, `race::set_player_car_kinds` (one of `sub_43603C`'s four
//! set-ups, by the kind of race).
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block; the rest preserved
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C` (0x436123). It begins with its call:
//! private copy, its three calls relocated, manual patch. Shadow mode: the Rust version,
//! the blocks put back, the original's, compared. Self-test: the game's blocks filled
//! with a pattern, cars, modes, players and absences of the test's, both versions, put
//! back.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::set_up_race_cars", rva(0x4364AD), thunk, &ORIGINAL)
    .private_copy(0x10)
    .relocate_calls(&[0x00, 0x05, 0x0A])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = block_regions();
    r.extend(player_regions());
    r.extend(kind_regions());
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_up_race_cars(r) }?;
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(ECX, g.ecx)
        .with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = saved_regions();
    let saved = capture(&kept);
    let regions = regions();
    for (mode, players, cars, absent) in [
        (0u32, 1u32, 1u32, [0u32, 0u32]),
        (0, 2, 2, [1, 0]),
        (1, 2, 2, [0, 1]),
        (2, 2, 2, [1, 1]),
        (3, 1, 8, [0, 0]),
    ] {
        check_state(
            &mut t,
            &format!("mode {mode}, {players} players, {cars} cars, absent {absent:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| prepare(0x5A5A_1234 ^ cars, mode, players, cars, absent),
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
