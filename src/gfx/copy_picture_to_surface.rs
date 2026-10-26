//! `gfx::copy_picture_to_surface` (0x4060E3): copies the picture buffer onto the surface, row
//! by row: the background of a menu frame.
//!
//! In:  nothing
//! Out: every register preserved
//! If the surface is not locked (`dword_4F3870` is 0) it is locked first, and nothing is
//! copied if that fails; it is then unlocked again afterwards. The copy takes
//! `dword_4F49A4` rows of `dword_4F49B4` dwords from the buffer, which it reads without
//! gaps, and after each row moves on `dword_4F49C4` more bytes on the surface. All of these
//! are read after the lock, which sets them (`sub_405726`).
//! Callers: `menu::frame_callback`, `menu::draw_frame` and a dozen more.
//! The lock and the unlock are the game's (`gfx::lock_surface` 0x4058AD,
//! `gfx::unlock_surface` 0x405949, DirectDraw calls); the Rust version reaches them through
//! `PictureSurface`, which is where a build without the game puts its own surface.
//! Refuses a row count of 0 (the original counts through all 32 bits) and null pointers. The
//! name is a reading.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, rva};
use crate::gfx::picture::picture;
use crate::gfx::surface::BASE_RVA;

// ---- game data ----

const LOCKED_RVA: usize = rva(0x4F3870); // dword_4F3870: 1 while the surface is locked
const ROWS_RVA: usize = rva(0x4F49A4); // dword_4F49A4
const ROW_DWORDS_RVA: usize = rva(0x4F49B4); // dword_4F49B4
const ROW_GAP_RVA: usize = rva(0x4F49C4); // dword_4F49C4: surface bytes after a row's pixels
const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface

// ---- Rust version ----

/// The surface the picture goes to, and the picture.
pub trait PictureSurface {
    fn locked(&self) -> bool;
    /// Returns whether the surface is now locked.
    fn lock(&mut self) -> bool;
    fn unlock(&mut self);
    /// What the copy reads, valid once the surface is locked.
    fn picture(&self) -> *const u8;
    fn pixels(&self) -> *mut u8;
    fn rows(&self) -> u32;
    fn row_dwords(&self) -> u32;
    fn row_gap(&self) -> u32;
}

/// The game's own.
pub struct GameSurface;

impl PictureSurface for GameSurface {
    fn locked(&self) -> bool {
        unsafe { game_read::<u32>(LOCKED_RVA) != 0 }
    }
    fn lock(&mut self) -> bool {
        let mut g = Regs::default();
        crate::calls::call(LOCK_SURFACE, &mut g);
        !g.carry()
    }
    fn unlock(&mut self) {
        let mut g = Regs::default();
        crate::calls::call(UNLOCK_SURFACE, &mut g);
    }
    fn picture(&self) -> *const u8 {
        picture()
    }
    fn pixels(&self) -> *mut u8 {
        unsafe { game_read::<u32>(BASE_RVA) as *mut u8 }
    }
    fn rows(&self) -> u32 {
        unsafe { game_read(ROWS_RVA) }
    }
    fn row_dwords(&self) -> u32 {
        unsafe { game_read(ROW_DWORDS_RVA) }
    }
    fn row_gap(&self) -> u32 {
        unsafe { game_read(ROW_GAP_RVA) }
    }
}

pub unsafe fn copy_picture_to_surface(surface: &mut dyn PictureSurface) -> Result<(), BadInput> {
    let was_locked = surface.locked();
    if !was_locked && !surface.lock() {
        return Ok(());
    }
    let (mut from, mut to) = (surface.picture(), surface.pixels());
    let (rows, row_bytes) = (surface.rows(), surface.row_dwords() as usize * 4);
    let refused = if from.is_null() || to.is_null() {
        Some(BadInput("no picture buffer or no surface"))
    } else if rows == 0 {
        Some(BadInput("a surface without rows"))
    } else {
        None
    };
    if let Some(why) = refused {
        // Nothing is written yet; the surface is left as it was found.
        if !was_locked {
            surface.unlock();
        }
        return Err(why);
    }
    let gap = surface.row_gap() as usize;
    for _ in 0..rows {
        unsafe {
            std::ptr::copy(from, to, row_bytes);
            from = from.add(row_bytes);
            to = to.wrapping_add(row_bytes).wrapping_add(gap);
        }
    }
    if !was_locked {
        surface.unlock();
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    if let Err(e) = unsafe { copy_picture_to_surface(&mut GameSurface) } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: a copy of the original whose lock and unlock are stand-ins and whose six
// variables are the test's, copying between two buffers of the test ----
