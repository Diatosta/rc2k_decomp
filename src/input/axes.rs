//! A controller's state as the front end reads them: the first device holds +0x2C .. +0x38
//! four values the poll left (x axis, y axis, z axis, the buttons' bits).

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
