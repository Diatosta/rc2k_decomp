//! An input device's four readings, with `Devices.nonlinear_steering` set the
//! first squared / 256 with its sign kept.
//!
//! Refuses a device past 7 or a null one.

use crate::errors::{BadInput, Refused};
use rc2k_core::drive::controls::device_curve;

pub fn device_axes(device: u32, input: &crate::input::owner::Input) -> Result<[u32; 4], Refused> {
    if device > 7 {
        return Err(BadInput("a device past 7").into());
    }
    let Some(d) = input.devices.input_device(device as usize)? else {
        return Err(BadInput("a null input device").into());
    };
    let [first_reading, second_reading, mut steering] = d
        .readings
        .each_ref()
        .map(crate::memory::session::Dword::get);
    let bits = d.bits.get();
    if input.devices.nonlinear_steering.get() != 0 {
        steering = device_curve(steering);
    }
    Ok([steering, bits, first_reading, second_reading])
}
