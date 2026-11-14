//! `race::viewer_action`: one of the replay viewer's actions, by number:
//!
//! - 0, 1, 0xB, 0xC: to the start, the first mark, the second mark, the end
//!   (`drive::read_replay_frame_0`).
//! - 3, 4, 8, 9: 10 back, 1 back, 1 on, 10 on (held to 0 .. the count, signed).
//! - 5, 7: a frame back, a frame on (`drive::read_replay_frame`).
//! - 2, 0xA: the first mark here (not past the second), the second here (not before the first).
//! - 6, 0x22: play from here (`race::play_from_here`; 0x22 with `play_from_here` 1).
//! - 0x1A: play between the marks (`race::play_marked`).
//! - 0x23: play from the start again and again until a key ends a play.
//! - 0x11: cut the marked blocks (`race::keep_undo`, `race::copy_marked`, the rest moved down,
//!   the end zeroed; `race::reset_marks`).
//! - 0x1B: copy them (`race::copy_marked`).
//! - 0x12: paste the clipboard here (if it fits in 0x708 blocks; the rest moved up).
//! - 0x13, 0x20: undo (`race::undo_edit`; 0x20 also ends a camera edit).
//! - 0x14: another replay (not for the last race's): the stage is tidied, the loading screen
//!   shows (`menu::replay_slot_loop`), the old replay is closed and the new one opened (or chosen
//!   again) and loaded; if none is chosen the old one is reloaded if open, else the viewer is
//!   left (`list_leave` 1).
//! - 0x15: saved back (a saved slot below 100, else as 0x16).
//! - 0x16: saved as a slot chosen (`race::choose_save_slot`, `race::save_viewed_replay`).
//! - 0x17: a picture of the next frame (`snapshot_asked` 1).
//! - 0xE, 0x18: the block's camera byte 2 / 3 flipped, spread on (`race::spread_camera`).
//! - 0xF: byte 5 one on (0 .. 3, past 3 back to 2) unless one player's; 0x10: byte 4 (0 .. 2).
//! - 0xD, 0x19: camera 1 / 2 edited (`race::edit_camera`).
//! - 0x1C, 0x24, 0x1D: camera 1 on / back, camera 2 on, the edit ended.
//! - 0x1E, 0x1F: the camera edited back / on; 0x21: the edit ended (`camera_edit` 0).
//! - 0x25 .. 0x28: play between the marks saving pictures (`race::start_frame_pictures` 0, 0xA0,
//!   0x50, 0x28 ms; `race::stop_frame_pictures`).
//!
//! Called by `race::replay_viewer`. The camera bytes' edits (0xE .. 0x10, 0x18) keep the undo
//! before they look at the block. Nothing is checked of the table's room for a cut or a paste
//! beyond the 0x708 blocks, except that a block shown past the count is refused for a paste (it
//! would move memory by a negative count).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Does viewer action `action`; returns whether it is an action.
pub fn viewer_action(
    race: &'static Race,
    o: Owners<'static>,
    action: u32,
) -> Result<bool, Refused> {
    let replay = &race.replay;
    use rc2k_core::race::viewer::{
        Action, Var, frame_back, frame_on, goto_block, mark_in_here, mark_out_here,
    };
    let Some(action) = Action::of(action) else {
        return Ok(false);
    };
    match action {
        Action::Goto(goto) => {
            let block = goto_block(goto, |v| match v {
                Var::Block => replay.replay_view().replay_block.get(),
                Var::Blocks => race.shared.replay_file().index.block_count.get(),
                Var::MarkIn => race.viewer_buffers().mark_in.get(),
                Var::MarkOut => race.viewer_buffers().mark_out.get(),
            });
            // Whether the frame could be read is not looked at.
            crate::drive::read_replay_frame_0::read_replay_frame_0(race, o.system, block)?;
        }
        Action::MarkIn => {
            if mark_in_here(
                replay.replay_view().replay_block.get(),
                race.viewer_buffers().mark_out.get(),
            ) {
                race.viewer_buffers()
                    .mark_in
                    .set(replay.replay_view().replay_block.get());
            }
        }
        Action::MarkOut => {
            if mark_out_here(
                replay.replay_view().replay_block.get(),
                race.viewer_buffers().mark_in.get(),
            ) {
                race.viewer_buffers()
                    .mark_out
                    .set(replay.replay_view().replay_block.get());
            }
        }
        Action::FrameBack => {
            let (block, frame) = frame_back(
                replay.replay_view().replay_block.get(),
                replay.replay_view().replay_frame.get(),
            );
            read_at(race, o.system, block, frame)?;
        }
        Action::FrameOn => {
            let (block, frame) = frame_on(
                replay.replay_view().replay_block.get(),
                replay.replay_view().replay_frame.get(),
                race.shared.replay_file().index.block_count.get(),
            );
            read_at(race, o.system, block, frame)?;
        }
        Action::PlayHere { from_here } => {
            if from_here {
                replay.replay_view().play_from_here.set(1);
            }
            crate::race::play_marked::play_from_here(race, o)?;
        }
        Action::PlayMarked => crate::race::play_marked::play_marked(race, o)?,
        Action::PlayAgain => loop {
            replay.replay_view().replay_block.set(0);
            replay.replay_view().replay_frame.set(0);
            crate::race::play_marked::play_from_here(race, o)?;
            if replay.replay_view().key_after_play.get() != 0 {
                break;
            }
        },
        Action::Cut => cut(race, o.system)?,
        Action::CopyMarked => {
            crate::race::copy_marked::copy_marked(race)?;
        }
        Action::Paste => paste(race, o.system)?,
        Action::Undo { end_edit } => {
            crate::race::undo_edit::undo_edit(race, o.system)?;
            if end_edit {
                race.viewer_buffers().camera_edit.set(0);
            }
        }
        Action::AnotherReplay => another_replay(race, o)?,
        Action::Save { back } => save(race, o, back)?,
        Action::Snapshot => replay.replay_view().snapshot_asked.set(1),
        Action::Cycle { byte } => {
            if byte == 4 || race.replay_work().header_one_player.get() != 1 {
                cycle_byte(race, byte as usize)?;
            }
        }
        Action::Flip { byte } => {
            crate::race::keep_undo::keep_undo(race);
            let block = replay.replay_view().replay_block.get();
            if block != race.shared.replay_file().index.block_count.get() {
                let at = crate::race::viewer::camera_byte(byte.wrapping_add(block.wrapping_mul(8)));
                let old: u8 = at.read(race, "a replay block's camera byte")?;
                at.write(race, old ^ 1, "a replay block's camera byte")?;
                crate::race::spread_camera::spread_camera(race, at, old)?;
            }
        }
        Action::EditAndStep { camera, back } => {
            crate::race::edit_camera::edit_camera(race, camera);
            crate::race::edit_camera::step_camera(race, if back { -1 } else { 1 })?;
            race.viewer_buffers().camera_edit.set(0);
        }
        Action::Edit { camera } => crate::race::edit_camera::edit_camera(race, camera),
        Action::Step { back } => {
            crate::race::edit_camera::step_camera(race, if back { -1 } else { 1 })?;
        }
        Action::EndEdit => race.viewer_buffers().camera_edit.set(0),
        Action::Pictures { time } => {
            crate::race::start_frame_pictures::start_frame_pictures(race, time)?;
            crate::race::play_marked::play_marked(race, o)?;
            crate::race::stop_frame_pictures::stop_frame_pictures(race);
        }
    }
    Ok(true)
}

