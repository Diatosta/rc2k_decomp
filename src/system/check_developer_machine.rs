//! `system::check_developer_machine` (0x416292): whether the computer is called "andrew".
//!
//! In:  nothing
//! Out: ebx = the text buffer; ax = the last two characters looked at, in lower case; the
//!      rest preserved
//! `dword_529D94` = 0; the computer's name is asked for into the text buffer
//! (`GetComputerNameA`, through `sub_410848`; if that fails the buffer is looked at as it
//! is); if its first six characters are "andrew" in any case, `dword_529D94` = 1.
//! (`race::tyre_change_gauge` reads it.) One caller, `start`.
//! Shadow mode asks twice. Self-test: a redirected copy whose name comes from the test.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
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

/// `name` writes the computer's name at the place it is given, if it can. Returns the
/// low word the original leaves in eax.
pub unsafe fn check_developer_machine(name: &mut dyn FnMut(*mut u8)) -> u16 {
    unsafe {
        game_write(DEVELOPER_RVA, 0u32);
        name(text());
        // Converted: `rc2k_core::system::machine::developer`.
        let (yes, last) =
            rc2k_core::system::machine::developer((text() as *const [u8; 6]).read_unaligned());
        if yes {
            game_write(DEVELOPER_RVA, 1u32);
        }
        last
    }
}

fn left(r: &Regs, last: u16) -> Left {
    Left::nothing()
        .with(EBX, text() as u32)
        .with(EAX, r.eax & 0xFFFF_0000 | last as u32)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let last = unsafe {
        check_developer_machine(&mut |to| {
            computer_name(to);
        })
    };
    Ok(left(r, last))
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
