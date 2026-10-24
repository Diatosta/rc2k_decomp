//! `race::replay_path` (0x41EF27): a replay's file by its number: 222 the last race's
//! (`var\tmprpl`); below 100 a saved one (`savedata\replay\replay00` numbered); else
//! one of the game's own (`files\replay\replay00` numbered by the number less 100), both
//! through `files::numbered_path`.
//!
//! In:  ebx the number
//! Out: eax the path; ebx less 100 for the game's own; ZF set for 222 (from `cmp`), else
//!      clear (`test eax, eax` of the path); carry clear
//!
//! Callers: `race::open_viewed_replay` (reads ZF), `race::save_viewed_replay`. Refuses what
//! `numbered_path` refuses.

use crate::errors::Refused;
use crate::physics::flow::a;

pub const LAST_RACE_PATH: usize = 0x5295F0; // "var\tmprpl"
const SAVED_TEMPLATE: usize = 0x52974B; // "savedata\replay\replay00"
const OWN_TEMPLATE: usize = 0x529764; // "files\replay\replay00"
// `numbered_path`'s buffer

/// What the original leaves in its registers: eax the path (0 for none), ebx the number
/// the path was made with (for a file of the game's own), and whether it is the last
/// race's file (the zero flag).
#[derive(Clone, Copy, Debug)]
pub struct ReplayPath {
    pub path: u32,
    pub number: u32,
    pub last_race: bool,
}

/// The Rust version. `slot` the replay (ebx).
pub fn replay_path(slot: u32) -> Result<ReplayPath, Refused> {
    // Converted: `rc2k_core::race::viewer::ReplayFile`.
    use rc2k_core::race::viewer::ReplayFile;
    let mut number = slot;
    let template = match ReplayFile::of(slot) {
        ReplayFile::LastRace => {
            return Ok(ReplayPath {
                path: a(LAST_RACE_PATH),
                number,
                last_race: true,
            });
        }
        ReplayFile::Saved(_) => SAVED_TEMPLATE,
        ReplayFile::Own(n) => {
            number = n;
            OWN_TEMPLATE
        }
    };
    let path = unsafe {
        crate::files::numbered_path::numbered_path_in_game(a(template) as *const u8, number)
    }?;
    Ok(ReplayPath {
        path: path as u32,
        number,
        last_race: path as u32 == 0,
    })
}
