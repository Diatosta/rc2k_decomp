//! `race::spread_camera` (0x41E6BD): a block's camera byte changed: the new value also into the
//! next blocks' same byte (8 bytes on each) while they held the old one, for the blocks after the
//! one shown (`index_block_count` - `replay_block` less one at most).
//!
//! Callers: `race::viewer_action` (four), `race::next_camera`, `race::previous_camera`, with
//! the block shown not the last. The count is not checked: 0 (or less) is limited only by the
//! bytes met; refused when the walk would pass the 0x708 blocks of the table, before anything
//! is written.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::trap::Trap;
use crate::race::viewer::CameraByte;

/// `byte` the changed camera byte, `old` its old value.
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
    // Converted: `rc2k_core::race::viewer::spread` (how far it goes, first).
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
