//! `system::check_developer_machine` (0x416292): whether the computer is called "andrew".
//!
//! `dword_529D94` = 0; the computer's name is asked for into the text buffer (`GetComputerNameA`,
//! through `system::computer_name`; if that fails the buffer is looked at as it is); if its first
//! six characters are "andrew" in any case, `dword_529D94` = 1. (`race::tyre_change_gauge` reads
//! it.) One caller, `start`, which calls it directly.

/// IDA's address of the switch.
pub const DEVELOPER_START: usize = 0x529D94;

/// `g_developerMachine` (`dword_529D94`): 1 on a computer called "andrew".
pub static DEVELOPER: crate::session::Dword = crate::session::Dword::new(0);

const NAME_ROOM: u32 = 0x40;

/// `system::computer_name`: the computer's name with its NUL (room for 0x40 bytes), if it can be
/// had; the callers store it where the original did.
pub fn computer_name() -> Option<Vec<u8>> {
    let mut name = crate::os::computer_name(NAME_ROOM)?;
    name.push(0);
    Some(name)
}
pub fn check_developer_machine() {
    DEVELOPER.set(0);
    let text = &crate::strings::owner::strings().text.buffer.text;
    if let Some(name) = computer_name() {
        text.write_at(0, &name, "the computer's name")
            .unwrap_or_else(|e| crate::fatal::raise::end("system::computer_name", e));
    }
    // Converted: `rc2k_core::system::machine::developer`.
    let (yes, _) = rc2k_core::system::machine::developer(core::array::from_fn(|n| text.get(n)));
    if yes {
        DEVELOPER.set(1);
    }
}

#[cfg(test)]
mod developer_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        crate::gfx::first_values_are_the_games(&DEVELOPER, DEVELOPER_START, DEVELOPER_START + 4);
        assert_eq!(
            crate::moved::place(DEVELOPER_START),
            DEVELOPER.addr() as usize
        );
    }
}
