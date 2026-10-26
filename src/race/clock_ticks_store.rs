//! The race clock's ticks, 0x5CC198 .. 0x5CC3C0: the text at 0x5CC198 ("demof\build"), the 0x20
//! marks set to -1 at the start and end of a race, and the last tick and tick count of the race
//! sounds. Stops before the kind tables (`race::kind_records_store`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct ClockTicks {
    /// +0x0 (0x5CC198): `aDemofBuild` ("demof\\build", 12 bytes) and the bytes after it up to 0x5CC1B0: not read by the game's code.
    pub unknown_text: [Dword; 6],
    /// +0x18 (0x5CC1B0): `unk_5CC1B0`: 0x20 dwords set to -1 by `drive::race_loop` and `drive::finish_run` (a reading).
    pub marks: [Dword; 32],
    /// +0x98 (0x5CC230): not named.
    pub unknown_98: [Dword; 96],
    /// +0x218 (0x5CC3B0): `g_raceSoundTickLast`: the scene clock at the last race-sound tick (`sound::race_sound_tick`, `drive::race_loop`, `drive::finish_run`).
    pub sound_tick_last: Dword,
    /// +0x21c (0x5CC3B4): `dword_5CC3B4`: set to the scene clock at the race's start (`drive::race_loop`, `drive::finish_run`).
    pub clock_a: Dword,
    /// +0x220 (0x5CC3B8): `g_raceSoundTicks`: the ticks counted (`sound::race_sound_tick`, `race::stage_loading`).
    pub sound_ticks: Dword,
    /// +0x224 (0x5CC3BC): `dword_5CC3BC`: the scene clock when the stage was loaded (`race::stage_loading`).
    pub clock_b: Dword,
}

/// The piece: the race block's bytes at IDA 0x5CC198 seen as `ClockTicks`. They stay there
/// while code reaches them by RVA from `race::store::asm_base()` and by IDA address
/// (`moved::place`): one copy for all three.
pub struct InRaceBlock;

impl core::ops::Deref for InRaceBlock {
    type Target = ClockTicks;
    fn deref(&self) -> &ClockTicks {
        // SAFETY: the race block lives as long as the program and is aligned to 4 (the
        // test checks); every field is an atomic or a cell, as the block's bytes were shared.
        unsafe { &*(crate::pointers::host_at(piece_addr()) as *const ClockTicks) }
    }
}

pub static CLOCK_TICKS: InRaceBlock = InRaceBlock;

/// IDA 0x5CC198, the first byte of the piece.
pub const START: usize = 0x5CC198;
/// IDA 0x5CC3C0, the end of the piece.
pub const END: usize = 0x5CC3C0;

const _: () = assert!(END - START == core::mem::size_of::<ClockTicks>());

/// Where IDA 0x5CC198 is now (in the race block).
pub fn piece_addr() -> usize {
    crate::race::store::home(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(ClockTicks, unknown_text), 0x5CC198 - START);
        assert_eq!(offset_of!(ClockTicks, clock_a), 0x5CC3B4 - START);
        assert_eq!(offset_of!(ClockTicks, clock_b), 0x5CC3BC - START);
    }
}
