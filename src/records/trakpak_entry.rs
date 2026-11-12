//! `records::trakpak_entry` (0x473027): returns the entry of the loaded track pack that a
//! record refers to.
//!
//! Takes the record (dword +4 = entry index, dword +8 = track pack id). Returns the entry, the
//! index * 0x40, the record's pack id and the loaded pack's id. The entries are 0x40 bytes each and
//! start 0x34 bytes into the pack header. If the record's pack id is not the loaded pack's, the
//! original reports "Invalid TrakPak Header" or, when the header's first dword is non-zero, loops
//! forever (it reloads the header address on every pass instead of following the link): that case
//! is left to the original.

use crate::errors::BadInput;
use crate::errors::{FatalError, Refused};
use crate::records::stages_store::{RecordRef, TrackEntry};

// ---- game data ----

pub fn loaded_pack_id(stages: &crate::records::stages_store::StageTables) -> u32 {
    stages.trak_pak.pack_id.get()
}

/// What the original reports when a record belongs to another track pack.
pub const INVALID_HEADER: FatalError = FatalError {
    code: 0x8070_DA00,
    message: "Invalid TrakPak Header",
};

/// Entry `index` in the pack header at byte `header` of the stage tables, as its byte offset
/// there.
/// The original loops forever instead of reporting the error when the header's first dword
/// is non-zero; this version reports it in both cases.
pub fn trakpak_entry(
    header: usize,
    index: u32,
    pack_id: u32,
    loaded_pack_id: u32,
) -> Result<usize, FatalError> {
    // Converted: `rc2k_core::files::records::pack_entry`.
    rc2k_core::files::records::pack_entry(index, pack_id, loaded_pack_id)
        .map(|place| header + place)
        .ok_or(INVALID_HEADER)
}

/// The entry for `record`'s track (+4, +8 the pack's id), in the pack the game has loaded.
///
/// The offset is bounded to the stage tables as a whole (the original's index has no bound
/// and reads on past the pack into what follows).
pub fn record_entry(
    stages: &crate::records::stages_store::StageTables,
    record: RecordRef,
) -> Result<usize, Refused> {
    let record = record.record()?;
    Ok(trakpak_entry(
        core::mem::offset_of!(crate::records::stages_store::StageTables, trak_pak),
        record.track(),
        record.pack_id(),
        loaded_pack_id(stages),
    )?)
}

/// A record's track entry: typed when it lies in the pack's 36 entries; past them the
/// original's index runs on into what follows the pack (`record_entry`), read as the words
/// they are.
pub enum Track<'a> {
    InPack(&'a TrackEntry),
    /// A run-on past the pack, as the original.
    /// The entry's byte offset in the stage tables.
    RunOn(usize),
}

/// The dword at byte `at` of the stage tables, past the pack.
pub fn run_on(
    stages: &crate::records::stages_store::StageTables,
    at: usize,
) -> Result<u32, BadInput> {
    let mut bytes = [0; 4];
    crate::cells::load_cells(stages, at, &mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

impl Track<'_> {
    /// +0x00: the track's name (a string id).
    pub fn name(
        &self,
        stages: &crate::records::stages_store::StageTables,
    ) -> Result<u32, BadInput> {
        match self {
            Track::InPack(entry) => Ok(entry.name.get()),
            Track::RunOn(at) => run_on(stages, *at + core::mem::offset_of!(TrackEntry, name)),
        }
    }

    /// +0x0C: the stage's length.
    pub fn length(
        &self,
        stages: &crate::records::stages_store::StageTables,
    ) -> Result<u32, BadInput> {
        match self {
            Track::InPack(entry) => Ok(entry.length.get()),
            Track::RunOn(at) => run_on(stages, *at + core::mem::offset_of!(TrackEntry, length)),
        }
    }

    /// +0x34: the stage number.
    pub fn stage(
        &self,
        stages: &crate::records::stages_store::StageTables,
    ) -> Result<u32, BadInput> {
        match self {
            Track::InPack(entry) => Ok(entry.stage.get()),
            Track::RunOn(at) => run_on(stages, *at + core::mem::offset_of!(TrackEntry, stage)),
        }
    }
}

/// The track entry for `record`, with the refusals of `record_entry`.
pub fn track_of(
    stages: &crate::records::stages_store::StageTables,
    record: RecordRef,
) -> Result<Track<'_>, Refused> {
    let at = record_entry(stages, record)?;
    let index = record.record()?.track() as usize;
    Ok(match stages.trak_pak.tracks.get(index) {
        Some(entry) => Track::InPack(entry),
        None => Track::RunOn(at),
    })
}
