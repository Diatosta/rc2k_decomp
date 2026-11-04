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
use crate::input::store::DEVICES;
use crate::input::store::device_of;
use crate::race::view_store::RACE_VIEW;
use crate::session::Dword;
use crate::sound::store::RACE_SOUNDS;

/// The variables it reads (the game's or a test's).
#[derive(Clone, Copy)]
pub struct Vars {
    pub cars: &'static [Dword; 2],
    pub one_player: &'static Dword,
    pub count: &'static Dword,
    pub devices: &'static [Dword; 2],
    pub views: &'static [Dword; 2],
    pub objects: &'static [crate::sound::store::SoundObject; 8],
    pub two_devices: &'static Dword,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            // g_playerCarA, g_playerCarB, g_onePlayer, g_raceSoundObjectCount.
            cars: &RACE_SOUNDS.player_cars,
            one_player: &RACE_SOUNDS.one_player,
            count: &RACE_SOUNDS.count,
            devices: &DEVICES.input_devices,
            views: &RACE_VIEW.view_blocks,
            objects: &RACE_SOUNDS.objects,
            two_devices: &DEVICES.two_devices_one_player,
        }
    }
}

/// The Rust version on the variables in `v`.
/// Converted: `rc2k_core::input::effects::loads` (the way worked out before any call:
/// nothing a call changes decides it).
pub fn update_effects_with(v: &Vars) -> Result<(), Refused> {
    use rc2k_core::input::effects::{Players, loads};
    if v.count.get() == 0 {
        return Ok(());
    }
    let p = Players {
        cars: v.cars.each_ref().map(super::super::session::Dword::get),
        one_player: v.one_player.get() == 1,
        // the devices by handle: the core only asks whether one is none (0)
        devices: v.devices.each_ref().map(crate::session::Dword::get),
        views: v
            .views
            .each_ref()
            .map(|c| crate::race::blocks_store::block_of(c.get())),
        one_players_devices: v.two_devices.get() != 0,
    };
    let loaded = loads(&p, |device| {
        device_of(device).is_some_and(|record| record.effects_on.get() == 1)
    })?;
    for load in loaded {
        if load.update {
            let object = v
                .objects
                .get(load.car as usize)
                .ok_or(BadInput("a car past the sound objects' table"))?;
            let device = device_of(load.device).ok_or(BadInput("a load on a null device"))?;
            crate::input::update_car_effects::update_car_effects(load.view, object, device)?;
        }
    }
    Ok(())
}

/// The race's force feedback brought up to date.
pub fn update_effects() -> Result<(), Refused> {
    update_effects_with(&Vars::game())
}
