//! `race::replay_path` (0x41EF27): a replay's file by its number: 222 the last race's
//! (`var\tmprpl`); below 100 a saved one (`savedata\replay\replay00` numbered); else
//! one of the game's own (`files\replay\replay00` numbered by the number less 100), both
//! through `files::numbered_path`.
//!
//! In:  ebx the number
//! Out: eax the path; ebx less 100 for the game's own; ZF set for 222 (from `cmp`), else
//!      clear (`test eax, eax` of the path); carry clear
//!
//! Callers: `race::open_viewed_replay` (reads ZF), `race::save_viewed_replay`. Refuses
//! what `numbered_path` refuses. Shadow mode compares. Self-test: 222, 0, 7, 99, 100, 105
//! and 150 against the original; the path buffer put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::regs_entry;

pub const LAST_RACE_PATH: usize = 0x5295F0; // "var\tmprpl"
const SAVED_TEMPLATE: usize = 0x52974B; // "savedata\replay\replay00"
const OWN_TEMPLATE: usize = 0x529764; // "files\replay\replay00"
// `numbered_path`'s buffer

/// The Rust version: the registers and flags it leaves.
pub fn replay_path(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::race::viewer::ReplayFile`.
    use rc2k_core::race::viewer::ReplayFile;
    r.eax = a(LAST_RACE_PATH);
    r.set_carry(false);
    let template = match ReplayFile::of(r.ebx) {
        ReplayFile::LastRace => {
            r.set_zero(true);
            return Ok(r);
        }
        ReplayFile::Saved(_) => SAVED_TEMPLATE,
        ReplayFile::Own(n) => {
            r.ebx = n;
            OWN_TEMPLATE
        }
    };
    let path = unsafe {
        crate::files::numbered_path::numbered_path_in_game(a(template) as *const u8, r.ebx)
    }?;
    r.eax = path as u32;
    r.set_zero(r.eax == 0);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &replay_path)
}
