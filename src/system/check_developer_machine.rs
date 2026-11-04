//! `system::check_developer_machine` (0x416292): whether the computer is called "andrew".
//!
//! In:  nothing
//! Out: ebx = the text buffer; ax = the last two characters looked at, in lower case; the
//!      rest preserved
//! `dword_529D94` = 0; the computer's name is asked for into the text buffer
//! (`GetComputerNameA`, through `sub_410848`; if that fails the buffer is looked at as it
//! is); if its first six characters are "andrew" in any case, `dword_529D94` = 1.
//! (`race::tyre_change_gauge` reads it.) One caller, `start`, which calls it directly
//! (2026-10-23); the registers above are the original's, for reference.

/// IDA's address of the switch.
pub const DEVELOPER_START: usize = 0x529D94;

/// `g_developerMachine` (`dword_529D94`): 1 on a computer called "andrew".
pub static DEVELOPER: crate::session::Dword = crate::session::Dword::new(0);

/// Where the switch is (`moved::MOVED`'s home for IDA 0x529D94).
pub fn developer_home() -> usize {
    DEVELOPER.addr() as usize
}

const NAME_ROOM: u32 = 0x40;

// ---- Rust version ----

/// `sub_410848`: the computer's name with its NUL at game address `to` (0x40 bytes), if
/// it can be had.
pub fn computer_name(to: crate::pointers::GamePtr) -> bool {
    let Some(mut name) = crate::os::computer_name(NAME_ROOM) else {
        return false;
    };
    name.push(0);
    to.write_bytes(&name, "the computer's name")
        .unwrap_or_else(|e| crate::fatal::raise::end("system::computer_name", e));
    true
}

/// The Rust version (see the header).
pub fn check_developer_machine() {
    DEVELOPER.set(0);
    let at = crate::strings::text_from_ids::text_at();
    computer_name(at);
    // Converted: `rc2k_core::system::machine::developer`.
    let (yes, _) = rc2k_core::system::machine::developer(
        at.read("the computer's name")
            .unwrap_or_else(|e| crate::fatal::raise::end("the computer's name", e)),
    );
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
