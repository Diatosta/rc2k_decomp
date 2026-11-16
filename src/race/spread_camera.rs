//! `race::spread_camera`: after a block's camera byte changes, the new value is also written into
//! the same byte of the next blocks (8 bytes on each) while they held the old one, for the blocks
//! after the one shown (`index.block_count` - `replay_block` less one at most).
//!
//! The count is not checked: a count of 0 or less is limited only by the bytes met. Refuses, before
//! anything is written, a walk that would pass the 0x708 blocks of the table.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::trap::Trap;
use crate::race::viewer::CameraByte;

/// Spreads the changed camera byte `byte` (`old` is its old value) over the following blocks.
pub fn spread_camera(race: &Race, byte: CameraByte, old: u8) -> Result<(), Refused> {
    let replay = &race.replay;
    // The camera bytes of the following blocks, 8 bytes apart (a stride over bytes).
    let at = |n: u32| byte.blocks_on(n);
    let dl = byte.read(race, "a replay camera byte")?;
    let count = race
        .shared
        .replay_file()
        .index
        .block_count
        .get()
        .wrapping_sub(replay.replay_view().replay_block.get());
    // How far it goes is worked out first.
    let trap = Trap::new();
    let s = rc2k_core::race::viewer::spread(count, old, |n| {
        trap.keep_read(at(n).read(race, "a replay camera byte"))
    })?;
    trap.finish()?;
    for n in 1..=s.taken {
        at(n).write(race, dl, "a replay camera byte")?;
    }
    Ok(())
}
