//! `system::read_keyboard_type` (0x410AA3): `g_keyboardType` (0x518404) 2 when the
//! keyboard's subtype (`GetKeyboardType(1)`) is 0xD01 .. 0xD06 (a Japanese keyboard's);
//! otherwise it is left as it is.
//!
//! One caller, `start` (0x40101F); The Win32 call is made from Rust. `start` calls it directly.

// ---- game data ----

/// The value the variable is given for a subtype, if any.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    // Converted: `rc2k_core::system::machine::keyboard_type`.
    rc2k_core::system::machine::keyboard_type(subtype)
}

pub fn read_keyboard_type(system: &crate::system::owner::System) {
    let subtype = crate::os::keyboard_subtype();
    if let Some(kind) = keyboard_type(subtype) {
        system.session.machine.keyboard_type.set(kind);
    }
}
