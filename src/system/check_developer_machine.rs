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

use crate::strings::text_from_ids::text;
use windows_sys::Win32::System::WindowsProgramming::GetComputerNameA;

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

/// `sub_410848`: the computer's name at `to` (0x40 bytes), if it can be had.
pub unsafe fn computer_name(to: *mut u8) -> bool {
    let mut room = NAME_ROOM;
    unsafe { GetComputerNameA(to, &mut room) != 0 }
}

/// The Rust version (see the header).
pub fn check_developer_machine() {
    unsafe {
        DEVELOPER.set(0);
        computer_name(text());
        // Converted: `rc2k_core::system::machine::developer`.
        let (yes, _) =
            rc2k_core::system::machine::developer((text() as *const [u8; 6]).read_unaligned());
        if yes {
            DEVELOPER.set(1);
        }
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
