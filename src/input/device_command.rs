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
use crate::framework::rva;
use crate::input::devices::put;
use crate::input::store::DeviceSlot;
use crate::sound::store::EffectsSlot;
use crate::sound::store::ObjectsSlot;
use crate::store::{game_addr, game_read, game_write};

pub const DEVICES_RVA: DeviceSlot = DeviceSlot::at(0x4F6EE8); // g_inputDevices: the two records
const EFFECTS_ON_RVA: EffectsSlot = EffectsSlot::at(0x518090); // dword_518090
const CONTROLS_RVA: [usize; 2] = [rva(0x75AE28), rva(0x75AF94)]; // dword_75AE28, dword_75AF94
const TWO_DEVICES_RVA: DeviceSlot = DeviceSlot::at(0x4F68FC); // dword_4F68FC
const ONE_PLAYER_RVA: ObjectsSlot = ObjectsSlot::at(0x514F78); // g_onePlayer

fn device(n: usize) -> u32 {
    unsafe { game_read(DEVICES_RVA + 4 * n) }
}

/// The command `command` (its low byte the command, the next the device; see the header).
pub unsafe fn device_command(command: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::input::devices::command_steps`.
    use rc2k_core::input::devices::{CommandStep::*, command_steps};
    unsafe {
        let steps = command_steps(
            command as u8,
            (command >> 8) as u8,
            game_read(TWO_DEVICES_RVA),
            game_read(ONE_PLAYER_RVA),
        )?;
        for step in steps {
            match step {
                EffectsOn(on) => game_write(EFFECTS_ON_RVA, on as u32),
                Release(n) => crate::input::release_device::release_device(device(n))?,
                Acquire(n) => crate::input::acquire_device::acquire_device(device(n))?,
                EndEffects => crate::input::end_effects::end_effects()?,
                Tick => crate::input::input_tick::input_tick()?,
                SetUp(n) => crate::input::set_up_device::set_up_device(
                    device(n),
                    game_addr(CONTROLS_RVA[n]) as u32,
                )?,
                Reading(n, on) => put(device(n), 0x414, on as u32),
                SecondApart => {
                    if game_read::<u32>(CONTROLS_RVA[0]) == game_read::<u32>(CONTROLS_RVA[1]) {
                        game_write(CONTROLS_RVA[1], 0u32);
                    }
                }
                Sleep(ms) => crate::system::sleep::sleep(ms),
                BeginEffects => crate::input::begin_effects::begin_effects()?,
            }
        }
    }
    Ok(())
}
