//! `saves::write_savegame_slot` (0x4AB558): builds a saved game in the buffer at 0x51EBB4 and
//! writes it to `savedata\savegame\svgameNN`.
//!
//! In:  ebx = slot, 0 .. 99
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! The file is always 0x8000 bytes: "R5SG", 0, version 0x51122, `packed_local_time`, the
//! slot's 0x20-byte name from the saved game list (`byte_526BB4`), then what
//! `gather_savegame` collects (0x725C bytes); the rest of the buffer is written as it is.
//! Only caller: `menu::edit_savegame_name`.
//! Refuses a slot past 99 and what `gather_savegame` refuses.
//! Shadow mode writes the file twice; the time stamp is left out of the comparison.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_in_game;
use crate::files::write_file::write_file;
use crate::framework::{game_ptr, rva};
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_NAMES_RVA};
use crate::saves::gather_savegame::gather_savegame;
use crate::system::packed_local_time::packed_local_time;
use std::ptr::copy_nonoverlapping;

// ---- game data ----

pub const FILE_RVA: usize = rva(0x51EBB4); // dword_51EBB4: the saved game as it is written
pub const FILE_LEN: usize = 0x8000;
pub const TEMPLATE_RVA: usize = rva(0x5296FB); // aSavedataSavega: "savedata\savegame\svgame00"
pub const SLOTS: u32 = 100;

pub const MAGIC: u32 = 0x4753_3552; // "R5SG"
pub const VERSION: u32 = 0x5_1122;

pub fn slot_name(slot: u32) -> *mut u8 {
    game_ptr::<u8>(SAVEGAME_NAMES_RVA).wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the file was written.
pub unsafe fn write_savegame_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    unsafe {
        let file: *mut u8 = game_ptr(FILE_RVA);
        // First, because it is what can be refused: nothing else is written before it.
        gather_savegame(file.add(0x30))?;
        // Converted: `rc2k_core::saves::kept::slot_head`.
        use rc2k_core::saves::kept;
        let head = kept::slot_head(
            kept::SAVEGAME,
            kept::SAVEGAME_VERSION,
            packed_local_time(),
            &(slot_name(slot) as *const [u8; NAME_LEN]).read_unaligned(),
        );
        copy_nonoverlapping(head.as_ptr(), file, head.len());
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        Ok(write_file(path, file, FILE_LEN as u32)?.ok)
    }
}
