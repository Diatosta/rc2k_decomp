//! `menu::take_player1_units` (0x467EA8): the game's units from player 1's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `g_isImperialUnits` = player 1's `metric_units` (+0x178; despite the name the variable
//! holds 1 for metric). Called at the start of each kind of event (`sub_482382` and more).
//! Self-test: the game's player 1 with values of the test's, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::player::{Player, player_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::take_player1_units", rva(0x467EA8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const UNITS_RVA: usize = rva(0x51DE2C); // g_isImperialUnits

pub fn take_player1_units() -> Result<(), Refused> {
    let p = player_at(0);
    if p == 0 {
        return Err(BadInput("no player 1 in g_players").into());
    }
    unsafe { game_write(UNITS_RVA, (*(p as *const Player)).metric_units) };
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    take_player1_units()?;
    Ok(Left::nothing())
}

fn regions() -> [Region; 1] {
    [at(UNITS_RVA)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let p = player_at(0);
    if p == 0 {
        return t.skip("no player 1 in g_players");
    }
    let all = [regions()[0], (p, std::mem::size_of::<Player>())];
    let saved = capture(&all);
    for units in [0u32, 1, 0x1234_5678] {
        check_state(
            &mut t,
            &format!("units {units:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(UNITS_RVA, 0xAAAA_AAAAu32);
                (*(p as *mut Player)).metric_units = units;
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
