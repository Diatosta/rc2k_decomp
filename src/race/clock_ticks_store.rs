//! The race clock's ticks, 0x5CC198 .. 0x5CC3C0: the text at 0x5CC198 ("demof\build"), the 0x20
//! marks set to -1 at the start and end of a race, and the last tick and tick count of the race
//! sounds. Stops before the kind tables (`race::kind_records_store`).
//! Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ClockTicks {
    /// +0x0 (0x5CC198): `aDemofBuild` ("demof\\build", 12 bytes) and the bytes after it up to 0x5CC1B0: not read by the game's code.
    pub unused_text: [PlainDword; 6],
    /// +0x18 (0x5CC1B0): `unk_5CC1B0`: 0x20 dwords set to -1 by `drive::race_loop` and `drive::finish_run` (IDA: only those two writes refer to it; read by no code).
    pub marks: [PlainDword; 32],
    /// +0x98 (0x5CC230): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_98: [PlainDword; 96],
    /// +0x218 (0x5CC3B0): `g_raceSoundTickLast`: the scene clock at the last race-sound tick (`sound::race_sound_tick`, `drive::race_loop`, `drive::finish_run`).
    pub sound_tick_last: PlainDword,
    /// +0x21c (0x5CC3B4): `dword_5CC3B4`: set to the scene clock at the race's start (`drive::race_loop`, `drive::finish_run`).
    pub clock_a: PlainDword,
    /// +0x220 (0x5CC3B8): `g_raceSoundTicks`: the ticks counted (`sound::race_sound_tick`, `race::stage_loading`).
    pub sound_ticks: PlainDword,
    /// +0x224 (0x5CC3BC): `dword_5CC3BC`: the scene clock when the stage was loaded (`race::stage_loading`).
    pub clock_b: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ClockTicks {
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5CC198, the first byte of the piece.
pub const START: usize = 0x5CC198;
/// IDA 0x5CC3C0, the end of the piece.
pub const END: usize = 0x5CC3C0;

const _: () = assert!(END - START == core::mem::size_of::<ClockTicks>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    std::ptr::from_ref(race().frame.clock_ticks()).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            race().frame.clock_ticks(),
        );
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(ClockTicks, unused_text), 0x0);
        assert_eq!(offset_of!(ClockTicks, clock_a), 0x21c);
        assert_eq!(offset_of!(ClockTicks, clock_b), 0x224);
    }
}
