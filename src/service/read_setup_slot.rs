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
use crate::files::numbered_path::numbered_path_in_game;
use crate::files::read_file::read_file;
use crate::pointers::GamePtr;
use crate::session::{Bytes, Dword};

// ---- game data ----

pub const SETUP_FILE_LEN: usize = 0x48;
pub const SLOT_COUNT: u32 = 100;
pub const NAME_LEN: usize = 0x20;

/// IDA's address of the set-ups' variables.
pub const START: usize = 0x527838;
/// Their length.
pub const LEN: usize = 0x528504 - START;

/// The car set-ups' buffers.
#[repr(C)]
pub struct SetUps {
    /// `dword_527838`: the head of a set-up file as it is read or written.
    pub file: Bytes<SETUP_FILE_LEN>,
    /// `byte_527880`: the set-ups' names, 100 of 0x20 bytes.
    pub names: Bytes<{ SLOT_COUNT as usize * NAME_LEN }>,
    /// `dword_528500`: a value the options file keeps; nothing else reads it.
    pub kept: Dword,
}

/// The set-ups' buffers; the first values are the image's (all zero).
pub static SET_UPS: SetUps = SetUps {
    file: Bytes::new([0; SETUP_FILE_LEN]),
    names: Bytes::new([0; SLOT_COUNT as usize * NAME_LEN]),
    kept: Dword::new(0),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x527838).
pub fn home() -> usize {
    crate::pointers::host_number(&SET_UPS as *const SetUps)
}

/// IDA's address of the file name's template.
pub const TEMPLATE_START: usize = 0x529716;

/// `aSavedataCarset`: the set-up file's name, the number's two digits last.
pub static TEMPLATE: Bytes<27> = Bytes::new(*b"savedata\\carsetup\\carset00\0");

/// Where the template is (`moved::MOVED`'s home for IDA 0x529716).
pub fn template_home() -> usize {
    TEMPLATE.addr()
}

// "CRSU"
pub fn slot_name(slot: u32) -> *mut u8 {
    SET_UPS
        .names
        .as_ptr()
        .wrapping_add(slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub fn read_setup_slot(slot: u32) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    {
        let path = numbered_path_in_game(TEMPLATE.as_ptr(), slot)?;
        let file = GamePtr::new(SET_UPS.file.addr() as u32);
        let read = read_file(
            path,
            crate::pointers::host(file.addr()),
            SETUP_FILE_LEN as u32,
            0,
        )?;
        // Converted: `rc2k_core::saves::kept::setup_name`.
        let name = rc2k_core::saves::kept::setup_name(
            &file.read::<[u8; SETUP_FILE_LEN]>("the setup file")?,
        );
        match name {
            Some(name) if read.ok => {
                GamePtr::new(crate::pointers::game_address(slot_name(slot)))
                    .write_bytes(&name[..NAME_LEN], "a setup's name")?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&SET_UPS, START, START + LEN);
        crate::gfx::first_values_are_the_games(&TEMPLATE, TEMPLATE_START, TEMPLATE_START + 27);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(SetUps, names), 0x527880 - START);
        assert_eq!(offset_of!(SetUps, kept), 0x528500 - START);
        assert_eq!(crate::moved::place(START), SET_UPS.file.addr());
        assert_eq!(crate::moved::place(0x527880), SET_UPS.names.addr());
        assert_eq!(crate::moved::place(TEMPLATE_START), TEMPLATE.addr());
    }
}
