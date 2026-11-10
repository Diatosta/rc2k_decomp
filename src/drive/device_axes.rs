//! `drive::device_axes` (0x409430): an input device's four readings, with `dword_4F6900` set the
//! first squared / 256 with its sign kept.
//!
//! Takes: the device
//! Returns: the readings
//!
//! Caller: `drive::car_input_words`. Refuses a device past 7 or a null one.

use crate::errors::{BadInput, Refused};
use rc2k_core::drive::controls::device_curve;

/// [eax, ebx, ecx, edx].
// Converted: `rc2k_core::drive::controls::device_curve`.
pub fn device_axes(device: u32, input: &crate::input::owner::Input) -> Result<[u32; 4], Refused> {
    if device > 7 {
        return Err(BadInput("a device past 7").into());
    }
    let Some(d) = input.devices.input_device(device as usize)? else {
        return Err(BadInput("a null input device").into());
    };
    let [ecx, edx, mut eax] = d.readings.each_ref().map(crate::session::Dword::get);
    let ebx = d.bits.get();
    if input.devices.nonlinear_steering.get() != 0 {
        eax = device_curve(eax);
    }
    Ok([eax, ebx, ecx, edx])
}
