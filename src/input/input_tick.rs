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

use crate::errors::{BadInput, Refused};
use crate::field;
use crate::input::devices::{at, put, record_addrs};
use crate::input::store::DEVICES;
use crate::input::types::InputDevice;

/// A tick of the devices (see the header).
pub fn input_tick() -> Result<(), Refused> {
    let command = DEVICES
        .input_command
        .atomic()
        .swap(0, std::sync::atomic::Ordering::SeqCst);
    if command != 0 {
        return crate::input::device_command::device_command(command);
    }
    let records = record_addrs();
    for record in records {
        crate::input::read_device::read_device(record)?;
        crate::input::device_effects::device_effects(record)?;
    }
    if DEVICES.two_devices_one_player.get() != 0 {
        // Converted: `rc2k_core::input::devices::merge_readings`.
        let [first, second] = records;
        let read = |record: u32| -> Result<[u32; 4], BadInput> {
            let mut out = [0; 4];
            for (v, o) in out
                .iter_mut()
                .zip([0x4DC, 0x4E0, 0x4E4, field!(InputDevice, bits)])
            {
                *v = at(record, o)?;
            }
            Ok(out)
        };
        let mut a = read(first)?;
        let b = read(second)?;
        rc2k_core::input::devices::merge_readings(&mut a, &b);
        for (o, v) in [0x4DC, 0x4E0, 0x4E4, 0x4E8].into_iter().zip(a) {
            put(first, o, v)?;
        }
    }
    Ok(())
}

/// For the standalone's input thread.
pub fn input_tick_in_game() -> Result<(), Refused> {
    input_tick()
}
