//! `service::read_setup_slot` (0x4AB31E): reads the head of a saved car setup,
//! `savedata\carsetup\carsetNN`, into the buffer at 0x527838 and copies its name into the
//! slot name table.
//!
//! In:  ebx = slot, 0 .. 99 (eax is saved and compared but has no effect)
//! Out: carry = failed (zero flag = succeeded); every register preserved
//! It reads 0x48 bytes. The file is accepted if the read did not fail, the first dword is
//! "CRSU" and the dword at +8 is 0x50000; the 0x20 bytes at +0x10 then go to
//! `byte_527880[slot * 0x20]`. A file shorter than 0x48 bytes is not noticed as such: what the
//! buffer held before decides (reproduced). Refuses a slot past the table's 100 entries.
//! Called for all 100 slots at start-up (`sub_411239`) and by `load_setup_slot`.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::{buffer as path_buffer, numbered_path_in_game};
use crate::files::read_file::{LAST_FILE_PATH_RVA, read_file};
use crate::framework::{game_addr, game_ptr, rva};
use crate::service::harness::{Region, at};
use std::ptr::copy_nonoverlapping;

// ---- game data ----

pub const TEMPLATE_RVA: usize = rva(0x529716); // aSavedataCarset: "savedata\carsetup\carset00"
pub const SETUP_FILE_RVA: usize = rva(0x527838); // dword_527838: the head of a setup file
pub const SETUP_FILE_LEN: usize = 0x48;
const SLOT_NAMES_RVA: usize = rva(0x527880); // byte_527880[100][0x20]
pub const SLOT_COUNT: u32 = 100;
pub const NAME_LEN: usize = 0x20;

// "CRSU"
pub fn slot_name(slot: u32) -> *mut u8 {
    game_ptr::<u8>(SLOT_NAMES_RVA).wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub unsafe fn read_setup_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    unsafe {
        let path = numbered_path_in_game(game_ptr(TEMPLATE_RVA), slot)?;
        let file: *mut u8 = game_ptr(SETUP_FILE_RVA);
        let read = read_file(path, file, SETUP_FILE_LEN as u32, 0)?;
        // Converted: `rc2k_core::saves::kept::setup_name`.
        let name = rc2k_core::saves::kept::setup_name(
            &(file as *const [u8; SETUP_FILE_LEN]).read_unaligned(),
        );
        match name {
            Some(name) if read.ok => {
                copy_nonoverlapping(name.as_ptr(), slot_name(slot), NAME_LEN);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

/// What reading slot `slot` may write.
pub fn regions(slot: u32) -> Vec<Region> {
    let mut regions = vec![
        (game_addr(SETUP_FILE_RVA) as u32, SETUP_FILE_LEN),
        (path_buffer() as u32, 0x100),
        at(LAST_FILE_PATH_RVA),
    ];
    if slot < SLOT_COUNT {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}
