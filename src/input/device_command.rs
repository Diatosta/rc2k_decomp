//! `input::device_command` (0x408AE5): the input thread's command (al):
//! - 1: device ah given back (`input::release_device`) and acquired again (`input::acquire_device`);
//! - 2 (the race's end): `dword_518090` 0, the effects ended (`input::end_effects`), a
//!   tick (`input::input_tick`), both devices given back and +0x414 0 (not read);
//! - 3 (the race's start): device 0 set up from player 1's controls (`dword_75AE28`,
//!   `input::set_up_device`) and read (+0x414 1); unless one player on one device
//!   (`dword_4F68FC` 0 and `g_onePlayer` 1), device 1 from player 2's (`dword_75AF94`, its
//!   controller first made none when it is player 1's), likewise; a second's sleep, the
//!   effects begun (`input::begin_effects`), a tick, `dword_518090` 1;
//! - others: nothing.
//!
//! In:  al = the command, ah = the device (for 1)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_tick` (when `g_inputCommand` was set). Refuses (before
//! anything) a device past 1 for command 1.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::store::DEVICES;
use crate::race::state_store::RACE_STATE;
use crate::sound::store::RACE_SOUNDS;

/// Device `n`'s record: a command names a device the table holds.
fn device(n: usize) -> Result<&'static crate::input::types::InputDevice, crate::errors::BadInput> {
    DEVICES.device(n).ok_or(crate::errors::BadInput(
        "a device command for a device without a record",
    ))
}

/// The command `command` (its low byte the command, the next the device; see the header).
pub fn device_command(command: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::input::devices::command_steps`.
    use rc2k_core::input::devices::{CommandStep::*, command_steps};
    {
        let steps = command_steps(
            command as u8,
            (command >> 8) as u8,
            DEVICES.two_devices_one_player.get(),
            RACE_SOUNDS.one_player.get(),
        )?;
        for step in steps {
            match step {
                EffectsOn(on) => crate::input::store::FORCE_FEEDBACK.race_on.set(on as u32),
                Release(n) => crate::input::release_device::release_device(device(n)?)?,
                Acquire(n) => crate::input::acquire_device::acquire_device(device(n)?)?,
                EndEffects => crate::input::end_effects::end_effects()?,
                Tick => crate::input::input_tick::input_tick()?,
                SetUp(n) => crate::input::set_up_device::set_up_device(
                    device(n)?,
                    crate::pointers::GamePtr::to(&RACE_STATE.settings.cars[n]).offset(
                        std::mem::offset_of!(crate::race::state_store::RaceCar, controller) as u32,
                    ),
                )?,
                Reading(n, on) => device(n)?.reading.set(on as u32),
                SecondApart => {
                    if RACE_STATE.settings.cars[0].controller.get()
                        == RACE_STATE.settings.cars[1].controller.get()
                    {
                        RACE_STATE.settings.cars[1].controller.set(0);
                    }
                }
                Sleep(ms) => crate::system::sleep::sleep(ms),
                BeginEffects => crate::input::begin_effects::begin_effects()?,
            }
        }
    }
    Ok(())
}
