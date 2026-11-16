//! A controller's state and axes as the front end reads them:
//! the first device holds +0x2C .. +0x38 four values the
//! poll left (x axis, y axis, z axis, the buttons' bits); the six axes at
//! `device_state` and as they were at `axes_rest`; the device's record (`records`)
//! +0x14 the axes it has.

use crate::errors::BadInput;
use crate::input::types::InputDevice;

/// The device's four state values:
/// (+0x2C, +0x30, +0x34, +0x38).
pub fn device_state(device: &InputDevice) -> [u32; 4] {
    [
        device.axes[0].get(),
        device.axes[1].get(),
        device.axes[2].get(),
        device.buttons.get(),
    ]
}

/// `device_state` of the first input device.
pub fn first_device_state(input: &crate::input::owner::Input) -> Result<[u32; 4], BadInput> {
    let Some(device) = input.devices.device(0) else {
        return Err(BadInput("no first input device record"));
    };
    Ok(device_state(device))
}

/// What `input::axis_moved` leaves: the carry (none) and the value.
pub struct Moved {
    pub carry: bool,
    pub eax: u32,
}

/// With a first argument of 0, after a sleep of 250 ms the axes' rest values are taken
/// (`Devices.device_state[0]` -> `Keyboard.axes_rest[0]`), `carry` set. Else the number of the
/// first of the device's axes that moved more than 0x80 from rest (+0x10 when it went up);
/// `carry` when none. A controller number past the table's 16 is refused.
pub fn axis_moved(eax: u32, input: &crate::input::owner::Input) -> Result<Moved, BadInput> {
    let now: [u32; 6] = std::array::from_fn(|n| input.devices.device_state[n].get());
    if eax == 0 {
        crate::system::sleep::sleep(250);
        for (rest, now) in input.keyboard().axes_rest.iter().zip(now) {
            rest.set(now);
        }
        return Ok(Moved { carry: true, eax });
    }
    let axes = input
        .devices
        .controllers
        .get(input.devices.records[0].controller.get() as usize)
        .ok_or(BadInput("a controller's axes past the table"))?
        .axis_bits
        .get();
    let rest: [u32; 6] = std::array::from_fn(|n| input.keyboard().axes_rest[n].get());
    Ok(
        match rc2k_core::input::devices::axis_moved(&now, &rest, axes) {
            Ok(axis) => Moved {
                carry: false,
                eax: axis,
            },
            Err(last) => Moved {
                carry: true,
                eax: last,
            },
        },
    )
}
