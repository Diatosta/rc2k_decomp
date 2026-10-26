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
use crate::files::numbered_path::numbered_path_in_game;
use crate::files::read_file::read_file;
use crate::menu::name_lists::NAME_LEN;
use crate::race::replay::REPLAYS;

// ---- game data ----

const HEAD_LEN: usize = 0x430;
const SLOTS: u32 = 100;

fn slot_name(slot: u32) -> *mut u8 {
    REPLAYS
        .names
        .as_ptr()
        .wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_replay_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let path = numbered_path_in_game(crate::race::replay_path::TEMPLATES.saved.as_ptr(), slot)?;
        let head: *mut u8 = REPLAYS.header.as_ptr();
        let read = read_file(path, head, HEAD_LEN as u32, 0)?;
        // Converted: `rc2k_core::saves::kept::slot_name`.
        use rc2k_core::saves::kept;
        let block = std::slice::from_raw_parts(REPLAYS.header.as_ptr(), 0x30);
        match kept::slot_name(block, kept::REPLAY, kept::REPLAY_VERSION) {
            Some(name) if read.ok => {
                crate::pointers::GamePtr::new(slot_name(slot) as u32)
                    .write_bytes(&name[..NAME_LEN], "a replay slot's name")?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
