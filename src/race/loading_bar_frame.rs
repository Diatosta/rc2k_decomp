//! `race::loading_bar_frame` (0x4467C7): one frame of the loading bar while a stage loads
//! (the loading's callback `dword_623704`), with `dword_71C3E0` only: `dword_6365CC` 8;
//! nothing while quitting; with the window active and the surfaces not lost
//! (`dword_518380` not 1): `scene::begin_scene`, the view window chosen as unframed
//! (`dword_621514` 0 for `scene::choose_view_window`, put back), `race::loading_bar_quad`,
//! `scene::end_scene`, `scene::lock_frame`, an empty function, `scene::unlock_frame`,
//! `scene::present_frame`, carry clear. A failed step (or the window inactive, or the
//! surfaces lost) instead, unless quitting, restores the surfaces (`sub_405B10`, the
//! platform's) and leaves carry set when they were neither lost before nor after.
//!
//! In:  nothing
//! Out: CF / ZF as the path leaves them (see the code); the registers as its calls leave
//!      them
//!
//! Caller: the stage's loading (through `dword_623704`). The window is its Rust version;
//! the rest through `part` (1 begin, 2 the bar, 3 end, 4 lock, 5 unlock, 6 show, 7 the
//! restore), in the game their Rust versions (the restore the platform's call).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const ON_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const MARK_RVA: usize = rva(0x6365CC); // dword_6365CC
const QUITTING_RVA: usize = rva(0x51837C); // g_quitting
const ACTIVE_RVA: usize = rva(0x518374); // g_windowActive
const LOST_RVA: usize = rva(0x518380); // dword_518380
const FRAMED_RVA: usize = rva(0x621514); // dword_621514
const RESTORE_RVA: usize = rva(0x405B10); // sub_405B10

fn flags(g: &mut Regs, carry: bool, zero: bool) {
    g.set_carry(carry);
    g.set_zero(zero);
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn loading_bar_frame(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::loading::{FRAME_MARK, frame_tried, restored}`.
    use rc2k_core::race::loading::{FRAME_MARK, frame_tried, restored};
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(ON_RVA) == 0 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        game_write(MARK_RVA, FRAME_MARK);
        if game_read::<u32>(QUITTING_RVA) == 1 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        let drawn = 'frame: {
            let active: u32 = game_read(ACTIVE_RVA);
            if active == 0 || !frame_tried(active, game_read(LOST_RVA)) {
                break 'frame false;
            }
            crate::race::loading_bar_frame::game_part(1, &mut g)?;
            if g.carry() {
                break 'frame false;
            }
            let framed: u32 = game_read(FRAMED_RVA);
            game_write(FRAMED_RVA, 0u32);
            crate::scene::choose_view_window::choose_view_window().apply(&mut g);
            game_write(FRAMED_RVA, framed);
            crate::race::loading_bar_frame::game_part(2, &mut g)?;
            crate::race::loading_bar_frame::game_part(3, &mut g)?;
            crate::race::loading_bar_frame::game_part(4, &mut g)?;
            if g.carry() {
                break 'frame false;
            }
            crate::race::loading_bar_frame::game_part(5, &mut g)?;
            crate::race::loading_bar_frame::game_part(6, &mut g)?;
            !g.carry()
        };
        if drawn {
            g.set_carry(false);
            return Ok(g);
        }
        if game_read::<u32>(QUITTING_RVA) == 1 {
            flags(&mut g, false, true);
            return Ok(g);
        }
        g.ebx = game_read(LOST_RVA);
        crate::race::loading_bar_frame::game_part(7, &mut g)?;
        let (ebx, carry, zero) = restored(g.ebx, game_read(LOST_RVA));
        g.ebx = ebx;
        flags(&mut g, carry, zero);
    }
    Ok(g)
}

/// In the game: the calls' Rust versions (the restore the platform's).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::*;
    unsafe {
        match place {
            1 => begin_scene::begin_scene(r, &mut |r| crate::calls::call(rva(0x41ABC7), r)),
            2 => *r = crate::race::loading_bar_quad::loading_bar_quad(r)?,
            3 => end_scene::end_scene(r, &mut |r| crate::calls::call(rva(0x41AB8A), r)),
            4 => *r = lock_frame::lock_frame(r)?,
            5 => *r = unlock_frame::unlock_frame(r)?,
            6 => *r = present_frame::present_frame(r)?,
            _ => crate::calls::call(RESTORE_RVA, r),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { loading_bar_frame(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
