//! `menu::next_view` (0x467B45): the player's next view.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `view` + 1, back to 0 after 6. An action of the four players' details pages (menu tables 0x74EA90 ..).
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

pub static HOOK: RawHook =
    RawHook::new("menu::next_view", rva(0x467B45), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn field(player: &mut Player) -> &mut u32 {
    &mut player.view
}

// ---- Rust version ----

pub fn next_view() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::View, value, true)
    })
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    next_view()?;
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
    check_setting(&mut t, call_original, field, &[0, 1, 5, 6, 7], &rust);
    t.finish()
}
