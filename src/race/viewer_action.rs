//! `race::viewer_action` (0x41DE6F): one of the replay viewer's actions, ebx (its body is
//! `sub_41DE81`, in chunks, entered only from here; unknown actions `inc ebp`):
//!
//! | action | what |
//! | --- | --- |
//! | 0, 1, 0xB, 0xC | to the start, the first mark, the second mark, the end (`drive::read_replay_frame_0`) |
//! | 3, 4, 8, 9 | 10 back, 1 back, 1 on, 10 on (held to 0 .. the count, signed) |
//! | 5, 7 | a frame back, a frame on (`drive::read_replay_frame`) |
//! | 2, 0xA | the first mark here (not past the second), the second here (not before the first) |
//! | 6, 0x22 | play from here (`race::play_from_here`; 0x22 with `dword_5C253C` 1) |
//! | 0x1A | play between the marks (`race::play_marked`) |
//! | 0x23 | play from the start again and again until a key ends a play |
//! | 0x11 | cut the marked blocks (`race::keep_undo`, `race::copy_marked`, the rest moved down, the end zeroed; `race::reset_marks`) |
//! | 0x1B | copy them (`race::copy_marked`) |
//! | 0x12 | paste the clipboard here (if it fits in 0x708 blocks; the rest moved up) |
//! | 0x13, 0x20 | undo (`race::undo_edit`; 0x20 also ends a camera edit) |
//! | 0x14 | another replay (not for the last race's): tidied, the loading screen (`menu::replay_slot_loop`), the old closed, the new opened (or chosen again), loaded; none chosen: the old reloaded if open, else the viewer left (`byte_51D416` 1) |
//! | 0x15 | saved back (a saved slot below 100, else as 0x16) |
//! | 0x16 | saved as a slot chosen (`race::choose_save_slot`, `race::save_viewed_replay`) |
//! | 0x17 | a picture of the next frame (`dword_5C2540` 1) |
//! | 0xE, 0x18 | the block's camera byte 2 / 3 flipped (`xor 1`), spread on (`race::spread_camera`) |
//! | 0xF | byte 5 one on (0 .. 3, past 3 back to 2) unless one player's; 0x10: byte 4 (0 .. 2) |
//! | 0xD, 0x19 | camera 1 / 2 edited (`race::edit_camera_1`, `_2`) |
//! | 0x1C, 0x24, 0x1D | camera 1 on / back, camera 2 on, the edit ended |
//! | 0x1E, 0x1F | the camera edited back / on; 0x21: the edit ended (`dword_96E480` 0) |
//! | 0x25 .. 0x28 | play between the marks saving pictures (`race::start_frame_pictures` 0, 0xA0, 0x50, 0x28 ms; `race::stop_frame_pictures`) |
//!
//! In:  ebx the action
//! Out: ZF set when it was one; every register kept (`pusha` / `popa`)
//!
//! Callers: `race::replay_viewer` (three). The camera bytes' edits (0xE .. 0x10, 0x18) keep the
//! undo before they look at the block; nothing is checked of the table's room for a cut or a
//! paste beyond what the original checks (a block shown past the count is refused for a paste:
//! the original moves memory by a negative count).

use crate::errors::Refused;
use crate::install::vars::INSTALLER_PAGES;
use crate::menu::screen_loop::Carried;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::viewer_state_store::REPLAY_VIEW;

