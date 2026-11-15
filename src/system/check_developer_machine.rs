//! Whether the computer is called "andrew".
//!
//! `DEVELOPER` = 0; the computer's name is asked for into the text buffer (`GetComputerNameA`,
//! through `system::computer_name`; if that fails the buffer is looked at as it is); if its first
//! six characters are "andrew" in any case, `DEVELOPER` = 1. (`race::tyre_change_gauge` reads
//! it.) One caller, `start`, which calls it directly.

const NAME_ROOM: u32 = 0x40;

/// The computer's name with its NUL (room for 0x40 bytes), if it can be had.
pub fn computer_name() -> Option<Vec<u8>> {
    let mut name = crate::os::computer_name(NAME_ROOM)?;
    name.push(0);
    Some(name)
}
pub fn check_developer_machine(
    strings: &crate::strings::owner::Strings,
    system: &crate::system::owner::System,
) {
    let developer = &system.machine().developer;
    developer.set(0);
    let text = &strings.text.buffer.text;
    if let Some(name) = computer_name() {
        text.write_at(0, &name, "the computer's name")
            .unwrap_or_else(|e| crate::fatal::raise::end("system::computer_name", e));
    }
    let (yes, _) = rc2k_core::system::machine::developer(core::array::from_fn(|n| text.get(n)));
    if yes {
        developer.set(1);
    }
}
