//! `race::set_up_mode_4_cars` (0x43647F): `sub_43603C`'s set-up for `dword_62389C` 4:
//! `g_raceCars`, `dword_63DB94`, `dword_63DB98` 1; `race::reset_car_blocks`,
//! `drop_absent_player`, `set_player_car_kinds`.
//!
//! In:  nothing
//! Out: eax = `dword_601430` x 100, ecx = 8, esi = the last block (as `reset_car_blocks`)
//!
//! See `race/race_cars.rs`. One caller, `sub_43603C`. Shadow mode: the Rust version (its steps the
//! Rust versions), the blocks put back, the original's, compared. Self-test: a copy whose
//! calls note themselves, on the game's blocks (filled, put back), against the Rust version
//! on the same.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::race_cars::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: &[usize] = &[0x1E, 0x23, 0x28];

pub static HOOK: RawHook =
    RawHook::new("race::set_up_mode_4_cars", rva(0x43647F), thunk, &ORIGINAL).self_test(self_test);

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
    r.push(at(rva(0x70F474)));
    r.push(at(rva(0x710454)));
    r.extend([
        at(crate::race::objects::CARS_RVA),
        at(PLAYERS_RVA),
        at(PLAYERS_2_RVA),
    ]);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_up_mode_4_cars(r, &mut cars_part) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x43647F,
        0x2E,
        &[0xC7, 0x05, 0x90, 0xDB, 0x63, 0x00, 0x01, 0x00, 0x00, 0x00],
        CALLS,
        CAR_CALLEES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = set_up_regions();
    let saved = capture(&regions);
    let _ = game_addr(0);
    for (players, cars) in [(1u32, 1u32), (2, 2), (2, 8)] {
        run_cars(
            &mut t,
            &format!("{players} players, {cars} cars"),
            &copy,
            &regions,
            Regs::SENTINEL,
            &|| {
                prepare(0x3C3C_0000 ^ cars, 4, players, cars, [1, 0]);
                prepare_set_up(0x22, 0, [0, 0]);
            },
            &|r| unsafe { set_up_mode_4_cars(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
