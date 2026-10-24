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
use crate::menu::screen_loop::Carried;
use crate::physics::flow::a;
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERA_EDIT, CAMERAS, CLIP_BLOCKS, CLIP_CAMERAS, CLIP_PLACES, FRAME,
    KEY_AFTER_PLAY, LAST_RACE_VIEWED, LIST_LEAVE, MARK_IN, MARK_OUT, ONE_PLAYER, OPEN, PLACES,
    PLAY_FROM_HERE, SLOT, SNAPSHOT, get, move_dwords, pb, put,
};

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
            r.arg = goto_block(goto, |v| {
                get(match v {
                    Var::Block => BLOCK,
                    Var::Blocks => BLOCKS,
                    Var::MarkIn => MARK_IN,
                    Var::MarkOut => MARK_OUT,
                })
            });
            read_replay_frame_0(r)?;
        }
        Action::MarkIn => {
            if mark_in_here(get(BLOCK), get(MARK_OUT)) {
                put(MARK_IN, get(BLOCK));
            }
        }
        Action::MarkOut => {
            if mark_out_here(get(BLOCK), get(MARK_IN)) {
                put(MARK_OUT, get(BLOCK));
            }
        }
        Action::FrameBack => {
            let (block, frame) = frame_back(get(BLOCK), get(FRAME));
            read_at(r, block, frame)?;
        }
        Action::FrameOn => {
            let (block, frame) = frame_on(get(BLOCK), get(FRAME), get(BLOCKS));
            read_at(r, block, frame)?;
        }
        Action::PlayHere { from_here } => {
            if from_here {
                put(PLAY_FROM_HERE, 1);
            }
            crate::race::play_marked::play_from_here()?;
        }
        Action::PlayMarked => crate::race::play_marked::play_marked()?,
        Action::PlayAgain => loop {
            put(BLOCK, 0);
            put(FRAME, 0);
            crate::race::play_marked::play_from_here()?;
            if get(KEY_AFTER_PLAY) != 0 {
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
                put(CAMERA_EDIT, 0);
            }
        }
        Action::AnotherReplay => another_replay(r)?,
        Action::Save { back } => save(r, back)?,
        Action::Snapshot => put(SNAPSHOT, 1),
        Action::Cycle { byte } => {
            if byte == 4 || get(ONE_PLAYER) != 1 {
                cycle_byte(byte as usize)?;
            }
        }
        Action::Flip { byte } => {
            crate::race::keep_undo::keep_undo();
            let block = get(BLOCK);
            if block != get(BLOCKS) {
                let byte = a(CAMERAS + byte as usize).wrapping_add(block.wrapping_mul(8));
                let at = byte as *mut u8;
                let old = unsafe { at.read() };
                unsafe { at.write(old ^ 1) };
                unsafe { crate::race::spread_camera::spread_camera(byte, old) }?;
            }
        }
        Action::EditAndStep { camera, back } => {
            crate::race::edit_camera::edit_camera(camera);
            unsafe { crate::race::edit_camera::step_camera(if back { -1 } else { 1 }) }?;
            put(CAMERA_EDIT, 0);
        }
        Action::Edit { camera } => crate::race::edit_camera::edit_camera(camera),
        Action::Step { back } => {
            unsafe { crate::race::edit_camera::step_camera(if back { -1 } else { 1 }) }?
        }
        Action::EndEdit => put(CAMERA_EDIT, 0),
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
    put(BLOCK, block);
    put(FRAME, frame);
    r.arg = block;
    r.palette = frame;
    r.carry = unsafe { crate::drive::read_replay_frame::read_replay_frame(block, frame) }?;
    Ok(())
}

/// `drive::read_replay_frame_0` of the block in `arg`; the carry says it failed.
fn read_replay_frame_0(r: &mut Carried) -> Result<(), Refused> {
    r.carry = unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(r.arg) }?;
    Ok(())
}

