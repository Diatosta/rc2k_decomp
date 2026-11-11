//! `input::poll_device` (0x408869): an acquired device's state copied into its record.
//! With its DirectInput device (+4) acquired (+0x24 1): `Poll`; `GetDeviceState` (0x50
//! bytes into `g_deviceState`); when that fails `Acquire` and (sic) the record's first 0x50
//! bytes copied over the state (a copy where zeros were meant, which also leaves the copy's pointer
//! 0x50 further on, so that the rest reads and writes the record from there: section 16).
//! Then the
//! axes: for kinds up to 2, x the state dword `+0x1D & 7` (negated with bit 0x10) and y
//! `+0x1C & 7` (likewise), a y below 0 being the third axis (made positive) and the second
//! 0; for kinds above 2, x the state's first dword and the others by `+0x1A` / `+0x1B` (each
//! plus 0x100 with bit 0x10); the second and third made positive; into +0x2C, +0x30, +0x34;
//! the 32 buttons (state +0x30 .., bit 7) as bits into +0x38.
//!
//! Takes: the device's record (`input/devices.rs`)
//!
//! Caller: `input::read_device`. Its DirectInput calls: `Poll`, `GetDeviceState`
//! (its result), `Acquire` (platform).
//!
//! Its arguments are what it reads.

use crate::errors::{BadInput, Refused};
use crate::input::devices::{STATE_LEN, joy_state};
use crate::input::store::Devices;
use crate::input::types::InputDevice;
use crate::pointers::GamePtr;
use rc2k_core::input::joystick::{SELECT_SECOND, SELECT_THIRD, SELECT_X, SELECT_Y};

fn state(devices: &Devices, n: u32) -> u32 {
    devices.device_state[n as usize].get()
}

/// The record's byte at one of the four places of the axis codes (+0x1A .. +0x1D).
fn selector(record: &InputDevice, place: u32) -> u8 {
    match place {
        SELECT_SECOND => record.axis_code_byte(2),
        SELECT_THIRD => record.axis_code_byte(3),
        SELECT_Y => record.axis_pick_byte(0),
        SELECT_X => record.axis_pick_byte(1),
        _ => unreachable!("the core asks for the four places of the axis codes"),
    }
}

/// The device `record` polled (see the header).
pub fn poll_device(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    let device = record.device.get();
    if record.acquired.get() != 1 || device == 0 {
        return Ok(());
    }
    crate::input::di_objects::poll(device);
    let failed = match crate::input::di_objects::get_device_state(device, STATE_LEN as u32) {
        Ok(state) => {
            for (cell, v) in devices.device_state.iter().zip(state.dwords()) {
                cell.set(v);
            }
            false
        }
        Err(_) => true,
    };
    // The copy of a failed read moves its pointer on by 0x50 as well: what follows reads
    // and writes the record from there (sic) to the function's end.
    if failed {
        crate::input::di_objects::acquire(device);
        GamePtr::to(record).copy_to(
            GamePtr::to(&devices.device_state),
            STATE_LEN,
            "a device record read as its state",
        )?;
        return poll_shifted(devices, record);
    }
    // Converted: `rc2k_core::input::joystick` (`poll_axes`, `poll_buttons`).
    use rc2k_core::input::joystick::{poll_axes, poll_buttons};
    let [x, y, z] = poll_axes(
        record.kind.get(),
        |place| selector(record, place),
        |n| state(devices, n),
    );
    record.axes[0].set(x);
    record.axes[1].set(y);
    record.axes[2].set(z);
    record.buttons.set(poll_buttons(
        &joy_state(devices)[0x30..].try_into().unwrap(),
    ));
    Ok(())
}

/// The rest of the poll after a failed read, on the record 0x50 bytes on (sic: the
/// effects' fields stand where the axes were meant).
fn poll_shifted(devices: &Devices, record: &InputDevice) -> Result<(), Refused> {
    use rc2k_core::input::joystick::{poll_axes, poll_buttons};
    // A run-on past the record, as the original's pointer 0x50 on.
    let at = |offset: u32| GamePtr::to(record).offset(STATE_LEN as u32 + offset);
    let kind = at(8).read::<u32>("a device record's dword")?;
    let late = crate::input::devices::Deferred::default();
    let [x, y, z] = poll_axes(
        kind,
        |place| {
            late.get(
                at(place)
                    .read::<u8>("a device record's byte")
                    .map(u32::from),
            ) as u8
        },
        |n| state(devices, n),
    );
    late.finish()?;
    let put = |offset: u32, v: u32| -> Result<(), BadInput> {
        at(offset).write(v, "a device record's dword")
    };
    put(0x2C, x)?;
    put(0x30, y)?;
    put(0x34, z)?;
    put(
        0x38,
        poll_buttons(&joy_state(devices)[0x30..].try_into().unwrap()),
    )?;
    Ok(())
}
