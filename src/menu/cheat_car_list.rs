//! `menu::cheat_car_list` (0x480D35): with cheat bit 1 of `g_cheats` set, esi becomes the
//! cheat's car list `off_75A412`; else esi stays.
//!
//! In:  esi a car list
//! Out: esi; carry = the bit (`bt`); the rest kept
//!
//! Callers: the time trial's events, before each car choice. Self-test: each bit value,
//! `g_cheats` put back.

use crate::errors::Refused;
use crate::framework::{
    ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::cheat_car_list", rva(0x480D35), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const CHEAT_CARS_RVA: usize = rva(0x75A412); // off_75A412

/// The Rust version: esi and the carry in `r`.
pub fn cheat_car_list(r: &mut Regs) {
    // Converted: `rc2k_core::menu::car_lists::cheat_cars`.
    let on = rc2k_core::menu::car_lists::cheat_cars(unsafe { game_read(CHEATS_RVA) });
    if on {
        r.esi = game_addr(CHEAT_CARS_RVA) as u32;
    }
    r.set_carry(on);
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    cheat_car_list(&mut g);
    Ok(Left::carry(g.carry()).with(ESI, g.esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
    let vars = [at(CHEATS_RVA)];
    let saved = capture(&vars);
    for cheats in [0u32, 2, 0xFFFF_FFFD, 0xFFFF_FFFF, 1] {
        check_state(
            &mut t,
            &format!("g_cheats {cheats:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(CHEATS_RVA, cheats) },
            &[],
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
