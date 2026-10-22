//! `input::list_joysticks` (0x408216): unless the configuration turned them off
//! (`dword_4F68DC` not 0), the attached joysticks listed into `g_controllers`
//! (`EnumDevices` of `dword_4F6904`, DirectInput, for joysticks (4), attached only (1), with
//! `input::joystick_found`, context 0; its result not looked at).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_thread` (its start). The method DirectInput's own (in the
//! standalone program Rust's, `input::di_objects`). Shadow mode runs the original alone.
//! Self-test: the original and the Rust version on DirectInput objects made in Rust
//! (`input::di_harness`) with the joystick and the wheel attached: listed from none, from
//! 15 (one room left) and turned off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::input::di_harness::{DIRECT_INPUT_RVA, g, method, slot};

const OFF_RVA: usize = rva(0x4F68DC); // dword_4F68DC: joysticks turned off

/// The Rust version.
pub fn list_joysticks() {
    if g(OFF_RVA) != 0 {
        return;
    }
    let found = crate::input::joystick_found::callback as unsafe extern "system" fn(u32, u32) -> u32
        as usize as u32;
    method(g(DIRECT_INPUT_RVA), slot::ENUM_DEVICES, &[4, found, 0, 1]);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    list_joysticks();
    Ok(())
}
