//! `input::device_command` (0x408AE5): the input thread's command:
//! - 1: the device given back (`input::release_device`) and acquired again (`input::acquire_device`);
//! - 2 (the race's end): the force feedback's race switch (`force_feedback.race_on`) 0, the effects ended (`input::end_effects`), a
//!   tick (`input::input_tick`), both devices given back and +0x414 0 (not read);
//! - 3 (the race's start): device 0 set up from player 1's controls
//!   (`input::set_up_device`) and read (+0x414 1); unless one player on one device
//!   (`two_devices_one_player` 0 and `one_player` 1), device 1 from player 2's (its
//!   controller first made none when it is player 1's), likewise; a second's sleep, the
//!   effects begun (`input::begin_effects`), a tick, `race_on` 1;
//! - others: nothing.
//!
//! Caller: `input::input_tick` (when `g_inputCommand` was set). Refuses (before
//! anything) a device past 1 for command 1.
//!
//! Its arguments are what it reads.

use crate::errors::Refused;
use crate::input::owner::Input;
use crate::input::store::Devices;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Device `n`'s record: a command names a device the table holds.
fn device(
    devices: &Devices,
    n: usize,
) -> Result<&crate::input::types::InputDevice, crate::errors::BadInput> {
    devices.device(n).ok_or(crate::errors::BadInput(
        "a device command for a device without a record",
    ))
}

/// The command `command` (its low byte the command, the next the device; see the header).
pub fn device_command(
    input: &Input,
    command: u32,
    sound: &crate::sound::owner::Sound,
    system: &System,
    race: &Race,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::input::devices::command_steps`.
    use rc2k_core::input::devices::{CommandStep::*, command_steps};
    {
        let steps = command_steps(
            command as u8,
            (command >> 8) as u8,
            input.devices.two_devices_one_player.get(),
            sound.race.sounds.one_player.get(),
        )?;
        for step in steps {
            match step {
                EffectsOn(on) => input.force_feedback.race_on.set(on as u32),
                Release(n) => crate::input::release_device::release_device(
                    &input.devices,
                    device(&input.devices, n)?,
                )?,
                Acquire(n) => {
                    let record = device(&input.devices, n)?;
                    crate::input::acquire_device::acquire_device(input, system, record)?;
                }
                EndEffects => crate::input::end_effects::end_effects(input)?,
                Tick => crate::input::input_tick::input_tick(input, sound, system, race)?,
                SetUp(n) => crate::input::set_up_device::set_up_device(
                    input,
                    system,
                    device(&input.devices, n)?,
                    &race.shared.state.settings.cars[n],
                )?,
                Reading(n, on) => device(&input.devices, n)?.reading.set(on as u32),
                SecondApart => {
                    if race.shared.state.settings.cars[0].controller.get()
                        == race.shared.state.settings.cars[1].controller.get()
                    {
                        race.shared.state.settings.cars[1].controller.set(0);
                    }
                }
                Sleep(ms) => crate::system::sleep::sleep(ms),
                BeginEffects => crate::input::begin_effects::begin_effects(input, sound)?,
            }
        }
    }
    Ok(())
}
