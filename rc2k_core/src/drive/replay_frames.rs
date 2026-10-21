//! The replay's frames (converted 2026-10-21 from `drive::keep_replay_frame` 0x45FFB6,
//! `drive::read_replay_frame` 0x41EBAB, `drive::write_replay_frames` 0x41E930,
//! `drive::flush_replay_if` 0x460031, `drive::record_inputs` 0x4601A8 and
//! `drive::play_inputs` 0x460162, their switches `play_inputs_if` 0x4600A9 and
//! `record_inputs_if` 0x46008D, the frame marks of `replay_frame_marks` 0x41D8A3; the
//! starting state's layout from
//! `drive::save_replay_cars` 0x4600B8 and `drive::load_replay_cars` 0x46010D): a frame is
//! a hundred tenths; at each whole frame the four car blocks are kept in one of eight
//! slots; the tape holds each tenth's input words, 16 bytes a tenth, 8 for each of the
//! two cars.

/// The replay's mode (`dword_5C5CE0`): 1 recording, 2 playing.
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

/// Whether a frame is at or past the replay's last (`dword_5BECD0`): then the last is
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
/// the race's count (`dword_63DB94`; asked only while recording).
pub fn records_inputs(mode: u32, car: u32, cars: impl FnOnce() -> u32) -> Option<bool> {
    (mode == RECORDING).then(|| car < cars())
}

/// `drive::replay_frame_marks`: the offset of a frame's two marks (8 bytes a frame,
/// `unk_963BB4`; the original's `shl`, high bits lost).
pub fn marks_offset(frame: u32) -> u32 {
    frame.wrapping_mul(8)
}

/// A piece of the replay's starting state (`unk_7172EC`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartPiece {
    /// The first `CAR_DWORDS` of car block k (`g_carBlockStore` + 0xFE0 k).
    Car(u32),
    /// The race slots (`g_raceSlots`).
    RaceSlots,
    /// `unk_71BF84`.
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
}
