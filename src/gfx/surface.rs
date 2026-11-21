//! The global drawing surface and clip rectangle that the game's blitters read. The surface is a
//! `Target`. The screen's and a locked texture's pixels are reached by a handle
//! (`gfx::locked_pixels`; `draw_surface.pixels`, set by the platform's lock and
//! `d3d::texture_surface_info`); the picture buffer, the menus' surface after
//! `gfx::picture_as_surface`, by its number (`Drawing::on_picture`, `draw_surface.pixels` 0 then).

use crate::errors::BadInput;
use crate::gfx::owner::Gfx;
use std::sync::atomic::Ordering::Relaxed;

/// The surface's pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The screen or a locked texture, by its pixels' handle (`gfx::locked_pixels`).
    Locked(u32),
    /// A picture buffer (`gfx::picture`), by its number.
    Picture(u32),
}

/// The surface now; `None` for none (no pixels).
pub fn target(gfx: &Gfx) -> Option<Target> {
    match (
        gfx.drawing.on_picture.load(Relaxed),
        gfx.drawing.draw_surface.pixels.get(),
    ) {
        (0, 0) => None,
        (0, a) => Some(Target::Locked(a)),
        (n, _) => Some(Target::Picture(n)),
    }
}

/// The surface's pixels' handle (`gfx::locked_pixels`; 0 for none), for code that only ever draws
/// on the screen or a texture: refused while the picture buffer is the surface.
pub fn handle(gfx: &Gfx) -> Result<u32, BadInput> {
    match target(gfx) {
        Some(Target::Picture(_)) => Err(BadInput(
            "the picture buffer as the surface of code that draws on the screen only",
        )),
        Some(Target::Locked(a)) => Ok(a),
        None => Ok(0),
    }
}

/// The surface made an address one (the platform's lock, a texture's).
pub fn on_address(gfx: &Gfx) {
    gfx.drawing.on_picture.store(0, Relaxed);
}

/// `bytes` written at `offset` from the target's start; refused past its rows (pitch x height) or
/// its memory.
pub fn write(target: Target, offset: u32, bytes: &[u8], gfx: &Gfx) -> Result<(), BadInput> {
    let drawing = &gfx.drawing;
    match target {
        Target::Locked(a) => crate::gfx::rows::Rows::new(
            a,
            drawing.draw_surface.pitch.get(),
            drawing.draw_surface.height.get(),
        )
        .byte(offset)
        .write_bytes(bytes, "the surface"),
        Target::Picture(n) => {
            crate::gfx::picture::write_bytes(&gfx.pictures.buffers, n, offset as usize, bytes)
        }
    }
}

/// Where pixels go: `target`, `None` when no surface is set.
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub target: Option<Target>,
    /// Bytes from one row to the next.
    pub pitch: i32,
    pub bytes_per_pixel: i32,
    pub bits_per_pixel: u32,
}

/// The clip rectangle, all four edges inclusive.
#[derive(Clone, Copy, Debug)]
pub struct Clip {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

pub fn surface(gfx: &Gfx) -> Surface {
    let drawing = &gfx.drawing;
    Surface {
        target: target(gfx),
        pitch: drawing.draw_surface.pitch.get() as i32,
        bytes_per_pixel: drawing.draw_surface.bytes_per_pixel.get() as i32,
        bits_per_pixel: drawing.draw_surface.bits_per_pixel.get(),
    }
}

pub fn clip(gfx: &Gfx) -> Clip {
    let drawing = &gfx.drawing;
    Clip {
        left: drawing.blit.clip.left.get() as i32,
        right: drawing.blit.clip.right.get() as i32,
        top: drawing.blit.clip.top.get() as i32,
        bottom: drawing.blit.clip.bottom.get() as i32,
    }
}

#[cfg(test)]
mod target_tests {
    use super::*;
    use crate::gfx::picture::{Buffer, give_back, read_bytes, take};

    #[test]
    fn the_picture_is_a_target_by_number() {
        let gfx = crate::test_support::own::gfx();
        let drawing = &gfx.drawing;
        let buffers = &gfx.pictures.buffers;
        let _lock = crate::test_support::test_lock();
        let figures = crate::memory::figures();
        let kept = (
            drawing.on_picture.load(Relaxed),
            drawing.draw_surface.pixels.get(),
        );
        let memory = crate::memory::PlainList::empty();
        let list = &memory;
        let n = take(buffers, Buffer::Overlay, list, 0x100).unwrap();
        drawing.on_picture.store(n, Relaxed);
        drawing.draw_surface.pixels.set(0);
        assert_eq!(target(gfx), Some(Target::Picture(n)));
        assert!(handle(gfx).is_err(), "screen-only code refuses the picture");
        write(target(gfx).unwrap(), 8, &[9, 8, 7], gfx).unwrap();
        assert_eq!(read_bytes(buffers, n, 8, 3).unwrap(), [9, 8, 7]);
        on_address(gfx);
        assert_eq!(target(gfx), None);
        give_back(buffers, Buffer::Overlay);
        drawing.on_picture.store(kept.0, Relaxed);
        drawing.draw_surface.pixels.set(kept.1);
        crate::memory::with_figures(|f| *f = figures);
    }
}
