//! `scene::read_kind_records`: reads the four files of the object kinds' records into a
//! `KindRecords` value.
//!
//! The files are `files\a\objsinfo` (kinds 0 .. 0x18F), `files\b\carsinfo` (0x190 .. 0x257),
//! `files\g\frnt\frntinfo` (0x258 .. 0x31F) and `files\s\extr\extrinfo` (0x320 ..), read in the
//! order a, g, b, s. A file that is missing or short leaves its range as the records held it.
//! The caller stores the result (`scene::set_up_object_memory`).

use crate::errors::{BadInput, Refused};
use crate::files::read_whole_file::read_whole_file_len;
use crate::memory::session::TextPlace;
use crate::scene::file_numbers_store::texts;
use rc2k_core::scene::kind_records::{KINDS, KindRecords, RANGES, RECORD_LEN};

/// The files' path texts, in the order of `RANGES`.
const PATHS: [usize; 4] = [
    texts::OBJECTS_INFO,
    texts::FRONT_INFO,
    texts::CARS_INFO,
    texts::EXTRAS_INFO,
];

/// The records the polygon tables hold, with each file laid over its kinds.
pub fn read_kind_records(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<KindRecords, Refused> {
    let held: Vec<u8> = race.polygon_tables().read(|tables| {
        tables
            .object_materials
            .iter()
            .flat_map(|record| record.words.iter().flat_map(|word| word.to_le_bytes()))
            .collect()
    });
    let mut records = KindRecords::from_held(&held)?;
    for (range, path) in RANGES.into_iter().zip(PATHS) {
        let mut file = vec![0u8; range.kinds * RECORD_LEN];
        let read = read_whole_file_len(
            &scene.path_text(path).text("a kinds file's path")?,
            &mut file,
        )?;
        records.overlay(range, &file[..read]);
    }
    Ok(records)
}

/// Stores the records in the polygon tables.
pub fn store_kind_records(
    race: &crate::race::owner::Race,
    records: &KindRecords,
) -> Result<(), Refused> {
    if records.bytes().len() != KINDS * RECORD_LEN {
        return Err(BadInput("the kind records are not the table's size").into());
    }
    let mut tables = race.shared.buffers.polygon_tables.borrow_mut();
    let words = tables
        .object_materials
        .iter_mut()
        .flat_map(|record| record.words.iter_mut());
    for (word, bytes) in words.zip(records.bytes().as_chunks::<4>().0) {
        *word = u32::from_le_bytes(*bytes);
    }
    Ok(())
}
