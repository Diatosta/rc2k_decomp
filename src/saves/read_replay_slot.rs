//! Reads a replay's head, `savedata\replay\replayNN`, and copies its name into the replay list.
//!
//! `slot` is 0 .. 99. Up to 0x430 bytes go to the replay header (`header`). Accepted if the read
//! did not fail, the first dword is "REPL" and the dword at +8 is 0x50400 (or 0x50401, a replay with the extra tape); the 0x20 bytes at +0x10
//! then go to the replay names (`names[slot * 0x20]`). A short file is not noticed as such: what
//! the buffer held before decides (as in `saves::read_savegame_slot`). Refuses a slot past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::menu::name_lists::NAME_LEN;
use crate::race::owner::Race;

const HEAD_LEN: usize = 0x430;
const SLOTS: u32 = 100;

fn slot_name(race: &'static Race, slot: u32) -> crate::memory::place::BytePlace {
    crate::memory::place::BytePlace::In(&race.shared.replays().names, slot as usize * NAME_LEN)
}

/// True if the slot's file was read and accepted.
pub fn read_replay_slot(slot: u32, race: &'static Race) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let template = race.shared.replay_templates.saved;
    let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
    // A slot with a file of the own format is read from it, else from the 1999 format's.
    let own = crate::race::replay_path::own_format(&path);
    let (path, kind) = if crate::race::replay_path::exists(&own) {
        (own, crate::race::replay::SLOT_NEW)
    } else {
        (path, crate::race::replay::SLOT_OLD)
    };
    race.shared
        .replays()
        .kinds
        .set(slot as usize, crate::race::replay::SLOT_NONE);
    let mut bytes = race.shared.replays().header.read();
    let read = read_file_into(&path[..], &mut bytes[..HEAD_LEN], 0)?;
    race.shared.replays().header.write(&bytes);
    use rc2k_core::saves::kept;
    let block = &bytes[..0x30];
    let name = kept::replay_name(block);
    match name {
        Some(name) if read.ok => {
            slot_name(race, slot).write(&name[..NAME_LEN], "a replay slot's name")?;
            race.shared.replays().kinds.set(slot as usize, kind);
            Ok(true)
        }
        _ => Ok(false),
    }
}