/// The Rust version: whether the action is one (the zero flag the original leaves; it
/// keeps the registers). `action` the action (the original's ebx), `c` the carried values
/// of the viewer, which the screens it starts are run with.
pub fn viewer_action(action: u32, c: &Carried) -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::viewer::{Action, goto_block, frame_back, frame_on,
    // mark_in_here, mark_out_here}`.
    use rc2k_core::race::viewer::{
        Action, Var, frame_back, frame_on, goto_block, mark_in_here, mark_out_here,
    };
    // (A translated body: `r` holds the carried values of the original's registers.)
    let mut r = Carried {
        arg: action,
        palette: action,
        ..*c
    };
    let Some(action) = Action::of(action) else {
        return Ok(false);
    };
    let r = &mut r;
    match action {
        Action::Goto(goto) => {
            r.arg = goto_block(goto, |v| match v {
                Var::Block => REPLAY_VIEW.replay_block.get(),
                Var::Blocks => REPLAY_FILE.index_block_count.get(),
                Var::MarkIn => crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_in
                    .get(),
                Var::MarkOut => crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_out
                    .get(),
            });
            read_replay_frame_0(r)?;
        }
        Action::MarkIn => {
            if mark_in_here(
                REPLAY_VIEW.replay_block.get(),
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_out
                    .get(),
            ) {
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_in
                    .set(REPLAY_VIEW.replay_block.get());
            }
        }
        Action::MarkOut => {
            if mark_out_here(
                REPLAY_VIEW.replay_block.get(),
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_in
                    .get(),
            ) {
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .mark_out
                    .set(REPLAY_VIEW.replay_block.get());
            }
        }
        Action::FrameBack => {
            let (block, frame) = frame_back(
                REPLAY_VIEW.replay_block.get(),
                REPLAY_VIEW.replay_frame.get(),
            );
            read_at(r, block, frame)?;
        }
        Action::FrameOn => {
            let (block, frame) = frame_on(
                REPLAY_VIEW.replay_block.get(),
                REPLAY_VIEW.replay_frame.get(),
                REPLAY_FILE.index_block_count.get(),
            );
            read_at(r, block, frame)?;
        }
        Action::PlayHere { from_here } => {
            if from_here {
                REPLAY_VIEW.play_from_here.set(1);
            }
            crate::race::play_marked::play_from_here()?;
        }
        Action::PlayMarked => crate::race::play_marked::play_marked()?,
        Action::PlayAgain => loop {
            REPLAY_VIEW.replay_block.set(0);
            REPLAY_VIEW.replay_frame.set(0);
            crate::race::play_marked::play_from_here()?;
            if REPLAY_VIEW.key_after_play.get() != 0 {
                break;
            }
        },
        Action::Cut => cut(r)?,
        Action::CopyMarked => {
            crate::race::copy_marked::copy_marked()?;
        }
        Action::Paste => paste()?,
        Action::Undo { end_edit } => {
            crate::race::undo_edit::undo_edit()?;
            if end_edit {
                crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .camera_edit
                    .set(0);
            }
        }
        Action::AnotherReplay => another_replay(r)?,
        Action::Save { back } => save(r, back)?,
        Action::Snapshot => REPLAY_VIEW.snapshot_asked.set(1),
        Action::Cycle { byte } => {
            if byte == 4 || REPLAY_WORK.header_one_player.get() != 1 {
                cycle_byte(byte as usize)?;
            }
        }
        Action::Flip { byte } => {
            crate::race::keep_undo::keep_undo();
            let block = REPLAY_VIEW.replay_block.get();
            if block != REPLAY_FILE.index_block_count.get() {
                let byte = (crate::race::viewer_buffers_store::VIEWER_BUFFERS
                    .block_cameras
                    .addr())
                .wrapping_add(byte)
                .wrapping_add(block.wrapping_mul(8));
                let at = crate::pointers::host(byte);
                let old = unsafe { at.read() };
                unsafe { at.write(old ^ 1) };
                crate::race::spread_camera::spread_camera(byte, old)?;
            }
        }
        Action::EditAndStep { camera, back } => {
            crate::race::edit_camera::edit_camera(camera);
            crate::race::edit_camera::step_camera(if back { -1 } else { 1 })?;
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .camera_edit
                .set(0);
        }
        Action::Edit { camera } => crate::race::edit_camera::edit_camera(camera),
        Action::Step { back } => crate::race::edit_camera::step_camera(if back { -1 } else { 1 })?,
        Action::EndEdit => crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .camera_edit
            .set(0),
        Action::Pictures { time } => {
            crate::race::start_frame_pictures::start_frame_pictures(time)?;
            crate::race::play_marked::play_marked()?;
            crate::race::stop_frame_pictures::stop_frame_pictures();
        }
    }
    Ok(true)
}

/// A frame read (`drive::read_replay_frame`) after the block and frame are stored.
fn read_at(r: &mut Carried, block: u32, frame: u32) -> Result<(), Refused> {
    REPLAY_VIEW.replay_block.set(block);
    REPLAY_VIEW.replay_frame.set(frame);
    r.arg = block;
    r.palette = frame;
    r.carry = { crate::drive::read_replay_frame::read_replay_frame(block, frame) }?;
    Ok(())
}

/// `drive::read_replay_frame_0` of the block in `arg`; the carry says it failed.
fn read_replay_frame_0(r: &mut Carried) -> Result<(), Refused> {
    r.carry = { crate::drive::read_replay_frame_0::read_replay_frame_0(r.arg) }?;
    Ok(())
}

/// 0xF, 0x10: the block's camera byte `byte` one on (`viewer::next_byte`), spread on.
fn cycle_byte(byte: usize) -> Result<(), Refused> {
    crate::race::keep_undo::keep_undo();
    let block = REPLAY_VIEW.replay_block.get();
    if block == REPLAY_FILE.index_block_count.get() {
        return Ok(());
    }
    let byte_at = (crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_cameras
        .addr())
    .wrapping_add(byte as u32)
    .wrapping_add(block.wrapping_mul(8));
    let at = crate::pointers::host(byte_at);
    let old = unsafe { at.read() };
    let dl = rc2k_core::race::viewer::next_byte(byte as u32, old);
    unsafe { at.write(dl) };
    crate::race::spread_camera::spread_camera(byte_at, old)
}

