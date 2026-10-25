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

use crate::field;
use crate::input::devices::{at, put};
use crate::input::guids::INPUT_GUIDS;
use crate::input::types::{Controller, DiDeviceObjectInstance};

/// The Rust version: eax.
/// `axis_found` as DirectInput calls it: what Rust callers hand `EnumObjects`.
pub unsafe extern "system" fn callback(object: u32, entry: u32) -> u32 {
    axis_found(object, entry)
}

pub fn axis_found(object: u32, entry: u32) -> u32 {
    // Converted: `rc2k_core::input::joystick::axis_bits`.
    let guid = |at: u32| unsafe { (at as *const [u8; 16]).read_unaligned() };
    let object_type = guid(object.wrapping_add(field!(DiDeviceObjectInstance, guid_type)));
    let axes = std::array::from_fn(|n| guid(INPUT_GUIDS.axes.addr() as u32 + 16 * n as u32));
    let bits = rc2k_core::input::joystick::axis_bits(
        at(entry, field!(Controller, axis_bits)),
        &object_type,
        &axes,
    );
    put(entry, field!(Controller, axis_bits), bits);
    1
}
