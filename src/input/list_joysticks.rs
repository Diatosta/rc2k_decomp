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

use crate::input::di_harness::{method, slot};
use crate::input::store::DEVICES;

/// The Rust version.
pub fn list_joysticks() {
    if DEVICES.no_joysticks.get() != 0 {
        return;
    }
    let found = crate::pointers::game_address(crate::input::joystick_found::callback as *const ());
    method(
        DEVICES.direct_input.get(),
        slot::ENUM_DEVICES,
        &[4, found, 0, 1],
    );
}