/// 0x11: the marked blocks cut (`viewer::cut_marks`).
fn cut(_r: &mut Carried) -> Result<(), Refused> {
    let Some((first, second)) = rc2k_core::race::viewer::cut_marks(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .mark_in
            .get(),
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .mark_out
            .get(),
        REPLAY_FILE.index_block_count.get(),
    )?
    else {
        return Ok(());
    };
    crate::race::keep_undo::keep_undo();
    // `copy_marked` copies whenever `cut_marks` cuts (the same order test, the marks
    // unchanged by `keep_undo`): the original's edx fallback is never taken.
    let Some(count) = crate::race::copy_marked::copy_marked()? else {
        return Err(crate::errors::BadInput("a cut whose marks do not copy").into());
    };
    REPLAY_FILE
        .index_block_count
        .set(REPLAY_FILE.index_block_count.get().wrapping_sub(count));
    let rest = (REPLAY_FILE.index_block_count.get() + count) - second;
    use crate::race::viewer::{clear_cells, move_cells};
    for table in [
        &REPLAY_FILE.block_places as &dyn crate::math::cells::Cells,
        &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras,
    ] {
        let (first, second, rest) = (2 * first as usize, 2 * second as usize, 2 * rest as usize);
        move_cells(table, first, table, second, rest)?;
        clear_cells(table, first + rest, 2 * count as usize)?;
    }
    crate::race::reset_marks::reset_marks()
}

/// 0x12: the clipboard pasted at the block shown (`viewer::pastes`).
fn paste() -> Result<(), Refused> {
    let clip = crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .clip_blocks
        .get();
    let (block, blocks) = (
        REPLAY_VIEW.replay_block.get(),
        REPLAY_FILE.index_block_count.get(),
    );
    if !rc2k_core::race::viewer::pastes(clip, blocks, block)? {
        return Ok(());
    }
    crate::race::keep_undo::keep_undo();
    // The blocks from the one shown up by the clipboard's count, from the top down.
    // (The original's `std; rep movsd` from the last dword, `dword_5BECD4` and
    // `unk_963BB0` + count * 8, down: a move that may overlap.)
    use crate::race::viewer::{move_cells, move_cells_back};
    let v = &crate::race::viewer_buffers_store::VIEWER_BUFFERS;
    let (at, clip_dwords) = (2 * block as usize, 2 * clip as usize);
    let tables = [
        (
            &REPLAY_FILE.block_places as &dyn crate::math::cells::Cells,
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
    REPLAY_FILE.index_block_count.set(blocks + clip);
    crate::race::reset_marks::reset_marks()
}

/// 0x14: another replay chosen to view.
fn another_replay(r: &mut Carried) -> Result<(), Refused> {
    if REPLAY_VIEW.last_race_viewed.get() == 1 {
        return Ok(());
    }
    crate::race::viewer_tidy_stage::viewer_tidy_stage()?;
    loop {
        r.arg = 0;
        r.arg = crate::menu::replay_slot_loop::replay_slot_loop(r)?;
        if r.arg == u32::MAX {
            if REPLAY_FILE.replay_open.get() == 1 {
                return crate::race::viewer_load_stage::viewer_load_stage();
            }
            INSTALLER_PAGES.list_leave.set(1);
            return Ok(());
        }
        REPLAY_FILE.viewer_slot.set(r.arg);
        crate::race::close_viewed_replay::close_viewed_replay()?;
        r.palette = r.arg;
        r.carry = crate::race::open_viewed_replay::open_viewed_replay(r.palette)?;
        if !r.carry {
            return crate::race::viewer_load_stage::viewer_load_stage();
        }
    }
}

/// 0x15, 0x16: saved back into its slot (`back`, a slot below 100) or into one chosen.
fn save(r: &mut Carried, back: bool) -> Result<(), Refused> {
    if REPLAY_FILE.index_block_count.get() == 0 {
        return Ok(());
    }
    if back && REPLAY_FILE.viewer_slot.get() < 100 {
        r.palette = REPLAY_FILE.viewer_slot.get();
    } else {
        r.arg = 1;
        r.arg = crate::race::choose_save_slot::choose_save_slot(r)?;
        if r.arg == u32::MAX {
            return Ok(());
        }
        REPLAY_FILE.viewer_slot.set(r.arg);
        r.palette = r.arg;
    }
    // The carry the original leaves says it was not saved.
    r.carry = !crate::race::save_viewed_replay::save_viewed_replay(r.palette)?;
    if r.carry {
        return Ok(());
    }
    read_at(
        r,
        REPLAY_VIEW.replay_block.get(),
        REPLAY_VIEW.replay_frame.get(),
    )
}
