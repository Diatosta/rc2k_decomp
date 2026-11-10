//! `race::replay_path` (0x41EF27): a replay's file by its number: 222 the last race's
//! (`var\tmprpl`); below 100 a saved one (`savedata\replay\replay00` numbered); else one of the
//! game's own (`files\replay\replay00` numbered by the number less 100), both through
//! `files::numbered_path`.
//!
//! Takes: the number
//!
//! Callers: `race::open_viewed_replay` (reads ZF), `race::save_viewed_replay`. Refuses what
//! `numbered_path` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;

/// IDA's address of the last race's replay file's name.
pub const LAST_RACE_START: usize = 0x5295F0;
/// Its room.
pub const LAST_RACE_LEN: usize = 0x5296FB - LAST_RACE_START;

/// IDA's address of the replay files' names' templates.
#[cfg(test)]
pub const TEMPLATES_START: usize = 0x52974B;
/// Their length.
#[cfg(test)]
pub const TEMPLATES_LEN: usize = 0x52977A - TEMPLATES_START;

/// The replay files' names, the number's two digits last.
pub struct Templates {
    /// `aSavedataReplay`: "savedata\replay\replay00".
    pub saved: [u8; 25],
    /// `aFilesReplayRep`: "files\replay\replay00".
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

#[cfg(test)]
mod block_tests {
    use super::*;
    use crate::race::owner::race;

    #[test]
    fn values_are_the_games() {
        let shared = &race().shared;
        let image = crate::moved::image_bytes(LAST_RACE_START, LAST_RACE_LEN);
        assert_eq!(&race().replay.last_race[..], &image[..]);
        let image = crate::moved::image_bytes(TEMPLATES_START, TEMPLATES_LEN);
        let mine = [
            &shared.replay_templates.saved[..],
            &shared.replay_templates.own[..],
        ]
        .concat();
        assert_eq!(mine, image);
    }
}
// `numbered_path`'s buffer

/// The last race's file's name (without its NUL).
pub fn last_race(replay: &Replay) -> Vec<u8> {
    let bytes = &replay.last_race;
    bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())].to_vec()
}

/// Here the path's bytes (without the NUL); the original's eax is the last race's name or
/// `numbered_path`'s buffer, which this still fills.
#[derive(Clone, Debug)]
pub struct ReplayPath {
    pub path: Vec<u8>,
    pub number: u32,
    pub last_race: bool,
}

/// `slot` the replay.
pub fn replay_path(race: &Race, slot: u32) -> Result<ReplayPath, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer::ReplayFile`.
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
