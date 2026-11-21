//! The replay's staging: the replay file's frame (`ReplayFrame`, 0x2810 bytes) as the race
//! keeps it while recording and playing. The starting state (the four cars, the loose parts and
//! the computer driver's state, in the file's order) is written by `drive::save_replay_cars`
//! and read by `drive::load_replay_cars` and the replay check; the tape holds the input words
//! of each of a frame's hundred tenths for the two cars on it (`drive::record_inputs`,
//! `drive::play_inputs`). The frame is copied whole to and from the viewer's block
//! (`drive::flush_replay`, `drive::load_replay_frame`, `race::show_replay_still`).
//!
//! Plain dwords in the file's order, not a `ReplayFrame` value: the tape is read and written one
//! entry at a time on every tick, the starting state only once a frame, and the frame is copied
//! to and from the block as dwords.

use crate::errors::BadInput;
use crate::memory::session::PlainDword;
use rc2k_core::drive::replay_frames::{
    EXTRA_TAPE_BYTES, EXTRA_TAPE_CARS, FRAME_BYTES, ReplayFrame, START_DWORDS, TAPE_CARS, TENTHS,
    TapePlace, tape_place,
};

/// What the tape holds of one car at one tenth: 8 bytes.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TapeEntry {
    /// The input word `input_words[1]` of the tenth (`drive::record_inputs`).
    pub input: u32,
    /// The other three input words packed in one (`drive::pack_inputs`).
    pub packed: u32,
}

/// The replay's frame as the race keeps it.
#[repr(C)]
pub struct ReplayStaging {
    /// The starting state, `START_DWORDS` dwords (`rc2k_core::drive::replay_frames`).
    pub start: [u32; START_DWORDS],
    /// The tape: for each tenth of the frame, the entry of car 0 and of car 1.
    pub tape: [[TapeEntry; TAPE_CARS as usize]; TENTHS as usize],
    /// The extra tape of a replay with it (3 and 4 player races): for each tenth, the entry of
    /// car 2 and of car 3. It follows the frame in the file's block.
    pub extra_tape: [[TapeEntry; EXTRA_TAPE_CARS as usize]; TENTHS as usize],
}

const _: () = assert!(core::mem::size_of::<ReplayStaging>() == FRAME_BYTES + EXTRA_TAPE_BYTES);

const NO_ENTRY: TapeEntry = TapeEntry {
    input: 0,
    packed: 0,
};

impl ReplayStaging {
    /// All zero, as the game's memory starts.
    pub const fn new() -> Self {
        ReplayStaging {
            start: [0; START_DWORDS],
            tape: [[NO_ENTRY; TAPE_CARS as usize]; TENTHS as usize],
            extra_tape: [[NO_ENTRY; EXTRA_TAPE_CARS as usize]; TENTHS as usize],
        }
    }

    /// Every dword in the file's order: the starting state, the tape, the extra tape.
    pub fn words(&self) -> Vec<u32> {
        let entry_words = |e: &TapeEntry| [e.input, e.packed];
        self.start
            .iter()
            .copied()
            .chain(self.tape.iter().flatten().flat_map(entry_words))
            .chain(self.extra_tape.iter().flatten().flat_map(entry_words))
            .collect()
    }

    /// The bytes of the whole staging, in the file's order.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.words()
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect()
    }

    /// `words` put in the frame (the starting state, then the tape) from its first dword.
    pub fn set_frame_words(&mut self, words: impl IntoIterator<Item = u32>) {
        let mut words = words.into_iter();
        for slot in &mut self.start {
            let Some(word) = words.next() else { return };
            *slot = word;
        }
        for entry in self.tape.iter_mut().flatten() {
            let Some(input) = words.next() else { return };
            entry.input = input;
            let Some(packed) = words.next() else { return };
            entry.packed = packed;
        }
    }

    /// `words` put in the extra tape from its first dword.
    pub fn set_extra_tape_words(&mut self, words: impl IntoIterator<Item = u32>) {
        let mut words = words.into_iter();
        for entry in self.extra_tape.iter_mut().flatten() {
            let Some(input) = words.next() else { return };
            entry.input = input;
            let Some(packed) = words.next() else { return };
            entry.packed = packed;
        }
    }

    /// The starting state put in (the tape is left).
    pub fn set_start(&mut self, frame: &ReplayFrame) {
        for (slot, bytes) in self
            .start
            .iter_mut()
            .zip(frame.start_bytes().as_chunks::<4>().0)
        {
            *slot = u32::from_le_bytes(*bytes);
        }
    }

    /// The starting state, the tape zero.
    pub fn start_frame(&self) -> ReplayFrame {
        let bytes: Vec<u8> = self.start.iter().flat_map(|w| w.to_le_bytes()).collect();
        ReplayFrame::from_start_bytes(&bytes).expect("the starting state's size")
    }

    /// The starting state's dword `n`, refused past it.
    pub fn start_word(&self, n: usize) -> Result<u32, BadInput> {
        self.start
            .get(n)
            .copied()
            .ok_or(BadInput("a dword past the replay's starting state"))
    }

    /// The tape's entry of `car` at `tenth`, refused past the frame's hundred tenths or the cars
    /// on the tape (two; four when `extended`, a replay with the extra tape).
    pub fn entry(&self, tenth: u32, car: u32, extended: bool) -> Result<TapeEntry, BadInput> {
        self.entry_ref(tenth, car, extended).copied()
    }

    /// The tape's entry of `car` at `tenth` set, refused as `entry`.
    pub fn set_entry(
        &mut self,
        tenth: u32,
        car: u32,
        extended: bool,
        entry: TapeEntry,
    ) -> Result<(), BadInput> {
        match tape_place(tenth, car, extended)
            .ok_or(BadInput("a tape entry past the frame's tenths"))?
        {
            TapePlace::Main(tenth, car) => self.tape[tenth][car] = entry,
            TapePlace::Extra(tenth, car) => self.extra_tape[tenth][car] = entry,
        }
        Ok(())
    }

    fn entry_ref(&self, tenth: u32, car: u32, extended: bool) -> Result<&TapeEntry, BadInput> {
        match tape_place(tenth, car, extended)
            .ok_or(BadInput("a tape entry past the frame's tenths"))?
        {
            TapePlace::Main(tenth, car) => Ok(&self.tape[tenth][car]),
            TapePlace::Extra(tenth, car) => Ok(&self.extra_tape[tenth][car]),
        }
    }

    /// The extra tape zero (a recording with it starts from nothing).
    pub fn clear_extra_tape(&mut self) {
        self.extra_tape = [[NO_ENTRY; EXTRA_TAPE_CARS as usize]; TENTHS as usize];
    }
}

