//! `gfx::blit_marker_pair` (0x46D098): draws glyphs 0x9C and 0x9D of the sprite table at the
//! two points stored in an object.
//!
//! In:  edi = object (first point at +0x44/+0x48, second at +0x4C/+0x50), ebx = palette
//! Out: every register preserved
//! Each point is also stored in the globals `g_markerX`/`g_markerY`, which are left holding
//! the second one. What the object and the two glyphs are is not known yet.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::FatalError;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::gfx::blit_glyph::{BlitCall, blit_glyph_on_surface};

// ---- game data ----

pub const MARKER_X_RVA: usize = rva(0x755A18); // g_markerX
pub const MARKER_Y_RVA: usize = rva(0x755A1C); // g_markerY
const SPRITE_TABLE_PTR_RVA: usize = rva(0x755581); // g_spriteTable: pointer to a glyph table

const FIRST_GLYPH: u32 = 0x9C;
const SECOND_GLYPH: u32 = 0x9D;

pub fn sprite_table() -> u32 {
    unsafe { game_read(SPRITE_TABLE_PTR_RVA) }
}

// ---- Rust version ----

/// The two blitter calls the original makes for `object`.
pub unsafe fn marker_calls(object: *const u8, palette: u32, table: u32) -> [BlitCall; 2] {
    let coord = |at: usize| unsafe { object.add(at).cast::<i32>().read_unaligned() };
    [
        BlitCall::new(FIRST_GLYPH, coord(0x44), coord(0x48), palette, table),
        BlitCall::new(SECOND_GLYPH, coord(0x4C), coord(0x50), palette, table),
    ]
}

/// Stores a marker's position in the two globals, as the original does before each blit.
pub unsafe fn set_marker_position(call: &BlitCall) {
    unsafe {
        game_write(MARKER_X_RVA, call.x);
        game_write(MARKER_Y_RVA, call.y);
    }
}

/// The position the two globals hold.
pub fn marker_position() -> (i32, i32) {
    unsafe { (game_read(MARKER_X_RVA), game_read(MARKER_Y_RVA)) }
}

/// Draws both markers on the game's surface.
pub unsafe fn blit_marker_pair(object: *const u8, palette: u32) -> Result<(), FatalError> {
    for call in unsafe { marker_calls(object, palette, sprite_table()) } {
        unsafe {
            game_write(MARKER_X_RVA, call.x);
            game_write(MARKER_Y_RVA, call.y);
            blit_glyph_on_surface(
                call.glyph,
                call.table as *const u8,
                call.x,
                call.y,
                call.palette as *const u32,
            )?;
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let (object, palette) = (r.edi as *const u8, r.ebx);

    // A fatal error in the blitter: the original, run from the start, meets the same one.
    if let Err(e) = unsafe { blit_marker_pair(object, palette) } {
        return Err(e.into());
    }
    Ok(())
}
