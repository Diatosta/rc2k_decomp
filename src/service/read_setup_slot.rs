//! `service::read_setup_slot` (0x4AB31E): reads the head of a saved car setup,
//! `savedata\carsetup\carsetNN`, into the buffer at 0x527838 and copies its name into the
//! slot name table.
//!
//! Takes slot, 0 .. 99. It reads 0x48 bytes. The file is accepted if the read did not fail, the
//! first dword is "CRSU" and the dword at +8 is 0x50000; the 0x20 bytes at +0x10 then go to
//! `byte_527880[slot * 0x20]`. A file shorter than 0x48 bytes is not noticed as such: what the
//! buffer held before decides (reproduced). Refuses a slot past the table's 100 entries. Called for
//! all 100 slots at start-up (`system::load_game_data`) and by `load_setup_slot`.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::service::owner::{Service, service};
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
#[derive(rc2k_derive::Cells)]
pub struct SetUps {
    /// `dword_527838`: the head of a set-up file as it is read or written.
    pub file: Bytes<SETUP_FILE_LEN>,
    /// `byte_527880`: the set-ups' names, 100 of 0x20 bytes.
    pub names: Bytes<{ SLOT_COUNT as usize * NAME_LEN }>,
    /// `dword_528500`: a value the options file keeps; nothing else reads it.
    pub kept: Dword,
}

/// The set-ups' buffers; the first values are the image's (all zero).
impl SetUps {
    pub const fn new() -> Self {
        SetUps {
            file: Bytes::new([0; SETUP_FILE_LEN]),
            names: Bytes::new([0; SLOT_COUNT as usize * NAME_LEN]),
            kept: Dword::new(0),
        }
    }
}

/// IDA's address of the file name's template.
pub const TEMPLATE_START: usize = 0x529716;

/// `aSavedataCarset`: the set-up file's name, the number's two digits last.
pub const fn template() -> Bytes<27> {
    Bytes::new(*b"savedata\\carsetup\\carset00\0")
}

// "CRSU"
pub fn slot_name(slot: u32) -> crate::place::BytePlace {
    crate::place::BytePlace::In(&service().set_ups.names, slot as usize * NAME_LEN)
}

// ---- Rust version ----

/// True if the slot's file was read and accepted.
pub fn read_setup_slot(service: &'static Service, slot: u32) -> Result<bool, Refused> {
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    {
        let template = service.template.read();
        let path = numbered_path_bytes(&template[..template.len() - 1], slot)?;
        let file = &service.set_ups.file;
        let mut bytes = file.read();
        let read = read_file_into(&path[..], &mut bytes[..SETUP_FILE_LEN], 0)?;
        file.write(&bytes);
        // Converted: `rc2k_core::saves::kept::setup_name`.
        let name = rc2k_core::saves::kept::setup_name(&file.read());
        match name {
            Some(name) if read.ok => {
                crate::place::BytePlace::In(&service.set_ups.names, slot as usize * NAME_LEN)
                    .write(&name[..NAME_LEN], "a setup's name")?;
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
        crate::gfx::first_values_are_the_games(
            &crate::service::owner::SERVICE.set_ups,
            START,
            START + LEN,
        );
        crate::gfx::first_values_are_the_games(
            &crate::service::owner::SERVICE.template,
            TEMPLATE_START,
            TEMPLATE_START + 27,
        );
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(SetUps, names), 0x527880 - START);
        assert_eq!(offset_of!(SetUps, kept), 0x528500 - START);
        assert_eq!(
            crate::moved::place(START),
            crate::service::owner::SERVICE.set_ups.file.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x527880),
            crate::service::owner::SERVICE.set_ups.names.addr() as usize
        );
        assert_eq!(
            crate::moved::place(TEMPLATE_START),
            crate::service::owner::SERVICE.template.addr() as usize
        );
    }
}