/// Stores the block and frame, then reads that frame (`drive::read_replay_frame`); whether it
/// could be read is not looked at.
fn read_at(race: &Race, system: &System, block: u32, frame: u32) -> Result<(), Refused> {
    let replay = &race.replay;
    replay.replay_view().replay_block.set(block);
    replay.replay_view().replay_frame.set(frame);
    { crate::drive::read_replay_frame::read_replay_frame(race, system, block, frame) }?;
    Ok(())
}

/// 0xF, 0x10: the block's camera byte `byte` one on (`viewer::next_byte`), spread on.
fn cycle_byte(race: &Race, byte: usize) -> Result<(), Refused> {
    let replay = &race.replay;
    crate::race::keep_undo::keep_undo(race);
    let block = replay.replay_view().replay_block.get();
    if block == race.shared.replay_file().index.block_count.get() {
        return Ok(());
    }
    let at = crate::race::viewer::camera_byte((byte as u32).wrapping_add(block.wrapping_mul(8)));
    let old: u8 = at.read(race, "a replay block's camera byte")?;
    let dl = rc2k_core::race::viewer::next_byte(byte as u32, old);
    at.write(race, dl, "a replay block's camera byte")?;
    crate::race::spread_camera::spread_camera(race, at, old)
}

/// 0x11: the marked blocks cut (`viewer::cut_marks`).
fn cut(race: &Race, system: &System) -> Result<(), Refused> {
    let shared = &race.shared;
    let Some((first, second)) = rc2k_core::race::viewer::cut_marks(
        race.viewer_buffers().mark_in.get(),
        race.viewer_buffers().mark_out.get(),
        shared.replay_file().index.block_count.get(),
    )?
    else {
        return Ok(());
    };
    crate::race::keep_undo::keep_undo(race);
    let Some(count) = crate::race::copy_marked::copy_marked(race)? else {
        return Err(crate::errors::BadInput("a cut whose marks do not copy").into());
    };
    shared.replay_file().index.block_count.set(
        shared
            .replay_file()
            .index
            .block_count
            .get()
            .wrapping_sub(count),
    );
    let rest = (shared.replay_file().index.block_count.get() + count) - second;
    use crate::race::viewer::{clear_cells, move_cells};
    for table in [
        &shared.replay_file().index.block_places as &dyn crate::math::cells::Cells,
        &race.viewer_buffers().block_cameras,
    ] {
        let (first, second, rest) = (2 * first as usize, 2 * second as usize, 2 * rest as usize);
        move_cells(table, first, table, second, rest)?;
        clear_cells(table, first + rest, 2 * count as usize)?;
    }
    crate::race::reset_marks::reset_marks(race, system)
}

