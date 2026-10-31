//! `input::axis_found` (0x4083F4): DirectInput's `EnumObjects` callback for a joystick
//! being listed: the object's type GUID (`DIDEVICEOBJECTINSTANCEA` +4) named among the six
//! axes' (`GUID_XAxis`, `GUID_YAxis`, `GUID_ZAxis`, `GUID_RxAxis`, `GUID_RyAxis`,
//! `GUID_RzAxis`, at `unk_4F7E38` ..) sets the bit (1, 2, 4, 8, 0x10, 0x20) in the
//! controller's entry's +0x14 (kept: the bits are added to it). Always goes on (eax 1).
//!
//! In:  (stdcall) the `DIDEVICEOBJECTINSTANCEA`, the context (the controller's entry)
//! Out: eax = 1; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::joystick_found`.

use crate::errors::BadInput;
use crate::field;
use crate::input::devices::{at, put};
use crate::input::guids::INPUT_GUIDS;
use crate::input::types::{Controller, DiDeviceObjectInstance};

/// `axis_found` as DirectInput calls it: what Rust callers hand `EnumObjects`.
pub fn callback(object: u32, entry: u32) -> u32 {
    match axis_found(object, entry) {
        Ok(eax) => eax,
        Err(e) => {
            crate::input::devices::refused_here("input::axis_found", e);
            0
        }
    }
}

/// The Rust version: eax.
pub fn axis_found(object: u32, entry: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::input::joystick::axis_bits`.
    let object_type: [u8; 16] = crate::pointers::GamePtr::new(
        object.wrapping_add(field!(DiDeviceObjectInstance, guid_type)),
    )
    .read("an axis object's type")?;
    let mut axes = [[0u8; 16]; 6];
    for (n, axis) in axes.iter_mut().enumerate() {
        axis.copy_from_slice(&INPUT_GUIDS.axes.read_at(16 * n, 16, "an axis GUID")?);
    }
    let bits = rc2k_core::input::joystick::axis_bits(
        at(entry, field!(Controller, axis_bits))?,
        &object_type,
        &axes,
    );
    put(entry, field!(Controller, axis_bits), bits)?;
    Ok(1)
}
