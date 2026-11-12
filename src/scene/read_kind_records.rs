//! `scene::read_kind_records` (0x439975): reads the four files of the object kinds'
//! records.
//!
//! A kind's record is 0x80 bytes at `g_objectMaterials` + kind * 0x80, and the four files fill
//! the four ranges of kinds, each with `files::read_whole_file`: `files\a\objsinfo` kinds 0 ..
//! 0x18F, `files\b\carsinfo` 0x190 .. 0x257, `files\g\frnt\frntinfo` 0x258 .. 0x31F,
//! `files\s\extr\extrinfo` 0x320 ... The original reads them in the order a, g, b, s; a file
//! that is missing leaves its range as it is. One caller, `scene::set_up_object_memory`.

use crate::errors::Refused;
use crate::files::read_whole_file::read_whole_file_store;
use crate::scene::file_numbers_store::texts;
use crate::scene::polygon_store::PolygonTables;
use crate::scene::set_object_material::MATERIAL_LEN;
use crate::session::TextPlace;

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
        path: texts::OBJECTS_INFO, // an IDA address in `path_texts`
        first_kind: 0,
        kinds: 0x190,
    },
    KindFile {
        path: texts::FRONT_INFO, // an IDA address in `path_texts`
        first_kind: 0x258,
        kinds: 0xC8,
    },
    KindFile {
        path: texts::CARS_INFO, // an IDA address in `path_texts`
        first_kind: 0x190,
        kinds: 0xC8,
    },
    KindFile {
        path: texts::EXTRAS_INFO, // an IDA address in `path_texts`
        first_kind: 0x320,
        kinds: 0x190,
    },
];

/// Where the file's records lie in the polygon tables, and their length.
fn records(file: &KindFile) -> (usize, usize) {
    (
        core::mem::offset_of!(PolygonTables, object_materials) + file.first_kind * MATERIAL_LEN,
        file.kinds * MATERIAL_LEN,
    )
}

// ---- Rust version ----

pub fn read_kind_records(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    for file in &FILES {
        let (at, room) = records(file);
        read_whole_file_store(
            &scene
                .file_numbers
                .text_at(file.path)
                .text("a kinds file's path")?,
            race.polygon_tables(),
            at,
            room,
        )?;
    }
    Ok(())
}