/// 0x12: the clipboard pasted at the block shown (`viewer::pastes`).
fn paste(race: &Race, system: &System) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    let clip = race.viewer_buffers().clip_blocks.get();
    let (block, blocks) = (
        replay.replay_view().replay_block.get(),
        shared.replay_file().index.block_count.get(),
    );
    if !rc2k_core::race::viewer::pastes(clip, blocks, block)? {
        return Ok(());
    }
    crate::race::keep_undo::keep_undo(race);
    // The blocks from the one shown are moved up by the clipboard's count, from the top down:
    // the move may overlap itself.
    use crate::race::viewer::{move_cells, move_cells_back};
    let v = race.viewer_buffers();
    let (at, clip_dwords) = (2 * block as usize, 2 * clip as usize);
    let tables = [
        (
            &shared.replay_file().index.block_places as &dyn crate::math::cells::Cells,
            &v.clip_places,
        ),
        (&v.block_cameras, &v.clip_cameras),
    ];
    for (table, _) in tables {
        move_cells_back(table, at + clip_dwords, at, 2 * (blocks - block) as usize)?;
    }
    for (table, clipped) in tables {
        move_cells(table, at, clipped, 0, clip_dwords)?;
    }
    shared.replay_file().index.block_count.set(blocks + clip);
    crate::race::reset_marks::reset_marks(race, system)
}

/// 0x14: another replay chosen to view.
fn another_replay(race: &'static Race, o: Owners<'static>) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    if replay.replay_view().last_race_viewed.is_set() {
        return Ok(());
    }
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race, o)?;
    loop {
        let slot = crate::menu::replay_slot_loop::replay_slot_loop(o, false)?;
        if slot == u32::MAX {
            if shared.replay_file().replay_open.is_set() {
                return crate::race::viewer_load_stage::viewer_load_stage(race, o);
            }
            o.install.pages.list_leave.set(1);
            return Ok(());
        }
        shared.replay_file().viewer_slot.set(slot);
        crate::race::close_viewed_replay::close_viewed_replay(race)?;
        let failed = crate::race::open_viewed_replay::open_viewed_replay(race, o.system, slot)?;
        if !failed {
            return crate::race::viewer_load_stage::viewer_load_stage(race, o);
        }
    }
}

/// 0x15, 0x16: saved back into its slot (`back`, a slot below 100) or into one chosen.
fn save(race: &'static Race, o: Owners<'static>, back: bool) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    if shared.replay_file().index.block_count.get() == 0 {
        return Ok(());
    }
    let slot = if back && shared.replay_file().viewer_slot.get() < 100 {
        shared.replay_file().viewer_slot.get()
    } else {
        let slot = crate::race::choose_save_slot::choose_save_slot(race, o)?;
        if slot == u32::MAX {
            return Ok(());
        }
        shared.replay_file().viewer_slot.set(slot);
        slot
    };
    if !crate::race::save_viewed_replay::save_viewed_replay(race, o.system, slot)? {
        return Ok(());
    }
    read_at(
        race,
        o.system,
        replay.replay_view().replay_block.get(),
        replay.replay_view().replay_frame.get(),
    )
}
