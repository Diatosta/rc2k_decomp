//! `input::input_tick` (0x408A56): one tick of the input thread's devices: the command
//! `g_inputCommand` taken (`xchg` with 0); when there is one, `input::device_command`;
//! else both devices (`records`) read (`input::read_device`) and their
//! effects told (`input::device_effects`), and with one player on two devices
//! (`two_devices_one_player`) the second's readings added to the first's (its bits: those of 0x70FF).
//!
//! Callers: the input thread (`input::input_thread`, every 20 ms), `input::device_command`
//! (twice).
//!
//! Its arguments are what it reads.

use crate::errors::Refused;
use crate::input::devices::records;
use crate::input::owner::Input;
use crate::input::types::InputDevice;
use crate::session::Dword;

/// A tick of the devices (see the header).
pub fn input_tick(input: &Input, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let command = input
        .devices
        .input_command
        .atomic()
        .swap(0, std::sync::atomic::Ordering::SeqCst);
    if command != 0 {
        return crate::input::device_command::device_command(input, command, sound);
    }
    let records = records(&input.devices);
    for record in records {
        crate::input::read_device::read_device(&input.devices, record)?;
        crate::input::device_effects::device_effects(&input.devices, record)?;
    }
    if input.devices.two_devices_one_player.is_set() {
        // Converted: `rc2k_core::input::devices::merge_readings`.
        let [first, second] = records;
        let read = |record: &InputDevice| -> [u32; 4] {
            let [steering, brake, throttle] = record.readings.each_ref().map(Dword::get);
            [steering, brake, throttle, record.bits.get()]
        };
        let mut a = read(first);
        let b = read(second);
        rc2k_core::input::devices::merge_readings(&mut a, &b);
        for (cell, v) in first.readings.iter().chain([&first.bits]).zip(a) {
            cell.set(v);
        }
    }
    Ok(())
}

/// For the standalone's input thread.
pub fn input_tick_in_game(
    input: &Input,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    input_tick(input, sound)
}
