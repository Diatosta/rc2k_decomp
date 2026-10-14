//! `menu::next_car_number` (0x470D7F): the player's car number up by one.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `race_number` + 1, back to 1 above 90 (signed). An action of the four players' details
//! pages (menu tables 0x74EB48 ..). Compared in shadow mode.
//! Self-test: `menu::player_settings::check_setting`, 7 values for each player.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_car_number", rva(0x470D7F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn field(player: &mut Player) -> &mut u32 {
    &mut player.race_number
}

// ---- Rust version ----

pub fn next_car_number() -> Result<(), Refused> {
    change(field, |value| {
        let next = value.wrapping_add(1);
        if next as i32 > 90 { 1 } else { next }
    })
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_car_number()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = current().map_or(Vec::new(), |player| {
        vec![(player as u32, std::mem::size_of::<Player>())]
    });
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_setting(
        &mut t,
        call_original,
        field,
        &[0, 1, 89, 90, 91, 0x7FFF_FFFF, 0xFFFF_FFFF],
        &rust,
    );
    t.finish()
}
