//! `input::axis_found` (0x4083F4): DirectInput's `EnumObjects` callback for a joystick
//! being listed: the object's type GUID (`DIDEVICEOBJECTINSTANCEA` +4) named among the six
//! axes' (`GUID_XAxis`, `GUID_YAxis`, `GUID_ZAxis`, `GUID_RxAxis`, `GUID_RyAxis`,
//! `GUID_RzAxis`, at `unk_4F7E38` ..) sets the bit (1, 2, 4, 8, 0x10, 0x20) in the
//! controller's entry's +0x14 (kept: the bits are added to it). Always goes on (eax 1).
//!
//! In:  (stdcall) the `DIDEVICEOBJECTINSTANCEA`, the context (the controller's entry)
//!
//! Caller: DirectInput, from `input::joystick_found`.

use crate::errors::BadInput;
use crate::input::guids::InputGuids;
use crate::input::types::{Controller, DiDeviceObjectInstance};

/// `axis_found` as DirectInput calls it: what Rust callers hand `EnumObjects`, the context
/// the controller's entry.
pub fn callback(guids: &InputGuids, object: &DiDeviceObjectInstance, entry: &Controller) -> u32 {
    match axis_found(guids, object, entry) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::axis_found", e);
            0
        }
    }
}

/// The Rust version: eax.
pub fn axis_found(
    guids: &InputGuids,
    object: &DiDeviceObjectInstance,
    entry: &Controller,
) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::input::joystick::axis_bits`.
    let mut axes = [[0u8; 16]; 6];
    for (n, axis) in axes.iter_mut().enumerate() {
        axis.copy_from_slice(&guids.axes.read_at(16 * n, 16, "an axis GUID")?);
    }
    let bits =
        rc2k_core::input::joystick::axis_bits(entry.axis_bits.get(), &object.guid_type, &axes);
    entry.axis_bits.set(bits);
    Ok(1)
}
