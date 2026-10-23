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
//! Callers: `race::replay_viewer` (three). The camera bytes' edits (0xE .. 0x10, 0x18) keep
//! the undo before they look at the block; nothing is checked of the table's room for a
//! cut or a paste beyond what the original checks (a block shown past the count is
//! refused for a paste: the original moves memory by a negative count). Its calls
//! through `part` (`CALLEES`); in the game their Rust versions. Ungated (plays and
//! screens). Shadow mode runs the original alone (some actions play, save or load).
//! Self-test: the original against the Rust version on the actions that only change
//! memory (all but the plays, 0x14 .. 0x16 and 0x25 .. 0x28), with `read_replay_frame`'s
//! test replay; the tables set by the test, everything put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG};
use crate::physics::flow::a;
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERA_EDIT, CAMERAS, CLIP_BLOCKS, CLIP_CAMERAS, CLIP_PLACES, FRAME,
    KEY_AFTER_PLAY, LAST_RACE_VIEWED, LIST_LEAVE, MARK_IN, MARK_OUT, ONE_PLAYER, OPEN, PLACES,
    PLAY_FROM_HERE, Part, SLOT, SNAPSHOT, get, move_dwords, pb, put, regs_entry,
};

/// The Rust version: the registers it leaves (ZF: it was an action).
pub fn viewer_action(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut out = *entry;
    let known = action(entry, part)?;
    out.eflags = if known {
        out.eflags | ZERO_FLAG
    } else {
        out.eflags & !ZERO_FLAG
    };
    Ok(out)
}

/// `sub_41DE81`: whether the action is one.
fn action(entry: &Regs, part: Part) -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::viewer::{Action, goto_block, frame_back, frame_on,
    // mark_in_here, mark_out_here}`.
    use rc2k_core::race::viewer::{
        Action, Var, frame_back, frame_on, goto_block, mark_in_here, mark_out_here,
    };
    let mut r = Regs {
        eax: entry.ebx,
        ebp: 0,
        ..*entry
    };
    let Some(action) = Action::of(entry.ebx) else {
        return Ok(false);
    };
    match action {
        Action::Goto(goto) => {
            r.eax = goto_block(goto, |v| {
                get(match v {
                    Var::Block => BLOCK,
                    Var::Blocks => BLOCKS,
                    Var::MarkIn => MARK_IN,
                    Var::MarkOut => MARK_OUT,
                })
            });
            part(1, &mut r)?;
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
            read_at(part, &mut r, block, frame)?;
        }
        Action::FrameOn => {
            let (block, frame) = frame_on(get(BLOCK), get(FRAME), get(BLOCKS));
            read_at(part, &mut r, block, frame)?;
        }
        Action::PlayHere { from_here } => {
            if from_here {
                put(PLAY_FROM_HERE, 1);
            }
            part(3, &mut r)?;
        }
        Action::PlayMarked => part(4, &mut r)?,
        Action::PlayAgain => loop {
            put(BLOCK, 0);
            put(FRAME, 0);
            part(3, &mut r)?;
            if get(KEY_AFTER_PLAY) != 0 {
                break;
            }
        },
        Action::Cut => cut(part, &mut r)?,
        Action::CopyMarked => part(6, &mut r)?,
        Action::Paste => paste(part, &mut r)?,
        Action::Undo { end_edit } => {
            part(8, &mut r)?;
            if end_edit {
                put(CAMERA_EDIT, 0);
            }
        }
        Action::AnotherReplay => another_replay(part, &mut r)?,
        Action::Save { back } => save(part, &mut r, back)?,
        Action::Snapshot => put(SNAPSHOT, 1),
        Action::Cycle { byte } => {
            if byte == 4 || get(ONE_PLAYER) != 1 {
                cycle_byte(part, &mut r, byte as usize)?;
            }
        }
        Action::Flip { byte } => {
            part(5, &mut r)?;
            r.eax = get(BLOCK);
            if r.eax != get(BLOCKS) {
                r.ebx = a(CAMERAS + byte as usize).wrapping_add(r.eax.wrapping_mul(8));
                let at = r.ebx as *mut u8;
                let old = unsafe { at.read() };
                r.edx = r.edx & !0xFF00 | (old as u32) << 8;
                unsafe { at.write(old ^ 1) };
                part(16, &mut r)?;
            }
        }
        Action::EditAndStep { camera, back } => {
            part(16 + camera, &mut r)?;
            part(if back { 20 } else { 19 }, &mut r)?;
            put(CAMERA_EDIT, 0);
        }
        Action::Edit { camera } => part(16 + camera, &mut r)?,
        Action::Step { back } => part(if back { 20 } else { 19 }, &mut r)?,
        Action::EndEdit => put(CAMERA_EDIT, 0),
        Action::Pictures { time } => {
            r.eax = time;
            part(21, &mut r)?;
            part(4, &mut r)?;
            part(22, &mut r)?;
        }
    }
    Ok(true)
}

/// A frame read (`drive::read_replay_frame`) after the block and frame are stored.
fn read_at(part: Part, r: &mut Regs, block: u32, frame: u32) -> Result<(), Refused> {
    put(BLOCK, block);
    put(FRAME, frame);
    r.eax = block;
    r.ebx = frame;
    part(2, r)
}

