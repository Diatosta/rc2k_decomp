//! The replay's staging: the replay file's frame (`ReplayFrame`, 0x2810 bytes) as the race
//! keeps it while recording and playing. The starting state (the four cars, the loose parts and
//! the computer driver's state, in the file's order) is written by `drive::save_replay_cars`
//! and read by `drive::load_replay_cars` and the replay check; the tape holds the input words
//! of each of a frame's hundred tenths for the two cars on it (`drive::record_inputs`,
//! `drive::play_inputs`). The frame is copied whole to and from the viewer's block
//! (`drive::flush_replay`, `drive::load_replay_frame`, `race::show_replay_still`).
//!
//! Cells over the file's dwords, not a `ReplayFrame` value: the tape is read and written one
//! entry at a time on every tick, the starting state only once a frame, and a cell array is
//! copied to the block by the same `copy_cells` as any other store.

use crate::errors::BadInput;
use crate::session::PlainDword;
use rc2k_core::drive::replay_frames::{
    FRAME_BYTES, ReplayFrame, START_DWORDS, TAPE_CARS, TENTHS, tape_slot,
};

/// What the tape holds of one car at one tenth: 8 bytes.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct TapeEntry {
    /// The input word `input_words[1]` of the tenth (`drive::record_inputs`).
    pub input: PlainDword,
    /// The other three input words packed in one (`drive::pack_inputs`).
    pub packed: PlainDword,
}

/// The replay's frame as the race keeps it.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct ReplayStaging {
    /// The starting state, `START_DWORDS` dwords (`rc2k_core::drive::replay_frames`).
    pub start: [PlainDword; START_DWORDS],
    /// The tape: for each tenth of the frame, the entry of car 0 and of car 1.
    pub tape: [[TapeEntry; TAPE_CARS as usize]; TENTHS as usize],
}

const _: () = assert!(core::mem::size_of::<ReplayStaging>() == FRAME_BYTES);

impl ReplayStaging {
    /// All zero, as the game's memory starts.
    pub const fn new() -> Self {
        crate::cells::zeroed_cells()
    }

    /// The starting state put in (the tape is left).
    pub fn set_start(&self, frame: &ReplayFrame) {
        crate::cells::store_cells(&self.start, 0, &frame.start_bytes())
            .expect("the starting state inside the staging");
    }

    /// The starting state, the tape zero.
    pub fn start_frame(&self) -> ReplayFrame {
        let mut bytes = vec![0u8; 4 * START_DWORDS];
        crate::cells::load_cells(&self.start, 0, &mut bytes)
            .expect("the starting state inside the staging");
        ReplayFrame::from_start_bytes(&bytes).expect("the starting state's size")
    }

    /// The starting state's dword `n`, refused past it.
    pub fn start_word(&self, n: usize) -> Result<u32, BadInput> {
        self.start
            .get(n)
            .map(PlainDword::get)
            .ok_or(BadInput("a dword past the replay's starting state"))
    }

    /// The tape's entry of `car` at `tenth`, refused past the frame's hundred tenths or the
    /// two cars.
    pub fn entry(&self, tenth: u32, car: u32) -> Result<&TapeEntry, BadInput> {
        let (tenth, car) =
            tape_slot(tenth, car).ok_or(BadInput("a tape entry past the frame's tenths"))?;
        Ok(&self.tape[tenth][car])
    }
}

impl Default for ReplayStaging {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::drive::replay_frames::{CAR_DWORDS, TAPE_DWORDS};

    #[test]
    fn fields_sit_at_the_files_offsets() {
        let staging = ReplayStaging::new();
        staging.entry(3, 1).unwrap().packed.set(7);
        let mut bytes = vec![0u8; FRAME_BYTES];
        crate::cells::load_cells(&staging, 0, &mut bytes).unwrap();
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        // Tenth 3, car 1, second word: dword 3 * 4 + 1 * 2 + 1 of the tape.
        assert_eq!(frame.tape[15], 7);
        assert_eq!(frame.tape.iter().filter(|&&d| d != 0).count(), 1);
        assert!(staging.entry(100, 0).is_err());
        assert!(staging.entry(0, 2).is_err());
        assert_eq!(TAPE_DWORDS, 2 * TAPE_CARS as usize * TENTHS as usize);
    }

    #[test]
    fn the_starting_state_round_trips() {
        let staging = ReplayStaging::new();
        let mut frame = ReplayFrame::new();
        frame.cars[2][CAR_DWORDS - 1] = 0xCAFE;
        frame.extra[5] = 9;
        staging.set_start(&frame);
        assert_eq!(staging.start_frame(), frame);
        assert_eq!(staging.start_word(3 * CAR_DWORDS - 1).unwrap(), 0xCAFE);
        assert!(staging.start_word(START_DWORDS).is_err());
    }
}
