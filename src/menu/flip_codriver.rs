//! `menu::flip_codriver` (0x467BD6): the player's co-driver's sex the other.
//!
//! In:  nothing
//! Out: edi = the player; everything else preserved
//!
//! `codriver_sex` xor 1. No `pusha`: edi is left the player. An action of the four players' details pages (menu tables 0x74EBA4 ..).
//! Compared in shadow mode.
//! Self-test: `menu::player_settings::check_setting`, 4 values for each player.

use crate::errors::Refused;
use crate::framework::{EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player::Player;
use crate::menu::player_settings::*;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::flip_codriver", rva(0x467BD6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn field(player: &mut Player) -> &mut u32 {
    &mut player.codriver_sex
}

// ---- Rust version ----

pub fn flip_codriver() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::player::stepped`.
    change(field, |value| {
        rc2k_core::menu::player::stepped(rc2k_core::menu::player::Setting::CodriverSex, value, true)
    })
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    flip_codriver()?;
    Ok(Left::nothing().with(EDI, current()? as u32))
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
    check_setting(&mut t, call_original, field, &[0, 1, 2, 0xFFFF_FFFF], &rust);
    t.finish()
}