/// 0xF, 0x10: the block's camera byte `byte` one on (`viewer::next_byte`), spread on.
fn cycle_byte(part: Part, r: &mut Regs, byte: usize) -> Result<(), Refused> {
    part(5, r)?;
    r.eax = get(BLOCK);
    if r.eax == get(BLOCKS) {
        return Ok(());
    }
    r.ebx = a(CAMERAS + byte).wrapping_add(r.eax.wrapping_mul(8));
    let at = r.ebx as *mut u8;
    let old = unsafe { at.read() };
    let dl = rc2k_core::race::viewer::next_byte(byte as u32, old);
    r.edx = r.edx & !0xFFFF | (old as u32) << 8 | dl as u32;
    unsafe { at.write(dl) };
    part(16, r)
}

/// 0x11: the marked blocks cut (`viewer::cut_marks`).
fn cut(part: Part, r: &mut Regs) -> Result<(), Refused> {
    let Some((first, second)) =
        rc2k_core::race::viewer::cut_marks(get(MARK_IN), get(MARK_OUT), get(BLOCKS))?
    else {
        return Ok(());
    };
    part(5, r)?;
    part(6, r)?;
    let count = r.edx;
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
    part(7, r)
}

/// 0x12: the clipboard pasted at the block shown (`viewer::pastes`).
fn paste(part: Part, r: &mut Regs) -> Result<(), Refused> {
    let clip = get(CLIP_BLOCKS);
    let (block, blocks) = (get(BLOCK), get(BLOCKS));
    if !rc2k_core::race::viewer::pastes(clip, blocks, block)? {
        return Ok(());
    }
    part(5, r)?;
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
    part(7, r)
}

/// 0x14: another replay chosen to view.
fn another_replay(part: Part, r: &mut Regs) -> Result<(), Refused> {
    if get(LAST_RACE_VIEWED) == 1 {
        return Ok(());
    }
    part(9, r)?;
    loop {
        r.eax = 0;
        part(10, r)?;
        if r.eax == u32::MAX {
            if get(OPEN) == 1 {
                return part(13, r);
            }
            pb(LIST_LEAVE, 1);
            return Ok(());
        }
        put(SLOT, r.eax);
        part(11, r)?;
        r.ebx = r.eax;
        part(12, r)?;
        if !r.carry() {
            return part(13, r);
        }
    }
}

/// 0x15, 0x16: saved back into its slot (`back`, a slot below 100) or into one chosen.
fn save(part: Part, r: &mut Regs, back: bool) -> Result<(), Refused> {
    if get(BLOCKS) == 0 {
        return Ok(());
    }
    if back && get(SLOT) < 100 {
        r.ebx = get(SLOT);
    } else {
        r.eax = 1;
        part(15, r)?;
        if r.eax == u32::MAX {
            return Ok(());
        }
        put(SLOT, r.eax);
        r.ebx = r.eax;
    }
    part(14, r)?;
    if r.carry() {
        return Ok(());
    }
    r.eax = get(BLOCK);
    r.ebx = get(FRAME);
    part(2, r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::*;
    unsafe {
        match place {
            1 => {
                let failed = crate::drive::read_replay_frame_0::read_replay_frame_0(r.eax)?;
                r.set_carry(failed);
            }
            2 => {
                let failed = crate::drive::read_replay_frame::read_replay_frame(r.eax, r.ebx)?;
                r.set_carry(failed);
            }
            3 => *r = play_marked::play_from_here(r, &mut play_marked::game_part)?,
            4 => *r = play_marked::play_marked(r, &mut play_marked::game_part)?,
            5 => keep_undo::keep_undo(),
            6 => *r = copy_marked::copy_marked(r)?,
            7 => reset_marks::reset_marks()?,
            8 => *r = undo_edit::undo_edit(r)?,
            9 => *r = viewer_tidy_stage::viewer_tidy_stage(r, &mut viewer_tidy_stage::game_part)?,
            10 => {
                *r = crate::menu::replay_slot_loop::replay_slot_loop(
                    r,
                    &mut crate::menu::replay_slot_loop::game_part,
                )?
            }
            11 => {
                *r = close_viewed_replay::close_viewed_replay(
                    r,
                    &mut close_viewed_replay::game_part,
                )?
            }
            12 => {
                *r = open_viewed_replay::open_viewed_replay(r, &mut open_viewed_replay::game_part)?
            }
            13 => *r = viewer_load_stage::viewer_load_stage(r, &mut viewer_load_stage::game_part)?,
            14 => {
                *r = save_viewed_replay::save_viewed_replay(r, &mut save_viewed_replay::game_part)?
            }
            15 => *r = choose_save_slot::choose_save_slot(r, &mut choose_save_slot::game_part)?,
            16 => *r = spread_camera::spread_camera(r)?,
            17 => *r = edit_camera::edit_camera(r, 1),
            18 => *r = edit_camera::edit_camera(r, 2),
            19 => *r = edit_camera::step_camera(r, 1)?,
            20 => *r = edit_camera::step_camera(r, -1)?,
            21 => {
                *r = start_frame_pictures::start_frame_pictures(
                    r,
                    &mut start_frame_pictures::game_part,
                )?
            }
            _ => stop_frame_pictures::stop_frame_pictures(),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| viewer_action(r, &mut game_part))
}
