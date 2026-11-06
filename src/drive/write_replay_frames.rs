//! `drive::write_replay_frames` (0x41E930): while a replay is written to its file
//! (`dword_5BEC88` 1): `dword_5BECD4` + 100, the frame's head (`dword_9613A4`, `9613A8`)
//! 0, its byte +5 held to 2 ..= 3 (signed) unless `dword_719B34` is 1,
//! `race::write_replay_block` with `dword_5C252C` 100 round it, and `dword_9613A0` the
//! frame's address. All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::flush_replay`. Refuses what `write_replay_block` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::replay_work_store::REPLAY_WORK;
use rc2k_core::drive::replay_frames::written_view;

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::written_view`.
pub fn write_replay_frames(race: &Race) -> Result<(), Refused> {
    let replay = &race.replay;
    if race.shared.replay_file.replay_open.get() != 1 {
        return Ok(());
    }
    race.shared.replay_file.index_frame_count.set(
        race.shared
            .replay_file
            .index_frame_count
            .get()
            .wrapping_add(0x64),
    );
    replay.replay_view.frames_in_block.set(0x64);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[0].set(0);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[1].set(0);
    if REPLAY_WORK.header_one_player.get() != 1 {
        // the header's byte 5 (the second dword's second byte)
        let cell = &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[1];
        let v = written_view((cell.get() >> 8) as u8 as i8);
        cell.set(cell.get() & !0xFF00 | u32::from(v as u8) << 8);
    }
    // `race::write_replay_block` reads no register at entry (liveness: DF only).
    { crate::race::replay::write_replay_block(replay) }?;
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_used
        .set(0);
    replay.replay_view.frames_in_block.set(0);
    Ok(())
}