/// 0xF, 0x10: the block's camera byte `byte` one on (`viewer::next_byte`), spread on.
fn cycle_byte(byte: usize) -> Result<(), Refused> {
    crate::race::keep_undo::keep_undo();
    let block = get(BLOCK);
    if block == get(BLOCKS) {
        return Ok(());
    }
    let byte_at = a(CAMERAS + byte).wrapping_add(block.wrapping_mul(8));
    let at = byte_at as *mut u8;
    let old = unsafe { at.read() };
    let dl = rc2k_core::race::viewer::next_byte(byte as u32, old);
    unsafe { at.write(dl) };
    unsafe { crate::race::spread_camera::spread_camera(byte_at, old) }
}

/// 0x11: the marked blocks cut (`viewer::cut_marks`).
fn cut(_r: &mut Carried) -> Result<(), Refused> {
    let Some((first, second)) =
        rc2k_core::race::viewer::cut_marks(get(MARK_IN), get(MARK_OUT), get(BLOCKS))?
    else {
        return Ok(());
    };
    crate::race::keep_undo::keep_undo();
    // `copy_marked` copies whenever `cut_marks` cuts (the same order test, the marks
    // unchanged by `keep_undo`): the original's edx fallback is never taken.
    let Some(count) = crate::race::copy_marked::copy_marked()? else {
        return Err(crate::errors::BadInput("a cut whose marks do not copy").into());
    };
    put(BLOCKS, get(BLOCKS).wrapping_sub(count));
    let rest = (get(BLOCKS) + count) - second;
    for table in [PLACES, CAMERAS] {
        move_dwords(
            table + 8 * first as usize,
            table + 8 * second as usize,
            2 * rest as usize,
        );
        let end = table + 8 * (first + rest) as usize;
        unsafe { std::ptr::write_bytes(a(end) as *mut u8, 0, 8 * count as usize) };
    }
    crate::race::reset_marks::reset_marks()
}

/// 0x12: the clipboard pasted at the block shown (`viewer::pastes`).
fn paste() -> Result<(), Refused> {
    let clip = get(CLIP_BLOCKS);
    let (block, blocks) = (get(BLOCK), get(BLOCKS));
    if !rc2k_core::race::viewer::pastes(clip, blocks, block)? {
        return Ok(());
    }
    crate::race::keep_undo::keep_undo();
    // The blocks from the one shown up by the clipboard's count, from the top down.
    // (The original's `std; rep movsd` from the last dword, `dword_5BECD4` and
    // `unk_963BB0` + count * 8, down: a move that may overlap.)
    for base in [PLACES, CAMERAS] {
        unsafe {
            std::ptr::copy(
                a(base + 8 * block as usize) as *const u8,
                a(base + 8 * (block + clip) as usize) as *mut u8,
                8 * (blocks - block) as usize,
            );
        }
    }
    move_dwords(PLACES + 8 * block as usize, CLIP_PLACES, 2 * clip as usize);
    move_dwords(
        CAMERAS + 8 * block as usize,
        CLIP_CAMERAS,
        2 * clip as usize,
    );
    put(BLOCKS, blocks + clip);
    crate::race::reset_marks::reset_marks()
}

/// 0x14: another replay chosen to view.
fn another_replay(r: &mut Carried) -> Result<(), Refused> {
    if get(LAST_RACE_VIEWED) == 1 {
        return Ok(());
    }
    crate::race::viewer_tidy_stage::viewer_tidy_stage()?;
    loop {
        r.arg = 0;
        r.arg = crate::menu::replay_slot_loop::replay_slot_loop(r)?;
        if r.arg == u32::MAX {
            if get(OPEN) == 1 {
                return crate::race::viewer_load_stage::viewer_load_stage();
            }
            pb(LIST_LEAVE, 1);
            return Ok(());
        }
        put(SLOT, r.arg);
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
    if get(BLOCKS) == 0 {
        return Ok(());
    }
    if back && get(SLOT) < 100 {
        r.palette = get(SLOT);
    } else {
        r.arg = 1;
        r.arg = crate::race::choose_save_slot::choose_save_slot(r)?;
        if r.arg == u32::MAX {
            return Ok(());
        }
        put(SLOT, r.arg);
        r.palette = r.arg;
    }
    // The carry the original leaves says it was not saved.
    r.carry = !crate::race::save_viewed_replay::save_viewed_replay(r.palette)?;
    if r.carry {
        return Ok(());
    }
    read_at(r, get(BLOCK), get(FRAME))
}
