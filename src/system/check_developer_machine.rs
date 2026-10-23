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

use crate::framework::{game_write, rva};
use crate::strings::text_from_ids::text;
use windows_sys::Win32::System::WindowsProgramming::GetComputerNameA;

const DEVELOPER_RVA: usize = rva(0x529D94); // dword_529D94
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
        game_write(DEVELOPER_RVA, 0u32);
        computer_name(text());
        // Converted: `rc2k_core::system::machine::developer`.
        let (yes, _) =
            rc2k_core::system::machine::developer((text() as *const [u8; 6]).read_unaligned());
        if yes {
            game_write(DEVELOPER_RVA, 1u32);
        }
    }
}
