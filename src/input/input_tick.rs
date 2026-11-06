//! `input::input_tick` (0x408A56): one tick of the input thread's devices: the command
//! `g_inputCommand` taken (`xchg` with 0); when there is one, `input::device_command`;
//! else both devices (`unk_4F6EF0`, `unk_4F73E8`) read (`input::read_device`) and their
//! effects told (`input::device_effects`), and with one player on two devices
//! (`dword_4F68FC`) the second's readings added to the first's (its bits: those of 0x70FF).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the input thread (`sub_4078DA`, every 20 ms), `input::device_command`
//! (twice).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::devices::records;
use crate::input::types::InputDevice;
use crate::session::Dword;

/// A tick of the devices (see the header).
pub fn input_tick() -> Result<(), Refused> {
    let command = crate::input::owner::input()
        .devices
        .input_command
        .atomic()
        .swap(0, std::sync::atomic::Ordering::SeqCst);
    if command != 0 {
        return crate::input::device_command::device_command(command);
    }
    let records = records();
    for record in records {
        crate::input::read_device::read_device(record)?;
        crate::input::device_effects::device_effects(record)?;
    }
    if crate::input::owner::input()
        .devices
        .two_devices_one_player
        .get()
        != 0
    {
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
pub fn input_tick_in_game() -> Result<(), Refused> {
    input_tick()
}
