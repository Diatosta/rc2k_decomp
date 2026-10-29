//! `saves::read_replay_slot` (0x41ECAD): reads a replay's head,
//! `savedata\replay\replayNN`, and copies its name into the replay list.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! Up to 0x430 bytes to `dword_528504`. Accepted if the read did not fail, the first dword
//! is "REPL" and the dword at +8 is 0x50400; the 0x20 bytes at +0x10 then go to
//! `byte_528934[slot * 0x20]`. A short file is not noticed as such: what the buffer held
//! before decides (as in `saves::read_savegame_slot`).
//! One caller, `sub_411239`, for all 100 slots at start-up. Refuses a slot past 99.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::menu::name_lists::NAME_LEN;
use crate::race::replay::REPLAYS;

// ---- game data ----

const HEAD_LEN: usize = 0x430;
const SLOTS: u32 = 100;

fn slot_name(slot: u32) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::new(REPLAYS.names.addr()).offset(slot * NAME_LEN as u32)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub fn read_replay_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let template = crate::race::replay_path::TEMPLATES.saved.read();
    let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
    let mut bytes = REPLAYS.header.read();
    let read = read_file_into(&path[..], &mut bytes[..HEAD_LEN], 0)?;
    REPLAYS.header.write(&bytes);
    // Converted: `rc2k_core::saves::kept::slot_name`.
    use rc2k_core::saves::kept;
    let block = &bytes[..0x30];
    match kept::slot_name(block, kept::REPLAY, kept::REPLAY_VERSION) {
        Some(name) if read.ok => {
            slot_name(slot).write_bytes(&name[..NAME_LEN], "a replay slot's name")?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
