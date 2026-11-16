//! `race::replay_path`: a replay's file by its number: 222 the last race's (`var\tmprpl`); below
//! 100 a saved one (`savedata\replay\replay00` numbered); else one of the game's own
//! (`files\replay\replay00` numbered by the number less 100), both through `files::numbered_path`.
//! One number is the file named when the program was started (`Replay.given_file`): opened where
//! it lies, as the game's own are.
//!
//! Takes the number.
//!
//! Refuses what `numbered_path` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;

/// The room of the last race's replay file's name.
pub const LAST_RACE_LEN: usize = 0x10B;

/// The replay files' names, the number's two digits last.
pub struct Templates {
    /// "savedata\replay\replay00".
    pub saved: [u8; 25],
    /// "files\replay\replay00".
    pub own: [u8; 22],
}

impl Templates {
    pub const fn new() -> Self {
        Templates {
            saved: *b"savedata\\replay\\replay00\0",
            own: *b"files\\replay\\replay00\0",
        }
    }
}

/// The last race's file's name (without its NUL).
pub fn last_race(replay: &Replay) -> Vec<u8> {
    let bytes = &replay.last_race;
    bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())].to_vec()
}

/// The path's bytes (without the NUL), and whether it is the last race's name; `numbered_path`'s
/// buffer is still filled.
#[derive(Clone, Debug)]
pub struct ReplayPath {
    pub path: Vec<u8>,
    pub number: u32,
    pub last_race: bool,
}

/// The path of replay `slot`.
pub fn replay_path(race: &Race, slot: u32) -> Result<ReplayPath, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Logic in `rc2k_core::race::viewer::ReplayFile`.
    use rc2k_core::race::viewer::ReplayFile;
    let mut number = slot;
    let template = match ReplayFile::of(slot) {
        ReplayFile::LastRace => {
            return Ok(ReplayPath {
                path: last_race(replay),
                number,
                last_race: true,
            });
        }
        ReplayFile::Given => {
            let path = replay
                .given_file
                .get()
                .ok_or(crate::errors::BadInput("no replay file was named"))?;
            return Ok(ReplayPath {
                path: path.clone(),
                number,
                last_race: false,
            });
        }
        ReplayFile::Saved(_) => shared.replay_templates.saved.to_vec(),
        ReplayFile::Own(n) => {
            number = n;
            shared.replay_templates.own.to_vec()
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
