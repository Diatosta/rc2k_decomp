//! `input::poll_device` (0x408869): an acquired device's state copied into its record.
//! With its DirectInput device (+4) acquired (+0x24 1): `Poll`; `GetDeviceState` (0x50
//! bytes into `g_deviceState`); when that fails `Acquire` and (sic) the record's first 0x50
//! bytes copied over the state (`rep movsd` where zeros were meant, which also leaves esi
//! 0x50 further on, so that the rest reads and writes the record from there: section 16).
//! Then the
//! axes: for kinds up to 2, x the state dword `+0x1D & 7` (negated with bit 0x10) and y
//! `+0x1C & 7` (likewise), a y below 0 being the third axis (made positive) and the second
//! 0; for kinds above 2, x the state's first dword and the others by `+0x1A` / `+0x1B` (each
//! plus 0x100 with bit 0x10); the second and third made positive; into +0x2C, +0x30, +0x34;
//! the 32 buttons (state +0x30 .., bit 7) as bits into +0x38.
//!
//! In:  esi = the device's record (`input/devices.rs`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::read_device`. Its DirectInput calls: `Poll`, `GetDeviceState`
//! (eax its result), `Acquire` (platform).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::field;
use crate::input::devices::{Deferred, STATE_LEN, at, joy_state, put};
use crate::input::store::DEVICES;
use crate::input::types::InputDevice;
use crate::pointers::GamePtr;

fn state(n: u32) -> u32 {
    DEVICES.device_state[n as usize].get()
}

/// The device `record` polled (see the header).
pub fn poll_device(record: u32) -> Result<(), Refused> {
    let device = at(record, field!(InputDevice, device))?;
    if at(record, field!(InputDevice, acquired))? != 1 || device == 0 {
        return Ok(());
    }
    crate::input::di_objects::poll(device);
    let failed = {
        crate::input::di_objects::get_device_state(
            device,
            STATE_LEN as u32,
            DEVICES.device_state[0].addr(),
        ) as u32
    };
    // The `rep movsd` of a failed read moves esi on by 0x50 as well: what follows reads
    // and writes the record from there (sic) until the `popa`.
    let mut record = record;
    if failed != 0 {
        crate::input::di_objects::acquire(device);
        GamePtr::new(record).copy_to(
            GamePtr::new(DEVICES.device_state[0].addr()),
            STATE_LEN,
            "a device record read as its state",
        )?;
        record += STATE_LEN as u32;
    }
    // Converted: `rc2k_core::input::joystick` (`poll_axes`, `poll_buttons`).
    use rc2k_core::input::joystick::{poll_axes, poll_buttons};
    let late = Deferred::default();
    let [x, y, z] = poll_axes(
        at(record, field!(InputDevice, kind))?,
        |place| {
            late.get(
                GamePtr::new(record + place)
                    .read::<u8>("a device record's byte")
                    .map(u32::from),
            ) as u8
        },
        state,
    );
    late.finish()?;
    put(record, field!(InputDevice, axes), x)?;
    put(record, field!(InputDevice, axes) + 4, y)?;
    put(record, field!(InputDevice, axes) + 8, z)?;
    let buttons = poll_buttons(&joy_state()[0x30..].try_into().unwrap());
    put(record, field!(InputDevice, buttons), buttons)?;
    Ok(())
}
