//! `input::list_joysticks` (0x408216): unless the configuration turned them off
//! (`dword_4F68DC` not 0), the attached joysticks listed into `g_controllers`
//! (`EnumDevices` of `dword_4F6904`, DirectInput, for joysticks (4), attached only (1), with
//! `input::joystick_found`, context 0; its result not looked at).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_thread` (its start). The method DirectInput's own (in the
//! standalone program Rust's, `input::di_objects`).

use crate::input::di_harness::{DIRECT_INPUT_RVA, g, method, slot};
use crate::input::store::DeviceSlot;

const OFF_RVA: DeviceSlot = DeviceSlot::at(0x4F68DC); // dword_4F68DC: joysticks turned off

/// The Rust version.
pub fn list_joysticks() {
    if g(OFF_RVA) != 0 {
        return;
    }
    let found = crate::input::joystick_found::callback as unsafe extern "system" fn(u32, u32) -> u32
        as usize as u32;
    method(g(DIRECT_INPUT_RVA), slot::ENUM_DEVICES, &[4, found, 0, 1]);
}
