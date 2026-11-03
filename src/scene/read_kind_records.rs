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
use crate::pointers::GamePtr;
use crate::scene::file_numbers_store::{FILE_NUMBERS, texts};
use crate::scene::set_object_material::MATERIAL_LEN;

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
        path: texts::OBJECTS_INFO, // an IDA address in `unknown_14`
        first_kind: 0,
        kinds: 0x190,
    },
    KindFile {
        path: texts::FRONT_INFO, // an IDA address in `unknown_14`
        first_kind: 0x258,
        kinds: 0xC8,
    },
    KindFile {
        path: texts::CARS_INFO, // an IDA address in `unknown_14`
        first_kind: 0x190,
        kinds: 0xC8,
    },
    KindFile {
        path: texts::EXTRAS_INFO, // an IDA address in `unknown_14`
        first_kind: 0x320,
        kinds: 0x190,
    },
];

fn records(file: &KindFile) -> (GamePtr, usize) {
    (
        crate::scene::polygon::materials_ptr().offset((file.first_kind * MATERIAL_LEN) as u32),
        file.kinds * MATERIAL_LEN,
    )
}

// ---- Rust version ----

pub fn read_kind_records() -> Result<(), Refused> {
    for file in &FILES {
        let (to, room) = records(file);
        read_whole_file(FILE_NUMBERS.text_ptr(file.path), to, room as u32)?;
    }
    Ok(())
}
