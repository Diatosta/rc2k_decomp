//! `scene::read_kind_records` (0x439975): reads the four files of the object kinds'
//! records.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! A kind's record is 0x80 bytes at `g_objectMaterials` + kind * 0x80, and the four files fill
//! the four ranges of kinds, each with `files::read_whole_file`: `files\a\objsinfo` kinds 0 ..
//! 0x18F, `files\b\carsinfo` 0x190 .. 0x257, `files\g\frnt\frntinfo` 0x258 .. 0x31F,
//! `files\s\extr\extrinfo` 0x320 ... The original reads them in the order a, g, b, s; a file
//! that is missing leaves its range as it is. One caller, `scene::set_up_object_memory`.

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file;
use crate::framework::{game_addr, game_ptr, rva};
use crate::scene::set_object_material::{MATERIAL_LEN, MATERIALS_RVA};
use crate::service::harness::Region;

// ---- game data ----

/// A file of records: its path, the first kind it holds and how many kinds.
struct KindFile {
    path: usize,
    first_kind: usize,
    kinds: usize,
}

/// In the order they are read.
const FILES: [KindFile; 4] = [
    KindFile {
        path: rva(0x60AE69), // "files\a\objsinfo"
        first_kind: 0,
        kinds: 0x190,
    },
    KindFile {
        path: rva(0x60AEA1), // "files\g\frnt\frntinfo"
        first_kind: 0x258,
        kinds: 0xC8,
    },
    KindFile {
        path: rva(0x60AE7A), // "files\b\carsinfo"
        first_kind: 0x190,
        kinds: 0xC8,
    },
    KindFile {
        path: rva(0x60AE8B), // "files\s\extr\extrinfo"
        first_kind: 0x320,
        kinds: 0x190,
    },
];

fn records(file: &KindFile) -> Region {
    (
        (game_addr(MATERIALS_RVA) + file.first_kind * MATERIAL_LEN) as u32,
        file.kinds * MATERIAL_LEN,
    )
}

// ---- Rust version ----

pub unsafe fn read_kind_records() -> Result<(), Refused> {
    for file in &FILES {
        let (to, room) = records(file);
        unsafe { read_whole_file(game_ptr(file.path), to as *mut u8, room as u32) }?;
    }
    Ok(())
}
