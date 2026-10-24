//! The screenshot switch, its lock and the file name's template (phase 5, `notes/conversion.md`): IDA 0x4F52B4 .. 0x4F52C4 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

use crate::gfx::block::game_read;

crate::region!(
    Shots,
    STORE,
    0x4F52B4,
    0x4F52C4,
    crate::gfx::shots_data::RUNS
);

/// `g_screenshotsOn` (a `var\\ral.cfg` setting, `screensnapshot`).
pub const ON: Slot = Slot::at(0x4F52B4);
/// `dword_4F52B8`: the lock a screenshot holds.
pub const LOCKED_HERE: Slot = Slot::at(0x4F52B8);
/// The screenshot file's name, "000.pcx".
pub const NAME: Slot = Slot::at(0x4F52BC);

/// Whether screenshots are on.
pub fn on() -> u32 {
    unsafe { game_read(ON) }
}

/// The switch's first byte.
pub fn on_byte() -> u8 {
    unsafe { game_read(ON) }
}

pub const START: usize = 0x4F52B4;

pub type Slot = crate::gfx::block::Slot<Shots>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Shots>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