/// The replay block's frame (and with the extra tape, that too) copied into the staging.
pub fn load_from_block(race: &crate::race::owner::Race) {
    let buffers = race.viewer_buffers();
    let mut staging = race.replay.staging_mut();
    staging.set_frame_words(buffers.block.iter().map(PlainDword::get));
    if buffers.extra_tape.is_set() {
        staging.set_extra_tape_words(buffers.block_extra.iter().map(PlainDword::get));
    }
    drop(staging);
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
    let buffers = race.viewer_buffers();
    let words = race.replay.staging().words();
    let (frame, extra) = words.split_at(FRAME_BYTES / 4);
    for (cell, &word) in buffers.block.iter().zip(frame) {
        cell.set(word);
    }
    if buffers.extra_tape.is_set() {
        for (cell, &word) in buffers.block_extra.iter().zip(extra) {
            cell.set(word);
        }
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
    use rc2k_core::drive::replay_frames::{CAR_DWORDS, EXTRA_TAPE_DWORDS, TAPE_DWORDS};

    fn entry(input: u32, packed: u32) -> TapeEntry {
        TapeEntry { input, packed }
    }

    #[test]
    fn fields_sit_at_the_files_offsets() {
        let mut staging = ReplayStaging::new();
        staging.set_entry(3, 1, false, entry(0, 7)).unwrap();
        let bytes = staging.to_bytes();
        let frame = ReplayFrame::from_bytes(&bytes[..FRAME_BYTES]).unwrap();
        // Tenth 3, car 1, second word: dword 3 * 4 + 1 * 2 + 1 of the tape.
        assert_eq!(frame.tape[15], 7);
        assert_eq!(frame.tape.iter().filter(|&&d| d != 0).count(), 1);
        assert!(staging.entry(100, 0, false).is_err());
        assert!(staging.entry(0, 2, false).is_err());
        assert_eq!(TAPE_DWORDS, 2 * TAPE_CARS as usize * TENTHS as usize);
    }

    #[test]
    fn the_extra_tape_follows_the_frame() {
        let mut staging = ReplayStaging::new();
        // Tenth 3, car 3: dword 3 * 4 + 1 * 2 + 1 of the extra tape.
        staging.set_entry(3, 3, true, entry(0, 9)).unwrap();
        staging.set_entry(0, 2, true, entry(5, 0)).unwrap();
        assert!(staging.entry(0, 2, false).is_err());
        assert!(staging.entry(0, 4, true).is_err());
        let bytes = staging.to_bytes();
        assert_eq!(bytes.len(), FRAME_BYTES + EXTRA_TAPE_BYTES);
        let dword =
            |n: usize| u32::from_le_bytes(bytes[FRAME_BYTES + 4 * n..][..4].try_into().unwrap());
        assert_eq!(dword(15), 9);
        assert_eq!(dword(0), 5);
        assert_eq!((0..400).filter(|&n| dword(n) != 0).count(), 2);
        // The frame's own tape is not touched.
        assert!(bytes[..FRAME_BYTES].iter().all(|&b| b == 0));
        staging.clear_extra_tape();
        assert!(staging.to_bytes().iter().all(|&b| b == 0));
    }

    #[test]
    fn words_go_in_and_come_out_in_order() {
        let mut staging = ReplayStaging::new();
        let total = FRAME_BYTES / 4 + EXTRA_TAPE_DWORDS;
        staging.set_frame_words(0..(FRAME_BYTES / 4) as u32);
        staging.set_extra_tape_words(1000..1000 + EXTRA_TAPE_DWORDS as u32);
        let words = staging.words();
        assert_eq!(words.len(), total);
        assert!(
            words[..FRAME_BYTES / 4]
                .iter()
                .copied()
                .eq(0..(FRAME_BYTES / 4) as u32)
        );
        assert!(words[FRAME_BYTES / 4..].iter().copied().eq(1000..1400));
    }

    #[test]
    fn the_starting_state_round_trips() {
        let mut staging = ReplayStaging::new();
        let mut frame = ReplayFrame::new();
        frame.cars[2][CAR_DWORDS - 1] = 0xCAFE;
        frame.extra[5] = 9;
        staging.set_start(&frame);
        assert_eq!(staging.start_frame(), frame);
        assert_eq!(staging.start_word(3 * CAR_DWORDS - 1).unwrap(), 0xCAFE);
        assert!(staging.start_word(START_DWORDS).is_err());
    }
}
