//! `menu::load_background` (0x47EFFF): loads a menu screen's background picture and draws
//! the frame sprite onto it.
//!
//! In:  eax = the picture's path (may be written to, see `files::swap_missing_extension`);
//!      ebx as it comes: it is the palette the sprite is drawn with
//! Out: every register preserved. The carry flag is whatever the blitter leaves: the
//!      callers test it (and call a set carry fatal error 0x80712345), but it is no result
//! of the loading. The Rust version returns with it clear, as the hooked blitter does.
//! `gfx::load_picture`, whose result is not looked at; `gfx::convert_picture`;
//! `g_pictureFlags` = the screen mode's flags (`dword_4F4934`) and 0xFFF;
//! `gfx::picture_as_surface`; sprite 0xA0 of `g_spriteTable` at (0, 0). 76 callers, nearly all
//! of them screen loops. Refuses what the functions it calls refuse.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::block::{game_read, game_write};
use crate::gfx::convert_picture::convert_picture_now;
use crate::gfx::load_picture::load_picture;
use crate::gfx::picture::FLAGS_RVA;
use crate::gfx::picture_as_surface::picture_as_surface;
use crate::text::draw_text::Canvas;

// ---- game data ----

const MODE_FLAGS_RVA: usize = rva(0x4F4934); // dword_4F4934: the screen mode's flags
const FRAME_SPRITE: u32 = 0xA0;

// ---- Rust version ----

/// `palette` is ebx at entry.
pub unsafe fn load_background(
    path: *mut u8,
    palette: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        // Whether the file was read is not looked at.
        load_picture(path)?;
        convert_picture_now()?;
        game_write(FLAGS_RVA, game_read::<u32>(MODE_FLAGS_RVA) & 0xFFF);
        picture_as_surface()?;
        canvas.blit_glyph(BlitCall::new(FRAME_SPRITE, 0, 0, palette, sprite_table()))?;
        Ok(())
    }
}
