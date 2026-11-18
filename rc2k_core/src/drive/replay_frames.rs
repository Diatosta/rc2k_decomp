//! The replay's frames: a frame is a hundred tenths; at each whole frame the four car blocks are
//! kept in one of eight slots; the tape holds each tenth's input words, 16 bytes a tenth, 8 for
//! each of the two cars.

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
pub const LAST_FRAME: u32 = crate::race::replay_v2::MOST_BLOCKS;
pub const FRAME_PAST: u32 = 0x80A6_1151;
pub const TENTH_PAST: u32 = 0x80A6_1152;

/// The fatal error's code for reading a frame or a tenth past its end; None when both are in
/// range.
pub fn frame_fault(frame: u32, tenth: u32) -> Option<u32> {
    if frame > LAST_FRAME {
        Some(FRAME_PAST)
    } else if tenth > TENTHS {
        Some(TENTH_PAST)
    } else {
        None
    }
}

/// Whether a frame is at or past the replay's last: then the last is shown and nothing read.
pub fn past_end(frame: u32, frames: u32) -> bool {
    frame >= frames
}

/// The written frame head's byte +5 (a reading: the view), held to 2 ..= 3 (signed).
pub fn written_view(byte: i8) -> i8 {
    byte.clamp(2, 3)
}

/// The frame head's byte +5 for a network race: the view of this machine's car (`car`), so the
/// replay opens on it: 0 the first car, 1 the second (the tape holds two cars; a later one's
/// machine gets the first).
pub fn network_view(car: u32) -> u8 {
    u8::from(car == 1)
}

/// Whether the replay is flushed: while recording, at the last tenth of a frame.
pub fn flush_due(mode: u32, tenth: u32) -> bool {
    mode == RECORDING && tenth == TENTHS - 1
}

/// The cars on the tape of every replay.
pub const TAPE_CARS: u32 = 2;
/// The cars on the tape of a replay with the extra tape (3 and 4 player races).
pub const EXTENDED_TAPE_CARS: u32 = 4;
/// The cars the extra tape holds: cars 2 and 3.
pub const EXTRA_TAPE_CARS: u32 = EXTENDED_TAPE_CARS - TAPE_CARS;

/// Whether a car is on the tape; `extended` for a replay with the extra tape.
pub fn on_tape(car: u32, extended: bool) -> bool {
    car < if extended {
        EXTENDED_TAPE_CARS
    } else {
        TAPE_CARS
    }
}

/// Where a car's entry on the tape is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TapePlace {
    /// The frame's own tape (cars 0 and 1): the tenth and the car.
    Main(usize, usize),
    /// The extra tape that follows the frame (cars 2 and 3): the tenth and the car less 2.
    Extra(usize, usize),
}

/// A car's entry on the tape at a tenth (16 bytes a tenth, 8 a car; the extra tape the same for
/// cars 2 and 3): none past the frame's tenths or the cars on the tape.
pub fn tape_place(tenth: u32, car: u32, extended: bool) -> Option<TapePlace> {
    if tenth >= TENTHS || !on_tape(car, extended) {
        return None;
    }
    Some(if car < TAPE_CARS {
        TapePlace::Main(tenth as usize, car as usize)
    } else {
        TapePlace::Extra(tenth as usize, (car - TAPE_CARS) as usize)
    })
}

/// The dwords of the extra tape: a hundred tenths of 16 bytes, for cars 2 and 3.
pub const EXTRA_TAPE_DWORDS: usize = 400;
/// The extra tape's bytes: what a block of a replay with the extra tape holds after the plain
/// frame.
pub const EXTRA_TAPE_BYTES: usize = 4 * EXTRA_TAPE_DWORDS;
/// A block's bytes once unpacked in a replay with the extra tape.
pub const EXTENDED_FRAME_BYTES: usize = FRAME_BYTES + EXTRA_TAPE_BYTES;

/// A block's bytes once unpacked.
pub fn block_bytes(extended: bool) -> usize {
    if extended {
        EXTENDED_FRAME_BYTES
    } else {
        FRAME_BYTES
    }
}

/// The inputs of cars 2 and 3 are recorded: a local race of more than two players (a network
/// race's other machines keep their cars' state alone, as before).
pub fn records_extra_tape(players: u32, network: bool) -> bool {
    players > TAPE_CARS && !network
}

/// Whether the inputs are played back: while playing a replay.
pub fn plays_inputs(mode: u32) -> bool {
    mode == PLAYING
}

/// Whether a car's inputs are recorded: None unless recording; else for the cars below the
/// race's count (`RaceView.players`, asked only while recording).
pub fn records_inputs(mode: u32, car: u32, cars: impl FnOnce() -> u32) -> Option<bool> {
    (mode == RECORDING).then(|| car < cars())
}

/// The byte offset of a frame's two marks in `Buffers.viewer.block_cameras` (8 bytes a frame,
/// high bits lost).
pub fn marks_offset(frame: u32) -> u32 {
    frame.wrapping_mul(8)
}

/// A piece of the replay's starting state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartPiece {
    /// The first `CAR_DWORDS` of car block k.
    Car(u32),
    /// The race slots (eight of 0x1E dwords).
    RaceSlots,
    /// `RaceWork.replay_extra`.
    Extra,
}

/// The dwords kept of each car block.
pub const CAR_DWORDS: usize = 0x1A1;

/// The starting state's pieces in order, each with its dwords, copied one after another when
/// the cars are saved to a frame and loaded back.
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

/// The dwords of the loose parts (the race slots) in the starting state.
pub const SLOT_DWORDS: usize = 0xF0;
/// The dwords of the computer driver's state in the starting state.
pub const EXTRA_DWORDS: usize = 0x100;
/// The dwords of the tape: a hundred tenths of 16 bytes.
pub const TAPE_DWORDS: usize = 400;
/// A replay frame as the file holds it (0x2810 bytes): the starting state, then the tape.
pub const FRAME_DWORDS: usize = START_DWORDS + TAPE_DWORDS;
pub const FRAME_BYTES: usize = 4 * FRAME_DWORDS;

/// One frame of a replay file, field by field at the file's offsets (little-endian dwords; the
/// format is fixed so that every replay keeps loading):
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
        assert_eq!(frame_fault(LAST_FRAME + 1, 0), Some(FRAME_PAST));
        assert_eq!(frame_fault(LAST_FRAME, 101), Some(TENTH_PAST));
        assert_eq!(frame_fault(LAST_FRAME, 100), None);
        assert!(past_end(5, 5));
        assert_eq!(written_view(-4), 2);
        assert_eq!(written_view(9), 3);
        assert_eq!([0, 1, 2].map(network_view), [0, 1, 0]);
        assert!(flush_due(1, 99));
        assert!(!flush_due(2, 99));
        assert_eq!(tape_place(3, 1, false), Some(TapePlace::Main(3, 1)));
        assert_eq!(tape_place(TENTHS, 0, true), None);
        assert_eq!(tape_place(0, TAPE_CARS, false), None);
        assert_eq!(tape_place(7, 3, true), Some(TapePlace::Extra(7, 1)));
        assert_eq!(tape_place(7, 4, true), None);
        assert!(on_tape(1, false));
        assert!(!on_tape(2, false) && on_tape(2, true));
        assert_eq!(block_bytes(false), 0x2810);
        assert_eq!(block_bytes(true), 0x2E50);
        assert!(records_extra_tape(3, false) && !records_extra_tape(2, false));
        assert!(!records_extra_tape(4, true));
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
