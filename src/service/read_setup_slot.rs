//! Reads the head of a saved car setup, `savedata\carsetup\carsetNN`, into the set-ups' buffer
//! and copies its name into the slot name table.
//!
//! `slot` is 0 .. 99. It reads 0x48 bytes. The file is accepted if the read did not fail, the
//! first dword is "CRSU" and the dword at +8 is 0x50000; the 0x20 bytes at +0x10 then go to the
//! set-ups' names (`names[slot * 0x20]`). A file shorter than 0x48 bytes is not noticed as
//! such: what the buffer held before decides (kept). Refuses a slot past the table's 100
//! entries. Called for all 100 slots at start-up and by `load_setup_slot`.

use crate::errors::{BadInput, Refused};
use crate::files::numbered_path::numbered_path_bytes;
use crate::files::read_file::read_file_into;
use crate::memory::session::{PlainBytes, PlainDword};
use crate::service::owner::Service;

pub(super) const SETUP_FILE_LEN: usize = 0x48;
pub const SLOT_COUNT: u32 = 100;
pub const NAME_LEN: usize = 0x20;

/// The car set-ups' buffers.
#[derive(rc2k_derive::Cells)]
pub struct SetUps {
    /// The head of a set-up file as it is read or written.
    pub file: PlainBytes<SETUP_FILE_LEN>,
    /// The set-ups' names, 100 of 0x20 bytes.
    pub names: PlainBytes<{ SLOT_COUNT as usize * NAME_LEN }>,
    /// A value the options file keeps; nothing else reads it.
    pub kept: PlainDword,
}

/// The set-ups' buffers, all zero at first.
impl SetUps {
    pub const fn new() -> Self {
        SetUps {
            file: PlainBytes::new([0; SETUP_FILE_LEN]),
            names: PlainBytes::new([0; SLOT_COUNT as usize * NAME_LEN]),
            kept: PlainDword::new(0),
        }
    }
}

/// The set-up file's name, the number's two digits last.
pub const fn template() -> PlainBytes<27> {
    PlainBytes::new(*b"savedata\\carsetup\\carset00\0")
}

pub fn slot_name(service: &'static Service, slot: u32) -> crate::memory::place::BytePlace {
    crate::memory::place::BytePlace::In(&service.set_ups.names, slot as usize * NAME_LEN)
}

/// True if the slot's file was read and accepted.
pub fn read_setup_slot(service: &Service, slot: u32) -> Result<bool, Refused> {
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
        let name = rc2k_core::saves::kept::setup_name(&file.read());
        match name {
            Some(name) if read.ok => {
                for (k, &v) in name[..NAME_LEN].iter().enumerate() {
                    service.set_ups.names.set(slot as usize * NAME_LEN + k, v);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
