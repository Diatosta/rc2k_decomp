//! `input::update_effects` (0x40F2C0): the race's force feedback brought up to date, once
//! each 40 ms (`input::step_effects_clock`): with sound objects in use
//! (`g_raceSoundObjectCount`), `input::update_car_effects` for player A's car on the first
//! device (`g_inputDevices[0]`, the first view block) if that device has effects; then, if
//! the two devices are one player's (`dword_4F68FC` not 0), for car A again on the second
//! device. Otherwise (no effects on the first device, or one device a player), unless one
//! player (`g_onePlayer` 1), for player B's car on the second device with the second view
//! block. Each only if that device has effects (+0x28 1).
//!
//! In:  nothing
//! Out: eax, ebx, esi, edi as last loaded (the car's number, the view block, the device,
//!      the car's sound object), or as `update_car_effects` leaves them after a call;
//!      flags not read (the caller's `test eax, eax` is inside its `pusha`)
//!
//! See `input/effects.rs`. One caller, `input::step_effects_clock`. Refuses (before anything: the way
//! through is worked out
//! first, nothing it calls changes what decides it) a car number past the eight sound
//! objects and a null device on the way taken.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::input::devices::{Deferred, at};
use crate::input::effects::HAS_EFFECTS;
use crate::input::store::DEVICES;
use crate::pointers::GamePtr;
use crate::race::view_store::RACE_VIEW;
use crate::sound::store::RACE_SOUNDS;

/// Where the variables are (the game's or a test's).
#[derive(Clone, Copy)]
pub struct Vars {
    pub cars: u32,
    pub devices: u32,
    pub views: u32,
    pub objects: u32,
    pub two_devices: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            // g_playerCarA, g_playerCarB, g_onePlayer, g_raceSoundObjectCount (four dwords).
            cars: crate::pointers::game_address(RACE_SOUNDS.player_cars.as_ptr()),
            devices: DEVICES.input_devices[0].addr(),
            views: (RACE_VIEW.view_blocks[0].addr()),
            objects: crate::pointers::game_address(RACE_SOUNDS.table.as_ptr()),
            two_devices: DEVICES.two_devices_one_player.addr(),
        }
    }
}

fn rd(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read("a variable of the force feedback's update")
}

/// The Rust version on the variables in `v`.
/// Converted: `rc2k_core::input::effects::loads` (the way worked out before any call:
/// nothing a call changes decides it).
pub fn update_effects_with(v: &Vars) -> Result<(), Refused> {
    use rc2k_core::input::effects::{Players, loads};
    if rd(v.cars + 0xC)? == 0 {
        return Ok(());
    }
    let p = Players {
        cars: [rd(v.cars)?, rd(v.cars + 4)?],
        one_player: rd(v.cars + 8)? == 1,
        devices: [rd(v.devices)?, rd(v.devices + 4)?],
        views: [rd(v.views)?, rd(v.views + 4)?],
        one_players_devices: rd(v.two_devices)? != 0,
    };
    let late = Deferred::default();
    let loaded = loads(&p, |device| late.get(at(device, HAS_EFFECTS)) == 1)?;
    late.finish()?;
    for load in loaded {
        if load.update {
            let object = rd(v.objects + 4 * load.car)?;
            crate::input::update_car_effects::update_car_effects(load.view, object, load.device)?;
        }
    }
    Ok(())
}

/// The race's force feedback brought up to date.
pub fn update_effects() -> Result<(), Refused> {
    update_effects_with(&Vars::game())
}
