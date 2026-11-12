//! The replay's frames (from `drive::keep_replay_frame` 0x45FFB6,
//! `drive::read_replay_frame` 0x41EBAB, `drive::write_replay_frames` 0x41E930,
//! `drive::flush_replay_if` 0x460031, `drive::record_inputs` 0x4601A8 and
//! `drive::play_inputs` 0x460162, their switches `play_inputs_if` 0x4600A9 and
//! `record_inputs_if` 0x46008D, the frame marks of `replay_frame_marks` 0x41D8A3; the
//! starting state's layout from
//! `drive::save_replay_cars` 0x4600B8 and `drive::load_replay_cars` 0x46010D): a frame is
//! a hundred tenths; at each whole frame the four car blocks are kept in one of eight
//! slots; the tape holds each tenth's input words, 16 bytes a tenth, 8 for each of the
//! two cars.

/// The replay's mode: 1 recording, 2 playing.
pub const RECORDING: u32 = 1;
pub const PLAYING: u32 = 2;

/// Tenths a frame.
pub const TENTHS: u32 = 100;

/// The frame and tenth of a car block's clock (+8).
pub fn frame_and_tenth(clock: u32) -> (u32, u32) {
    (clock / TENTHS, clock % TENTHS)
}

/// The kept slots of the four car blocks (0x3F80 bytes), by frame & 7.
pub const SLOT_SIZE: u32 = 0x3F80;
pub const SLOTS: u32 = 8;

/// A frame's slot's offset.
pub fn slot_offset(frame: u32) -> u32 {
    (frame & (SLOTS - 1)) * SLOT_SIZE
}

/// The highest frame read, and the fatal errors past it and past a frame's tenths.
pub const LAST_FRAME: u32 = 0x708;
pub const FRAME_PAST: u32 = 0x80A6_1151;
pub const TENTH_PAST: u32 = 0x80A6_1152;

/// `read_replay_frame`'s checks: the fatal error's code for a frame or tenth past its
/// end.
pub fn frame_fault(frame: u32, tenth: u32) -> Option<u32> {
    if frame > LAST_FRAME {
        Some(FRAME_PAST)
    } else if tenth > TENTHS {
        Some(TENTH_PAST)
    } else {
        None
    }
}

/// Whether a frame is at or past the replay's last: then the last is
/// shown, nothing read.
pub fn past_end(frame: u32, frames: u32) -> bool {
    frame >= frames
}

/// `write_replay_frames`' frame head's byte +5 (a reading: the view), held to 2 ..= 3
/// (signed).
pub fn written_view(byte: i8) -> i8 {
    byte.clamp(2, 3)
}

/// `flush_replay_if`: while recording at the last tenth of a frame.
pub fn flush_due(mode: u32, tenth: u32) -> bool {
    mode == RECORDING && tenth == TENTHS - 1
}

/// The cars on the tape.
pub const TAPE_CARS: u32 = 2;

/// Whether a car is on the tape.
pub fn on_tape(car: u32) -> bool {
    car < TAPE_CARS
}

/// A car's place on the tape at a tenth (16 bytes a tenth, 8 a car).
pub fn tape_offset(tenth: u32, car: u32) -> u32 {
    tenth.wrapping_mul(16).wrapping_add(car * 8)
}

/// `drive::play_inputs_if`: the inputs played while playing a replay.
pub fn plays_inputs(mode: u32) -> bool {
    mode == PLAYING
}

/// `drive::record_inputs_if`: the inputs recorded while recording, for the cars below
/// the race's count (`RaceView.players`; asked only while recording).
pub fn records_inputs(mode: u32, car: u32, cars: impl FnOnce() -> u32) -> Option<bool> {
    (mode == RECORDING).then(|| car < cars())
}

/// `drive::replay_frame_marks`: the offset of a frame's two marks (8 bytes a frame,
/// `Buffers.viewer.block_cameras`; the original's `shl`, high bits lost).
pub fn marks_offset(frame: u32) -> u32 {
    frame.wrapping_mul(8)
}

