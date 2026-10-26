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
use crate::session::{Bytes, with_runs};

/// IDA's address of the last race's replay file's name.
pub const LAST_RACE_START: usize = 0x5295F0;
/// Its room.
pub const LAST_RACE_LEN: usize = 0x5296FB - LAST_RACE_START;

/// `aVarTmprpl` (room for 0x10B bytes): the last race's replay file's name.
pub static LAST_RACE: Bytes<LAST_RACE_LEN> = Bytes::new(with_runs(&[(0, b"var\\tmprpl\0")]));

/// Where the name is (`moved::MOVED`'s home for IDA 0x5295F0).
pub fn last_race_home() -> usize {
    LAST_RACE.addr()
}

/// IDA's address of the replay files' names' templates.
pub const TEMPLATES_START: usize = 0x52974B;
/// Their length.
pub const TEMPLATES_LEN: usize = 0x52977A - TEMPLATES_START;

/// The replay files' names, the number's two digits last.
#[repr(C)]
pub struct Templates {
    /// `aSavedataReplay`: "savedata\replay\replay00".
    pub saved: Bytes<25>,
    /// `aFilesReplayRep`: "files\replay\replay00".
    pub own: Bytes<22>,
}

pub static TEMPLATES: Templates = Templates {
    saved: Bytes::new(*b"savedata\\replay\\replay00\0"),
    own: Bytes::new(*b"files\\replay\\replay00\0"),
};

/// Where the templates are (`moved::MOVED`'s home for IDA 0x52974B).
pub fn templates_home() -> usize {
    crate::pointers::host_number(&TEMPLATES as *const Templates)
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &LAST_RACE,
            LAST_RACE_START,
            LAST_RACE_START + LAST_RACE_LEN,
        );
        crate::gfx::first_values_are_the_games(
            &TEMPLATES,
            TEMPLATES_START,
            TEMPLATES_START + TEMPLATES_LEN,
        );
        assert_eq!(crate::moved::place(LAST_RACE_START), LAST_RACE.addr());
        assert_eq!(crate::moved::place(0x529764), TEMPLATES.own.addr());
    }
}
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
                path: LAST_RACE.addr() as u32,
                number,
                last_race: true,
            });
        }
        ReplayFile::Saved(_) => TEMPLATES.saved.addr(),
        ReplayFile::Own(n) => {
            number = n;
            TEMPLATES.own.addr()
        }
    };
    let path = crate::files::numbered_path::numbered_path_in_game(
        crate::pointers::host_at(template) as *const u8,
        number,
    )?;
    Ok(ReplayPath {
        path: crate::pointers::game_address(path),
        number,
        last_race: crate::pointers::game_address(path) == 0,
    })
}
