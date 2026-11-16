//! While a replay is written to its file (`replay_open`
//! 1): `index_frame_count` + 100, the frame's head (`block`) 0, its byte +5 held to 2
//! ..= 3 (signed) unless `header_one_player` is 1, `race::write_replay_block` with `frames_in_block` 100
//! round it, and `block_used` the frame's address.
//!
//! Refuses what `write_replay_block` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::written_view;

pub fn write_replay_frames(race: &Race, system: &System) -> Result<(), Refused> {
    let replay = &race.replay;
    if !race.shared.replay_file().replay_open.is_set() {
        return Ok(());
    }
    race.shared.replay_file().index.frame_count.set(
        race.shared
            .replay_file()
            .index
            .frame_count
            .get()
            .wrapping_add(0x64),
    );
    replay.replay_view().frames_in_block.set(0x64);
    race.viewer_buffers().block[0].set(0);
    race.viewer_buffers().block[1].set(0);
    if race.replay_work().header_one_player.get() != 1 {
        // the header's byte 5 (the second dword's second byte)
        let cell = &race.viewer_buffers().block[1];
        let v = written_view((cell.get() >> 8) as u8 as i8);
        cell.set(cell.get() & !0xFF00 | u32::from(v as u8) << 8);
    }
    { crate::race::replay::write_replay_block(race, system, replay) }?;
    race.viewer_buffers().block_used.set(0);
    replay.replay_view().frames_in_block.set(0);
    Ok(())
}
