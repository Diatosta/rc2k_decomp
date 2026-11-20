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
use crate::memory::session::PlainDword;
use rc2k_core::drive::replay_frames::{
    EXTRA_TAPE_BYTES, EXTRA_TAPE_CARS, FRAME_BYTES, ReplayFrame, START_DWORDS, TAPE_CARS, TENTHS,
    TapePlace, tape_place,
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
    /// The extra tape of a replay with it (3 and 4 player races): for each tenth, the entry of
    /// car 2 and of car 3. It follows the frame in the file's block.
    pub extra_tape: [[TapeEntry; EXTRA_TAPE_CARS as usize]; TENTHS as usize],
}

const _: () = assert!(core::mem::size_of::<ReplayStaging>() == FRAME_BYTES + EXTRA_TAPE_BYTES);

impl ReplayStaging {
    /// All zero, as the game's memory starts.
    pub const fn new() -> Self {
        crate::memory::cells::zeroed_cells()
    }

    /// The starting state put in (the tape is left).
    pub fn set_start(&self, frame: &ReplayFrame) {
        crate::memory::cells::store_cells(&self.start, 0, &frame.start_bytes())
            .expect("the starting state inside the staging");
    }

    /// The starting state, the tape zero.
    pub fn start_frame(&self) -> ReplayFrame {
        let mut bytes = vec![0u8; 4 * START_DWORDS];
        crate::memory::cells::load_cells(&self.start, 0, &mut bytes)
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

    /// The tape's entry of `car` at `tenth`, refused past the frame's hundred tenths or the cars
    /// on the tape (two; four when `extended`, a replay with the extra tape).
    pub fn entry(&self, tenth: u32, car: u32, extended: bool) -> Result<&TapeEntry, BadInput> {
        match tape_place(tenth, car, extended)
            .ok_or(BadInput("a tape entry past the frame's tenths"))?
        {
            TapePlace::Main(tenth, car) => Ok(&self.tape[tenth][car]),
            TapePlace::Extra(tenth, car) => Ok(&self.extra_tape[tenth][car]),
        }
    }

    /// The extra tape zero (a recording with it starts from nothing).
    pub fn clear_extra_tape(&self) {
        for entry in self.extra_tape.iter().flatten() {
            entry.input.set(0);
            entry.packed.set(0);
        }
    }
}

/// The replay block's frame (and with the extra tape, that too) copied into the staging.
pub fn load_from_block(race: &crate::race::owner::Race) {
    use crate::memory::cells::copy_cells;
    let buffers = race.viewer_buffers();
    copy_cells((&buffers.block, 0), (race.replay.staging(), 0), FRAME_BYTES);
    if buffers.extra_tape.is_set() {
        copy_cells(
            (&buffers.block_extra, 0),
            (race.replay.staging(), FRAME_BYTES),
            EXTRA_TAPE_BYTES,
        );
    }
    if buffers.v2.is_set() {
        load_tail(race);
    }
}

/// The block's tail (a replay of the own format) read into the replay's tail.
fn load_tail(race: &crate::race::owner::Race) {
    use rc2k_core::drive::replay_frames::EXTENDED_FRAME_BYTES;
    use rc2k_core::race::replay_v2::Tail;
    let bytes = race.viewer_buffers().block_bytes();
    let head = race.replay.staging().start_frame().cars;
    let mut v2 = race.replay.v2().lock();
    let mask = v2.state_mask();
    v2.tail = Tail::from_bytes(&bytes[EXTENDED_FRAME_BYTES..], &head, mask).unwrap_or_else(|| {
        crate::memory::host_hooks::log("replay: a block's tail is damaged and is not played");
        Tail::new()
    });
}

/// The staging's frame (and with the extra tape, that too) copied into the replay block.
pub fn store_in_block(race: &crate::race::owner::Race) {
    use crate::memory::cells::copy_cells;
    let buffers = race.viewer_buffers();
    copy_cells((race.replay.staging(), 0), (&buffers.block, 0), FRAME_BYTES);
    if buffers.extra_tape.is_set() {
        copy_cells(
            (race.replay.staging(), FRAME_BYTES),
            (&buffers.block_extra, 0),
            EXTRA_TAPE_BYTES,
        );
    }
    if buffers.v2.is_set() {
        store_tail(race);
    }
}

/// The replay's tail put in the block's tail.
fn store_tail(race: &crate::race::owner::Race) {
    use rc2k_core::drive::replay_frames::EXTENDED_FRAME_BYTES;
    let head = race.replay.staging().start_frame().cars;
    let bytes = {
        let v2 = race.replay.v2().lock();
        v2.tail.to_bytes(&head, v2.state_mask())
    };
    if let Err(why) =
        race.viewer_buffers()
            .put_in_block(EXTENDED_FRAME_BYTES, &bytes, "the replay's tail")
    {
        crate::memory::host_hooks::log(&format!("replay: the tail is not kept: {why:?}"));
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
        staging.entry(3, 1, false).unwrap().packed.set(7);
        let mut bytes = vec![0u8; FRAME_BYTES];
        crate::memory::cells::load_cells(&staging, 0, &mut bytes).unwrap();
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        // Tenth 3, car 1, second word: dword 3 * 4 + 1 * 2 + 1 of the tape.
        assert_eq!(frame.tape[15], 7);
        assert_eq!(frame.tape.iter().filter(|&&d| d != 0).count(), 1);
        assert!(staging.entry(100, 0, false).is_err());
        assert!(staging.entry(0, 2, false).is_err());
        assert_eq!(TAPE_DWORDS, 2 * TAPE_CARS as usize * TENTHS as usize);
    }

    #[test]
    fn the_extra_tape_follows_the_frame() {
        let staging = ReplayStaging::new();
        // Tenth 3, car 3: dword 3 * 4 + 1 * 2 + 1 of the extra tape.
        staging.entry(3, 3, true).unwrap().packed.set(9);
        staging.entry(0, 2, true).unwrap().input.set(5);
        assert!(staging.entry(0, 2, false).is_err());
        assert!(staging.entry(0, 4, true).is_err());
        let mut bytes = vec![0u8; FRAME_BYTES + EXTRA_TAPE_BYTES];
        crate::memory::cells::load_cells(&staging, 0, &mut bytes).unwrap();
        let dword =
            |n: usize| u32::from_le_bytes(bytes[FRAME_BYTES + 4 * n..][..4].try_into().unwrap());
        assert_eq!(dword(15), 9);
        assert_eq!(dword(0), 5);
        assert_eq!((0..400).filter(|&n| dword(n) != 0).count(), 2);
        // The frame's own tape is not touched.
        assert!(bytes[..FRAME_BYTES].iter().all(|&b| b == 0));
        staging.clear_extra_tape();
        crate::memory::cells::load_cells(&staging, 0, &mut bytes).unwrap();
        assert!(bytes.iter().all(|&b| b == 0));
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
