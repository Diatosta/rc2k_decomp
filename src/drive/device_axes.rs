//! `drive::device_axes` (0x409430): an input device's four readings
//! (`g_inputDevices[esi]`, +0x4DC .. +0x4E8: into ecx, edx, eax, ebx, as
//! `input_device_read` 0x40930D, inlined), with `dword_4F6900` set the first (eax)
//! squared / 256 with its sign kept.
//!
//! In:  esi the device
//! Out: eax, ebx, ecx, edx the readings; esi kept; no flags read
//!
//! Caller: `drive::car_input_words`. Refuses a device past 7 or a null one.

use crate::errors::{BadInput, Refused};
use crate::input::store::DEVICES;
use rc2k_core::drive::controls::device_curve;

/// The Rust version: [eax, ebx, ecx, edx].
// Converted: `rc2k_core::drive::controls::device_curve`.
pub unsafe fn device_axes(device: u32) -> Result<[u32; 4], Refused> {
    if device > 7 {
        return Err(BadInput("a device past 7").into());
    }
    let d = DEVICES.input_device(device as usize);
    if d == 0 {
        return Err(BadInput("a null input device").into());
    }
    let rd = |o: u32| unsafe { ((d + o) as *const u32).read_unaligned() };
    let (ecx, edx, mut eax, ebx) = (rd(0x4DC), rd(0x4E0), rd(0x4E4), rd(0x4E8));
    if DEVICES.nonlinear_steering.get() != 0 {
        eax = device_curve(eax);
    }
    Ok([eax, ebx, ecx, edx])
}
