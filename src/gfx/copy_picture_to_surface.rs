//! Copies the picture buffer onto the surface, row by row: the background of a menu frame.
//! If the surface is not locked (`locked.surface` is 0) it is locked first, and nothing is copied
//! if that fails; it is then unlocked again afterwards. The copy takes `height` rows of
//! `row_dwords` dwords from the buffer, which it reads without gaps, and after each row moves on
//! `row_gap` more bytes on the surface. All of these are read after the lock, which sets them. The
//! lock and the unlock (`gfx::lock_surface`, `gfx::unlock_surface`) are reached through
//! `PictureSurface`, which is where a build without a display puts its own surface. Refuses a row
//! count of 0 and null pointers. When the GPU keeps the background (`PictureSurface::
//! defer_background`) nothing is copied: the picture buffer is noted as the background instead.

use crate::errors::BadInput;
use crate::gfx::owner::Pictures;
use crate::gfx::picture::{has_picture, with_buffer_unchanged};

// The surface's height (rows), `row_dwords` and `row_gap` (surface bytes after a row's pixels) are
// the draw surface's (`Gfx.drawing.draw_surface`).

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
    fn pictures(&self) -> &Pictures;
    /// Notes the picture as the frame's background for the GPU to draw under the canvas, if
    /// this surface is the screen and the GPU keeps it (`render::background`); false: copy it.
    fn defer_background(&mut self) -> bool {
        false
    }
    /// Notes the picture as the menu's background for the wash beside a menu's frame, if this
    /// surface is the screen (`render::wash`); `rows` rows of `row_bytes` bytes.
    fn note_wash(&mut self, _rows: u32, _row_bytes: usize) {}
}

/// The game's own.
pub struct GameSurface<'a>(
    pub &'a crate::gfx::owner::Gfx,
    pub &'a crate::system::owner::System,
);

impl PictureSurface for GameSurface<'_> {
    fn locked(&self) -> bool {
        self.0.locked.surface.is_set()
    }
    fn lock(&mut self) -> bool {
        // Locked.
        crate::platform::lock_surface(self.0, self.1) >= 1
    }
    fn unlock(&mut self) {
        crate::platform::unlock_surface(self.0);
    }
    fn pixels(&self) -> u32 {
        crate::gfx::surface::handle(self.0)
            .unwrap_or_else(|e| crate::fatal::raise::end("gfx::copy_picture_to_surface", e))
    }
    fn rows(&self) -> u32 {
        self.0.drawing.draw_surface.height.get()
    }
    fn row_dwords(&self) -> u32 {
        self.0.drawing.draw_surface.row_dwords.get()
    }
    fn row_gap(&self) -> u32 {
        self.0.drawing.draw_surface.row_gap.get()
    }
    fn pictures(&self) -> &Pictures {
        &self.0.pictures
    }
    fn note_wash(&mut self, rows: u32, row_bytes: usize) {
        let Some(target) = crate::platform::direct3d::screen(self.1.session.screen.get()) else {
            return;
        };
        if self.pixels() == target.pixels
            && target.bits == 32
            && self.row_gap() == 0
            && row_bytes == target.pitch
        {
            crate::render::wash::note_buffer(
                &self.0.pictures.buffers,
                self.0.pictures.picture.buffer.get(),
                rows,
                row_bytes,
            );
        }
    }
    fn defer_background(&mut self) -> bool {
        let Some(target) = crate::platform::direct3d::screen(self.1.session.screen.get()) else {
            return false;
        };
        let whole_rows = self.row_gap() == 0 && self.row_dwords() as usize * 4 == target.pitch;
        self.pixels() == target.pixels
            && self.rows() as usize == target.height
            && whole_rows
            && crate::render::background::defer(
                &self.0.pictures.buffers,
                &target,
                self.0.pictures.picture.buffer.get(),
            )
    }
}

pub fn copy_picture_to_surface(surface: &mut dyn PictureSurface) -> Result<(), BadInput> {
    let was_locked = surface.locked();
    if !was_locked && !surface.lock() {
        return Ok(());
    }
    let to = surface.pixels();
    let (rows, row_bytes) = (surface.rows(), surface.row_dwords() as usize * 4);
    let refused = if !has_picture(surface.pictures()) || to == 0 {
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
    surface.note_wash(rows, row_bytes);
    if surface.defer_background() {
        note_background(surface, to, rows, row_bytes);
        if !was_locked {
            surface.unlock();
        }
        return Ok(());
    }
    // The surface's rows: their dwords, then the gap to the next.
    let to_pixels = to;
    let to = crate::gfx::rows::Rows::new(to, row_bytes as u32 + surface.row_gap(), rows);
    // The picture's rows from its buffer (`gfx::picture`), onto the surface's.
    let copied = with_buffer_unchanged(
        &surface.pictures().buffers,
        surface.pictures().picture.buffer.get(),
        |from| {
            for y in 0..rows {
                let at = y as usize * row_bytes;
                let row = from
                    .get(at..at + row_bytes)
                    .ok_or(BadInput("a picture's row past its buffer"))?;
                to.write(y, 0, row, "a picture's row onto the surface")?;
            }
            Ok::<(), BadInput>(())
        },
    )
    .and_then(|r| r);
    if let Err(why) = copied {
        if !was_locked {
            surface.unlock();
        }
        return Err(why);
    }
    note_background(surface, to_pixels, rows, row_bytes);
    if !was_locked {
        surface.unlock();
    }
    Ok(())
}

/// Notes the picture drawn on the screen for the 2D record (and what it wrote there).
fn note_background(surface: &dyn PictureSurface, to: u32, rows: u32, row_bytes: usize) {
    use crate::gfx::draw2d::{Background, Command, mirror, push};
    if !crate::gfx::draw2d::enabled() || to != crate::gfx::locked_pixels::SCREEN {
        return;
    }
    let buffer = surface.pictures().picture.buffer.get();
    let Some(generation) =
        crate::gfx::draw2d::picture_generation(&surface.pictures().buffers, buffer)
    else {
        return;
    };
    let pitch = row_bytes as u32 + surface.row_gap();
    let _ = with_buffer_unchanged(&surface.pictures().buffers, buffer, |from| {
        for y in 0..rows {
            if let Some(row) = from.get(y as usize * row_bytes..(y as usize + 1) * row_bytes) {
                mirror(y * pitch, row);
            }
        }
    });
    push(Command::Background(Background {
        buffer,
        generation,
        rows,
        row_bytes,
        pitch,
    }));
}
