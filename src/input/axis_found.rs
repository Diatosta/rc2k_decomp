//! `input::axis_found` (0x4083F4): DirectInput's `EnumObjects` callback for a joystick
//! being listed: the object's type GUID (`DIDEVICEOBJECTINSTANCEA` +4) named among the six
//! axes' (`GUID_XAxis`, `GUID_YAxis`, `GUID_ZAxis`, `GUID_RxAxis`, `GUID_RyAxis`,
//! `GUID_RzAxis`, at `unk_4F7E38` ..) sets the bit (1, 2, 4, 8, 0x10, 0x20) in the
//! controller's entry's +0x14 (kept: the bits are added to it). Always goes on (eax 1).
//!
//! In:  (stdcall) the `DIDEVICEOBJECTINSTANCEA`, the context (the controller's entry)
//! Out: eax = 1; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::joystick_found`. Shadow mode runs the original alone.
//! Self-test: the original and the Rust version called with each axis's GUID, a button's
//! and a slider's, on entries of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::field;
use crate::framework::{game_addr, rva};
use crate::input::devices::{at, put};
use crate::input::types::{Controller, DiDeviceObjectInstance};

const AXES_RVA: usize = rva(0x4F7E38); // unk_4F7E38: the six axes' GUIDs, 16 bytes each

/// The Rust version: eax.
/// `axis_found` as DirectInput calls it: what Rust callers hand `EnumObjects` (no hook).
pub unsafe extern "system" fn callback(object: u32, entry: u32) -> u32 {
    axis_found(object, entry)
}

pub fn axis_found(object: u32, entry: u32) -> u32 {
    // Converted: `rc2k_core::input::joystick::axis_bits`.
    let guid = |at: u32| unsafe { (at as *const [u8; 16]).read_unaligned() };
    let object_type = guid(object.wrapping_add(field!(DiDeviceObjectInstance, guid_type)));
    let axes = std::array::from_fn(|n| guid(game_addr(AXES_RVA) as u32 + 16 * n as u32));
    let bits = rc2k_core::input::joystick::axis_bits(
        at(entry, field!(Controller, axis_bits)),
        &object_type,
        &axes,
    );
    put(entry, field!(Controller, axis_bits), bits);
    1
}
