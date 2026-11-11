//! `gfx::copy_picture_to_surface` (0x4060E3): copies the picture buffer onto the surface, row
//! by row: the background of a menu frame.
//!
//! If the surface is not locked (`dword_4F3870` is 0) it is locked first, and nothing is
//! copied if that fails; it is then unlocked again afterwards. The copy takes
//! `dword_4F49A4` rows of `dword_4F49B4` dwords from the buffer, which it reads without
//! gaps, and after each row moves on `dword_4F49C4` more bytes on the surface. All of these
//! are read after the lock, which sets them (0x405726).
//! Callers: `menu::frame_callback`, `menu::draw_frame` and a dozen more.
//! The lock and the unlock are the game's (`gfx::lock_surface` 0x4058AD,
//! `gfx::unlock_surface` 0x405949, DirectDraw calls); it reaches them through
//! `PictureSurface`, which is where a build without the game puts its own surface.
//! Refuses a row count of 0 (the original counts through all 32 bits) and null pointers. The
//! name is a reading.

use crate::errors::BadInput;
use crate::gfx::owner::gfx;
use crate::gfx::picture::{has_picture, with_picture};

// ---- game data ----

// The surface's height (dword_4F49A4: rows), row_dwords (dword_4F49B4) and row_gap
// (dword_4F49C4: surface bytes after a row's pixels) are `DRAW_SURFACE`'s.

/// The surface the picture goes to, and the picture.
pub trait PictureSurface {
    fn locked(&self) -> bool;
    /// Returns whether the surface is now locked.
    fn lock(&mut self) -> bool;
    fn unlock(&mut self);
    /// The first row's game address (0 for none).
    fn pixels(&self) -> u32;
    fn rows(&self) -> u32;
    fn row_dwords(&self) -> u32;
    fn row_gap(&self) -> u32;
}

/// The game's own.
pub struct GameSurface;

impl PictureSurface for GameSurface {
    fn locked(&self) -> bool {
        gfx().locked.surface.is_set()
    }
    fn lock(&mut self) -> bool {
        // `cmp g_surfaceLocked, 1`'s carry clear: locked.
        crate::platform::lock_surface() >= 1
    }
    fn unlock(&mut self) {
        crate::platform::unlock_surface();
    }
    fn pixels(&self) -> u32 {
        crate::gfx::surface::address()
            .unwrap_or_else(|e| crate::fatal::raise::end("gfx::copy_picture_to_surface", e))
    }
    fn rows(&self) -> u32 {
        gfx().drawing.draw_surface.height.get()
    }
    fn row_dwords(&self) -> u32 {
        gfx().drawing.draw_surface.row_dwords.get()
    }
    fn row_gap(&self) -> u32 {
        gfx().drawing.draw_surface.row_gap.get()
    }
}

pub fn copy_picture_to_surface(surface: &mut dyn PictureSurface) -> Result<(), BadInput> {
    let was_locked = surface.locked();
    if !was_locked && !surface.lock() {
        return Ok(());
    }
    let to = surface.pixels();
    let (rows, row_bytes) = (surface.rows(), surface.row_dwords() as usize * 4);
    let refused = if !has_picture() || to == 0 {
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
    // The surface's rows: their dwords, then the gap to the next.
    let to = crate::gfx::rows::Rows::new(to, row_bytes as u32 + surface.row_gap(), rows);
    // The picture's rows from its buffer (`gfx::picture`), onto the surface's.
    let copied = with_picture(|from| -> Result<(), BadInput> {
        for y in 0..rows {
            let at = y as usize * row_bytes;
            let row = from
                .get(at..at + row_bytes)
                .ok_or(BadInput("a picture's row past its buffer"))?;
            to.write(y, 0, row, "a picture's row onto the surface")?;
        }
        Ok(())
    })
    .and_then(|r| r);
    if let Err(why) = copied {
        if !was_locked {
            surface.unlock();
        }
        return Err(why);
    }
    if !was_locked {
        surface.unlock();
    }
    Ok(())
}
