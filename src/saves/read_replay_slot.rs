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
use crate::framework::{game_addr, game_ptr, rva};
use crate::menu::name_lists::{NAME_LEN, REPLAY_NAMES_RVA};

// ---- game data ----

const TEMPLATE_RVA: usize = rva(0x52974B); // "savedata\replay\replay00"
const HEAD_RVA: usize = rva(0x528504); // dword_528504
const HEAD_LEN: usize = 0x430;
const SLOTS: u32 = 100;

fn slot_name(slot: u32) -> *mut u8 {
    game_addr(REPLAY_NAMES_RVA + slot as usize * NAME_LEN) as *mut u8
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_replay_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        let head: *mut u8 = game_ptr(HEAD_RVA);
        let read = read_file(path, head, HEAD_LEN as u32, 0)?;
        // Converted: `rc2k_core::saves::kept::slot_name`.
        use rc2k_core::saves::kept;
        let block = std::slice::from_raw_parts(game_ptr::<u8>(HEAD_RVA), 0x30);
        match kept::slot_name(block, kept::REPLAY, kept::REPLAY_VERSION) {
            Some(name) if read.ok => {
                std::ptr::copy(name.as_ptr(), slot_name(slot), NAME_LEN);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
