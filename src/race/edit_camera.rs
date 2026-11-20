//! The two "edit a camera" actions of the replay viewer, and the camera changes they lead to.
//! `edit_camera` keeps the replay for an undo (`race::keep_undo`) and, unless the block shown
//! (`replay_block`) is the last (the replay index's `block_count`), sets `camera_edit` to 1 or 2:
//! the keys then change that camera. `step_camera`, unless the block shown is the last, moves
//! its camera byte (`camera_edit` - 1; `Buffers.viewer.block[2563][edit + block * 8]`) one on
//! (past 0x2E back to 0) or back (below 0 to 0x2E), skipping the cameras whose name
//! (`ViewerAndKinds.viewer_panel.camera_name_ids[0]`) is 0x1C1 or 0x1C2 unless the replay is one
//! player's (`header_one_player` 1) with a first car outside 0x17 .. 0x20; then the change is
//! spread on (`race::spread_camera`).

use crate::errors::Refused;
use crate::race::owner::Race;

/// Edits camera `camera` (1 or 2).
pub fn edit_camera(race: &Race, camera: u32) {
    let replay = &race.replay;
    crate::race::keep_undo::keep_undo(race);
    if rc2k_core::race::viewer_flow::edits_camera(
        replay.replay_view().replay_block.get(),
        race.shared.replay_file().index.block_count.get(),
    ) {
        race.viewer_buffers().camera_edit.set(camera);
    }
}

fn name_of(race: &Race, camera: u8) -> u16 {
    race.viewer_panel().camera_name(camera as usize)
}

/// Moves the camera forward (`step` 1) or back (-1).
pub fn step_camera(race: &Race, step: i8) -> Result<(), Refused> {
    let replay = &race.replay;
    let block = replay.replay_view().replay_block.get();
    if block == race.shared.replay_file().index.block_count.get() {
        return Ok(());
    }
    // The camera edited is the one of the car shown: the first or second, or in a replay with the
    // extra tape the third or fourth when the layout shows it alone.
    let mut edited = 0u32
        .wrapping_sub(1)
        .wrapping_add(race.viewer_buffers().camera_edit.get());
    if race.viewer_buffers().extra_tape.is_set() {
        let layout: u8 = crate::race::viewer::camera_byte(5u32.wrapping_add(block.wrapping_mul(8)))
            .read(race, "a replay camera byte")?;
        edited = rc2k_core::race::viewer::shown_camera_byte(layout, edited);
    }
    let at = crate::race::viewer::camera_byte(edited.wrapping_add(block.wrapping_mul(8)));
    let old: u8 = at.read(race, "a replay camera byte")?;
    use rc2k_core::race::viewer::{camera_skipped, every_camera, step_camera};
    let every = every_camera(
        race.replay_work().header_one_player.get() == 1,
        race.replay_work().header_kind_0.get(),
    );
    let new_camera = step_camera(old, step > 0, every, |n| camera_skipped(name_of(race, n)))?;
    at.write(race, new_camera, "a replay camera byte")?;
    let spread = crate::race::spread_camera::spread_camera(race, at, old);
    if spread.is_err() {
        // A refusal comes before anything is written: nothing is left written.
        at.write(race, old, "a replay camera byte")?;
    }
    spread
}
