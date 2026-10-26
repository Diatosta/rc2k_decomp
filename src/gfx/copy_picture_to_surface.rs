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
//! The registers above are the original's, for reference; a call by address goes through
//! `routes.rs`.

use crate::errors::BadInput;
use crate::gfx::locked::LOCKED;
use crate::gfx::picture::picture;
use crate::gfx::screen_state::DRAW_SURFACE;

// ---- game data ----

// The surface's height (dword_4F49A4: rows), row_dwords (dword_4F49B4) and row_gap
// (dword_4F49C4: surface bytes after a row's pixels) are `DRAW_SURFACE`'s.

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
        LOCKED.surface.get() != 0
    }
    fn lock(&mut self) -> bool {
        // `cmp g_surfaceLocked, 1`'s carry clear: locked.
        crate::platform::lock_surface() >= 1
    }
    fn unlock(&mut self) {
        crate::platform::unlock_surface();
    }
    fn picture(&self) -> *const u8 {
        picture()
    }
    fn pixels(&self) -> *mut u8 {
        DRAW_SURFACE.pixels.get() as *mut u8
    }
    fn rows(&self) -> u32 {
        DRAW_SURFACE.height.get()
    }
    fn row_dwords(&self) -> u32 {
        DRAW_SURFACE.row_dwords.get()
    }
    fn row_gap(&self) -> u32 {
        DRAW_SURFACE.row_gap.get()
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
        let copied = crate::pointers::GamePtr::new(from as u32).copy_to(
            crate::pointers::GamePtr::new(to as u32),
            row_bytes,
            "a picture's row onto the surface",
        );
        if let Err(why) = copied {
            if !was_locked {
                surface.unlock();
            }
            return Err(why);
        }
        from = from.wrapping_add(row_bytes);
        to = to.wrapping_add(row_bytes).wrapping_add(gap);
    }
    if !was_locked {
        surface.unlock();
    }
    Ok(())
}