/// A piece of the replay's starting state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartPiece {
    /// The first `CAR_DWORDS` of car block k (`g_carBlockStore` + 0xFE0 k).
    Car(u32),
    /// The race slots (`g_raceSlots`).
    RaceSlots,
    /// `RaceWork.replay_extra`.
    Extra,
}

/// The dwords kept of each car block.
pub const CAR_DWORDS: usize = 0x1A1;

/// The starting state's pieces in order, each with its dwords (`rep movsd` one after
/// another, as `drive::save_replay_cars` gathers them and `load_replay_cars` puts them
/// back).
pub const START_PIECES: [(StartPiece, usize); 6] = [
    (StartPiece::Car(0), CAR_DWORDS),
    (StartPiece::Car(1), CAR_DWORDS),
    (StartPiece::Car(2), CAR_DWORDS),
    (StartPiece::Car(3), CAR_DWORDS),
    (StartPiece::RaceSlots, 0xF0),
    (StartPiece::Extra, 0x100),
];

/// The starting state's dwords.
pub const START_DWORDS: usize = 4 * CAR_DWORDS + 0xF0 + 0x100;

/// Each piece with its offset in the starting state (dwords) and its dwords.
pub fn start_layout() -> impl Iterator<Item = (StartPiece, usize, usize)> {
    START_PIECES.iter().scan(0, |at, &(piece, n)| {
        let here = *at;
        *at += n;
        Some((piece, here, n))
    })
}

/// The dwords of the loose parts (`g_raceSlots`: eight of 0x1E dwords) in the starting state.
pub const SLOT_DWORDS: usize = 0xF0;
/// The dwords of the computer driver's state in the starting state.
pub const EXTRA_DWORDS: usize = 0x100;
/// The dwords of the tape: a hundred tenths of 16 bytes.
pub const TAPE_DWORDS: usize = 400;
/// A replay frame as the file holds it (0x2810 bytes): the starting state, then the tape.
pub const FRAME_DWORDS: usize = START_DWORDS + TAPE_DWORDS;
pub const FRAME_BYTES: usize = 4 * FRAME_DWORDS;

/// One frame of a replay file, field by field at the file's offsets (the byte format is the
/// 1999 game's and never changes; little-endian dwords):
/// - dword 0: the four cars' first `CAR_DWORDS` dwords each, one after another;
/// - then the loose parts, `SLOT_DWORDS`;
/// - then the computer driver's state, `EXTRA_DWORDS`;
/// - then the tape, `TAPE_DWORDS`: the inputs of each tenth, 16 bytes, 8 for each of two cars.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReplayFrame {
    pub cars: [[u32; CAR_DWORDS]; 4],
    pub race_slots: [u32; SLOT_DWORDS],
    pub extra: [u32; EXTRA_DWORDS],
    pub tape: [u32; TAPE_DWORDS],
}

impl ReplayFrame {
    pub const fn new() -> Self {
        ReplayFrame {
            cars: [[0; CAR_DWORDS]; 4],
            race_slots: [0; SLOT_DWORDS],
            extra: [0; EXTRA_DWORDS],
            tape: [0; TAPE_DWORDS],
        }
    }

