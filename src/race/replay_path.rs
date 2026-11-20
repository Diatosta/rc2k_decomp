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
pub(super) const LAST_RACE_LEN: usize = 0x10B;

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

/// The last race's file's name (without its NUL). A copy let in beside another in the same
/// folder (`--allow-another`) has a file of its own, so that two copies' races do not write one.
pub fn last_race(replay: &Replay) -> Vec<u8> {
    let bytes = &replay.last_race;
    let mut name = bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())].to_vec();
    name.extend_from_slice(crate::single_copy::last_race_suffix().as_bytes());
    name
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

/// The extension of a replay file in the own format (`rc2k_core::race::replay_v2`): the file of
/// a slot beside the original's, `replay07.rc2r` beside `replay07`.
const OWN_FORMAT_SUFFIX: &[u8] = b".rc2r";

/// The own-format file that goes with a saved or shipped replay's path.
pub fn own_format(path: &[u8]) -> Vec<u8> {
    let mut name = path.to_vec();
    name.extend_from_slice(OWN_FORMAT_SUFFIX);
    name
}

/// Whether a file is there.
pub fn exists(path: &[u8]) -> bool {
    crate::files::host_path(path).is_file()
}

/// The file at `path` taken away; whether there was one (a slot saved over has only the files
/// of the save left).
pub fn remove(path: &[u8]) -> bool {
    std::fs::remove_file(crate::files::host_path(path)).is_ok()
}

/// The file a slot loads from: the own-format one beside the original's when there is one, else
/// the original's (a replay copied in alone). A file named on the command line is that file.
pub fn file_to_load(slot: u32, path: Vec<u8>) -> Vec<u8> {
    use rc2k_core::race::viewer::ReplayFile;
    if matches!(
        ReplayFile::of(slot),
        ReplayFile::Saved(_) | ReplayFile::Own(_)
    ) {
        let own = own_format(&path);
        if exists(&own) {
            return own;
        }
    }
    path
}
