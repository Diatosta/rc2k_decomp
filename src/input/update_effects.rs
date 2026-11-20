//! The race's force feedback brought up to date, once each 40 ms (`input::step_effects_clock`):
//! with sound objects in use, `input::update_car_effects` for player A's car on the first device
//! (with the first view block) if that device has effects; then, if the two devices are one
//! player's (`two_devices_one_player` not 0), for car A again on the second device. Otherwise
//! (no effects on the first device, or one device a player), unless one player is playing, for
//! player B's car on the second device with the second view block. Each only if that device has
//! effects (+0x28 1). See `input/effects.rs`. Refuses (before anything: the way through is
//! worked out first) a car number past the eight sound objects and a null device on the way
//! taken. The third and fourth players of a larger split screen then get theirs on devices 2 and 3.

use crate::errors::{BadInput, Refused};
use crate::input::owner::Input;
use crate::input::store::device_of;
use crate::memory::session::Dword;
use crate::race::owner::Race;
use crate::sound::owner::Sound;
use crate::system::owner::System;

/// The variables it reads (the game's or a test's).
#[derive(Clone, Copy)]
pub struct Vars<'a> {
    pub cars: &'a [crate::memory::session::PlainDword; 2],
    pub one_player: &'a Dword,
    pub count: &'a Dword,
    pub devices: &'a [Dword; 4],
    pub views: &'a [crate::memory::session::PlainDword; 2],
    pub objects: &'a [crate::sound::store::SoundObject; 8],
    pub two_devices: &'a Dword,
    /// The race's players: the third and fourth (a larger split screen) have their cars' blocks, sound
    /// objects and devices 2 and 3.
    pub players: u32,
    /// The car blocks the view blocks name.
    pub car_blocks: &'a crate::race::blocks_store::CarBlocks,
}

impl<'a> Vars<'a> {
    pub fn game(input: &'a Input, sound: &'a Sound, race: &'a Race) -> Self {
        Self {
            // Player A's and B's cars, the one-player flag and the sound object count.
            cars: &sound.race.sounds.get().player_cars,
            one_player: &sound.threads.shared.one_player,
            count: &sound.threads.shared.count,
            devices: &input.devices.input_devices,
            views: &race.shared.view.get().view_blocks,
            objects: &sound.race.sounds.get().objects,
            two_devices: &input.devices.two_devices_one_player,
            players: race.shared.view.get().players.get(),
            car_blocks: race.car_blocks(),
        }
    }
}

/// On the variables in `v`. The logic is in `rc2k_core::input::effects::loads` (the way worked out
/// before any call: nothing a call changes decides it).
pub fn update_effects_with(input: &Input, system: &System, v: &Vars<'_>) -> Result<(), Refused> {
    use rc2k_core::input::effects::{Players, loads};
    if v.count.get() == 0 {
        return Ok(());
    }
    let p = Players {
        cars: v
            .cars
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
        one_player: v.one_player.get() == 1,
        // the devices by handle: the core only asks whether one is none (0)
        // (the force feedback is for the first two players' devices)
        devices: [v.devices[0].get(), v.devices[1].get()],
        // The view blocks by handle (the core only passes them on).
        views: v
            .views
            .each_ref()
            .map(crate::memory::session::PlainDword::get),
        one_players_devices: v.two_devices.is_set(),
    };
    let loaded = loads(&p, |device| {
        device_of(&input.devices, device).is_some_and(|record| record.effects_on.get() == 1)
    })?;
    for load in loaded {
        if load.update {
            let object = v
                .objects
                .get(load.car as usize)
                .ok_or(BadInput("a car past the sound objects' table"))?;
            let device = device_of(&input.devices, load.device)
                .ok_or(BadInput("a load on a null device"))?;
            let view = v
                .car_blocks
                .of(load.view)
                .ok_or(BadInput("a load on no view block"))?;
            // The player's switch is the profile of the race device (the slot the handle is in).
            let engine = v
                .devices
                .iter()
                .position(|d| d.get() == load.device)
                .is_none_or(|slot| crate::input::profiles::race_engine_vibration(input, slot));
            crate::input::update_car_effects::update_car_effects(
                &input.force_feedback,
                view,
                object,
                device,
                engine,
                system,
            )?;
        }
    }
    // The third and fourth players' cars, each on its own device if that has effects.
    for n in 2..v.players.min(4) as usize {
        let Some(device) = device_of(&input.devices, v.devices[n].get()) else {
            continue;
        };
        if device.effects_on.get() != 1 {
            continue;
        }
        let object = v
            .objects
            .get(n)
            .ok_or(BadInput("a car past the sound objects' table"))?;
        crate::input::update_car_effects::update_car_effects(
            &input.force_feedback,
            &v.car_blocks.cars[n],
            object,
            device,
            crate::input::profiles::race_engine_vibration(input, n),
            system,
        )?;
    }
    Ok(())
}

/// The race's force feedback brought up to date.
pub fn update_effects(
    input: &Input,
    sound: &Sound,
    race: &Race,
    system: &System,
) -> Result<(), Refused> {
    update_effects_with(input, system, &Vars::game(input, sound, race))
}
