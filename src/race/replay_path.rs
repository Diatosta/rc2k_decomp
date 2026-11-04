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

/// IDA's address of the replay files' names' templates.
pub const TEMPLATES_START: usize = 0x52974B;
/// Their length.
pub const TEMPLATES_LEN: usize = 0x52977A - TEMPLATES_START;

/// The replay files' names, the number's two digits last.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
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
        assert_eq!(
            crate::moved::place(LAST_RACE_START),
            LAST_RACE.addr() as usize
        );
        assert_eq!(crate::moved::place(0x529764), TEMPLATES.own.addr() as usize);
    }
}
// `numbered_path`'s buffer

/// The last race's file's name (without its NUL).
pub fn last_race() -> Vec<u8> {
    let bytes = LAST_RACE.read();
    bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())].to_vec()
}

/// What the original leaves in its registers: eax the path, ebx the number the path was
/// made with (for a file of the game's own), and whether it is the last race's file (the
/// zero flag). Here the path's bytes (without the NUL); the original's eax is the last
/// race's name or `numbered_path`'s buffer, which this still fills.
#[derive(Clone, Debug)]
pub struct ReplayPath {
    pub path: Vec<u8>,
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
                path: last_race(),
                number,
                last_race: true,
            });
        }
        ReplayFile::Saved(_) => TEMPLATES.saved.read().to_vec(),
        ReplayFile::Own(n) => {
            number = n;
            TEMPLATES.own.read().to_vec()
        }
    };
    let path =
        crate::files::numbered_path::numbered_path_bytes(&template[..template.len() - 1], number)?;
    Ok(ReplayPath {
        path,
        number,
        last_race: false,
    })
}
