//! The entry of the loaded track pack that a record refers to.
//!
//! The record holds the entry's number at +4 and the track pack's id at +8. The entries are 0x40
//! bytes each and start 0x34 bytes into the pack header. A record of another track pack is
//! fatal error "Invalid TrakPak Header", whatever the header's link (a slip kept as is).
//!
//! The entry's number comes from the player's files and has no bound: past the pack's 36
//! entries `Track::get` reads field by name on into what follows the pack, along the stage
//! tables' chain (`stages_store::Piece`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::records::stages_store::{Piece, RecordRef, StageTables, TrackEntry, TrackField};

fn loaded_pack_id(stages: &StageTables) -> u32 {
    stages.trak_pak.pack_id.get()
}

/// Reported when a record belongs to another track pack.
const INVALID_HEADER: FatalError = FatalError {
    code: 0x8070_DA00,
    message: "Invalid TrakPak Header",
};

/// The entry numbers after which the 32-bit byte offset (0x40 bytes an entry) wraps: kept,
/// since the numbers come from the player's files.
const ENTRIES_IN_ADDRESS_SPACE: u32 = 1 << 26;

/// A record's track: the entry of the loaded pack by its number, in the pack or past it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Track {
    /// The entry's number, wrapped as the 32-bit byte offset wrapped.
    index: u32,
}

impl Track {
    /// The typed entry, when the number is one of the pack's 36.
    pub fn entry(self, stages: &StageTables) -> Option<&TrackEntry> {
        stages.trak_pak.tracks.get(self.index as usize)
    }

    /// Field `field`: the entry's own in the pack; past the pack, the dword reached there along
    /// the tables (refused only past the tables' end).
    pub fn get(self, stages: &StageTables, field: TrackField) -> Result<u32, BadInput> {
        match self.entry(stages) {
            Some(entry) => Ok(entry.get(field)),
            None => stages
                .word_on(
                    Piece::Tracks,
                    self.index as usize * TrackEntry::WORDS + field.word(),
                )
                .ok_or(BadInput("a track entry's field past the stage tables")),
        }
    }

    /// The track's name (a string id).
    pub fn name(self, stages: &StageTables) -> Result<u32, BadInput> {
        self.get(stages, TrackField::Name)
    }

    /// The stage's length.
    pub fn length(self, stages: &StageTables) -> Result<u32, BadInput> {
        self.get(stages, TrackField::Length)
    }

    /// The stage number.
    pub fn stage(self, stages: &StageTables) -> Result<u32, BadInput> {
        self.get(stages, TrackField::Stage)
    }
}

/// The track entry for `record` (+4, +8 the pack's id), in the pack the game has loaded;
/// refused for the end mark, fatal for a record of another pack.
pub fn track_of(stages: &StageTables, record: RecordRef) -> Result<Track, Refused> {
    let record = record.record(stages)?;
    if record.pack_id() != loaded_pack_id(stages) {
        return Err(INVALID_HEADER.into());
    }
    Ok(Track {
        index: record.track() % ENTRIES_IN_ADDRESS_SPACE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::stages_store::RecordTable;

    /// Byte offsets of the pack's entries and of the end of the stage tables in the old image.
    const ENTRIES: usize = 0x734BFA;
    const END: usize = 0x73B674;

    /// The records' track numbers come from the player's files: every number reads its
    /// fields along the tables (in the pack, the typed entry) up to the old image's end, where
    /// the old byte-offset read stopped (a dword ending past `END`), and the first past the end
    /// is refused.
    #[test]
    fn track_numbers_run_on_to_the_end_of_the_tables() {
        let stages = Box::new(StageTables::new());
        let r = RecordTable::Championship.row(&stages, 2).unwrap();
        let mut refused = 0;
        for index in 0..=(END - ENTRIES) / 0x40 + 1 {
            r.record(&stages).unwrap().track.set(index as u32);
            let track = track_of(&stages, r).expect("the pack's id is the loaded one");
            for field in [TrackField::Name, TrackField::Length, TrackField::Stage] {
                let address = ENTRIES + 0x40 * index + 4 * field.word();
                let read = track.get(&stages, field);
                assert_eq!(read.is_ok(), address + 4 <= END, "{index} {field:?}");
                if let Ok(word) = read {
                    let k = (address - ENTRIES) / 4;
                    assert_eq!(Some(word), stages.word_on(Piece::Tracks, k));
                }
                refused += usize::from(read.is_err());
            }
            assert_eq!(track.entry(&stages).is_some(), index < 36);
        }
        assert!(refused >= 1);
        // The 32-bit address wraps: 2^26 entries on is the same entry.
        r.record(&stages)
            .unwrap()
            .track
            .set(ENTRIES_IN_ADDRESS_SPACE + 3);
        let track = track_of(&stages, r).unwrap();
        assert_eq!(
            track.name(&stages),
            Ok(stages.trak_pak.tracks[3].name.get())
        );
        // Another pack's record is a fatal error.
        r.record(&stages).unwrap().pack_id.set(7);
        assert!(track_of(&stages, r).is_err());
    }
}
