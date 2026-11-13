//! Reads a replay's frame while a replay plays from its file (else it reports failure). Frames
//! past 0x708 and tenths past 100 are the fatal errors 0x80A61151 and 0x80A61152. At or past the
//! last frame: the last frame, `replay_at_end` 1, the tenth 0. Else the tenth goes into
//! `replay_frame`, `replay_at_end` 0, the frame into `replay_block`, the file is put at the
//! frame's place and its bytes (packed when the place is below 0) are read into `block`
//! (`drive::read_packed`). Refuses what `read_packed` refuses.

use crate::errors::{FatalError, Refused};
use crate::race::owner::Race;
use crate::system::owner::System;
use rc2k_core::drive::replay_frames::{frame_fault, past_end};

pub fn read_replay_frame(
    race: &Race,
    system: &System,
    frame: u32,
    tenth: u32,
) -> Result<bool, Refused> {
    let race_owner = race;
    let replay = &race.replay;
    if !race_owner.shared.replay_file().replay_open.is_set() {
        return Ok(true);
    }
    if let Some(code) = frame_fault(frame, tenth) {
        return Err(FatalError { code, message: "" }.into());
    }
    if past_end(
        frame,
        race_owner.shared.replay_file().index.block_count.get(),
    ) {
        race_owner.shared.replay_file().replay_at_end.set(true);
        replay
            .replay_view()
            .replay_block
            .set(race_owner.shared.replay_file().index.block_count.get());
        replay.replay_view().replay_frame.set(0);
        return Ok(false);
    }
    replay.replay_view().replay_frame.set(tenth);
    race_owner.shared.replay_file().replay_at_end.set(false);
    replay.replay_view().replay_block.set(frame);
    let place = crate::race::replay::block_place(race_owner, 2 * frame as usize)?; // (place, count) a frame
    if {
        crate::files::handle::seek(
            race_owner.shared.replay_file().replay_handle.get(),
            place as i32,
            0,
        )
    }
    .is_none()
    {
        return Ok(true);
    }
    let count = crate::race::replay::block_place(race_owner, 2 * frame as usize + 1)?;
    let (_, carry, _) = {
        crate::drive::read_packed::read_packed(
            race_owner,
            system,
            race_owner.shared.replay_file().replay_handle.get(),
            count,
        )
    }?;
    Ok(carry)
}
