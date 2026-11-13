//! The input thread's command: 1 gives the device back and acquires it again; 2 (the race's
//! end) switches the race off, ends the effects, ticks, gives both devices back; 3 (the race's
//! start) sets up device 0 from player 1's controls and, unless one player uses one device,
//! device 1 from player 2's, sleeps a second, begins the effects, ticks and switches the race
//! on; others do nothing. Refuses (before anything) a device past 1 for command 1.

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
    use rc2k_core::input::devices::{CommandStep::*, command_steps};
    {
        let steps = command_steps(
            command as u8,
            (command >> 8) as u8,
            input.devices.two_devices_one_player.get(),
            sound.race.shared.one_player.get(),
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
                    &race.shared.state.settings.cars[n].controls,
                )?,
                Reading(n, on) => device(&input.devices, n)?.reading.set(on as u32),
                SecondApart => {
                    if race.shared.state.settings.cars[0].controls.controller.get()
                        == race.shared.state.settings.cars[1].controls.controller.get()
                    {
                        race.shared.state.settings.cars[1]
                            .controls
                            .controller
                            .set(0);
                    }
                }
                Sleep(ms) => crate::system::sleep::sleep(ms),
                BeginEffects => crate::input::begin_effects::begin_effects(input, sound)?,
            }
        }
    }
    Ok(())
}
