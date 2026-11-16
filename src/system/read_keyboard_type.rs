//! Sets the keyboard type to 2 when the keyboard's subtype is 0xD01 .. 0xD06 (a Japanese
//! keyboard's); otherwise it is left as it is.

/// The value the variable is given for a subtype, if any.
pub fn keyboard_type(subtype: u32) -> Option<u32> {
    rc2k_core::system::machine::keyboard_type(subtype)
}

pub fn read_keyboard_type(system: &crate::system::owner::System) {
    let subtype = crate::platform::os::keyboard_subtype();
    if let Some(kind) = keyboard_type(subtype) {
        system.machine().keyboard_type.set(kind);
    }
}