    /// The frame from the file's bytes; none unless exactly `FRAME_BYTES`.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FRAME_BYTES {
            return None;
        }
        let mut dwords = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|d| u32::from_le_bytes(*d));
        let mut frame = Self::new();
        for car in &mut frame.cars {
            car.fill_with(|| dwords.next().unwrap_or(0));
        }
        frame.race_slots.fill_with(|| dwords.next().unwrap_or(0));
        frame.extra.fill_with(|| dwords.next().unwrap_or(0));
        frame.tape.fill_with(|| dwords.next().unwrap_or(0));
        Some(frame)
    }

    /// The file's bytes of the frame.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.start_bytes();
        out.extend(self.tape.iter().flat_map(|d| d.to_le_bytes()));
        out
    }

    /// The bytes of the starting state alone (`START_DWORDS` dwords): the file's frame
    /// without its tape.
    pub fn start_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(FRAME_BYTES);
        let dwords = self
            .cars
            .iter()
            .flatten()
            .chain(&self.race_slots)
            .chain(&self.extra);
        out.extend(dwords.flat_map(|d| d.to_le_bytes()));
        out
    }

    /// The starting state from its bytes (`START_DWORDS` dwords), the tape zero; none unless
    /// exactly that many.
    pub fn from_start_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 4 * START_DWORDS {
            return None;
        }
        let mut whole = bytes.to_vec();
        whole.resize(FRAME_BYTES, 0);
        Self::from_bytes(&whole)
    }
}

impl Default for ReplayFrame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames() {
        assert_eq!(frame_and_tenth(1234), (12, 34));
        assert_eq!(slot_offset(9), SLOT_SIZE);
        assert_eq!(frame_fault(0x709, 0), Some(FRAME_PAST));
        assert_eq!(frame_fault(0x708, 101), Some(TENTH_PAST));
        assert_eq!(frame_fault(0x708, 100), None);
        assert!(past_end(5, 5));
        assert_eq!(written_view(-4), 2);
        assert_eq!(written_view(9), 3);
        assert!(flush_due(1, 99));
        assert!(!flush_due(2, 99));
        assert_eq!(tape_offset(3, 1), 56);
        assert!(on_tape(1));
        assert!(!on_tape(2));
    }

    #[test]
    fn start_state() {
        let last = start_layout().last().unwrap();
        assert_eq!(last, (StartPiece::Extra, START_DWORDS - 0x100, 0x100));
        assert_eq!(start_layout().nth(4).unwrap().1, 4 * CAR_DWORDS);
    }

    #[test]
    fn switches() {
        assert!(plays_inputs(PLAYING) && !plays_inputs(RECORDING));
        assert_eq!(records_inputs(PLAYING, 0, || unreachable!()), None);
        assert_eq!(records_inputs(RECORDING, 1, || 2), Some(true));
        assert_eq!(records_inputs(RECORDING, 2, || 2), Some(false));
        assert_eq!(marks_offset(0x2000_0001), 8);
    }

    #[test]
    fn frame_round_trips_byte_exact() {
        let bytes: Vec<u8> = (0..FRAME_BYTES as u32)
            .map(|n| (n.wrapping_mul(2654435761) >> 13) as u8)
            .collect();
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        assert_eq!(frame.to_bytes(), bytes);
        assert_eq!(frame.start_bytes(), bytes[..4 * START_DWORDS]);
        assert_eq!(ReplayFrame::from_bytes(&bytes[1..]), None);
        let start = ReplayFrame::from_start_bytes(&bytes[..4 * START_DWORDS]).unwrap();
        assert_eq!(start.cars, frame.cars);
        assert_eq!(start.extra, frame.extra);
        assert_eq!(start.tape, [0; TAPE_DWORDS]);
    }

    #[test]
    fn frame_fields_sit_at_the_files_offsets() {
        let mut bytes = vec![0u8; FRAME_BYTES];
        let mark = |dword: usize| 4 * dword;
        bytes[mark(CAR_DWORDS)] = 1; // the second car's first dword
        bytes[mark(4 * CAR_DWORDS)] = 2; // the first loose part
        bytes[mark(4 * CAR_DWORDS + SLOT_DWORDS)] = 3; // the driver's state
        bytes[mark(START_DWORDS)] = 4; // the tape
        let frame = ReplayFrame::from_bytes(&bytes).unwrap();
        assert_eq!(frame.cars[1][0], 1);
        assert_eq!(frame.race_slots[0], 2);
        assert_eq!(frame.extra[0], 3);
        assert_eq!(frame.tape[0], 4);
        assert_eq!(FRAME_BYTES, 0x2810);
    }
}
