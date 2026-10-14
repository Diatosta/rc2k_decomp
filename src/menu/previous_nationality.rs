//! `menu::previous_nationality` (0x467B21): the player's nationality before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `nationality` - 1, 0x27 from 0. An action of the four players' details pages (menu tables 0x74EA34 ..).
//! Compared in shadow mode.
//! Self-test: `menu::player_settings::check_setting`, 5 values for each player.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::previous_nationality",
    rva(0x467B21),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn field(player: &mut Player) -> &mut u32 {
    &mut player.nationality
}

// ---- Rust version ----

pub fn previous_nationality() -> Result<(), Refused> {
    change(field, |value| wrap_down(value, 0x27))
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    previous_nationality()?;
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
        &[0, 1, 0x26, 0x27, 0x28],
        &rust,
    );
    t.finish()
}
