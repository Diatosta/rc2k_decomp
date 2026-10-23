//! `gfx::blit_bar` (0x46B574): draws a bar filled to a percentage: the fill sprite with the
//! clip rectangle's right edge pulled in to where the fill ends, then an end piece there.
//!
//! In:  nothing in registers; the bar is described by six globals (see `Bar`)
//! Out: every register preserved
//! Nothing is drawn at 0 percent. If width * percent is 0 with a non-zero percent the original
//! reports fatal error 0x80701010. The blitter is given the fill's end x as its palette,
//! which only works because these sprites do not use one; it is reproduced as is.
//! Also leaves the fill's end x in `seconds` (0x7559AC), a work variable.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::FatalError;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::gfx::blit_glyph::{BlitCall, blit_glyph_on_surface};
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::surface::{CLIP_RIGHT_RVA, clip};

// ---- game data ----

const BAR_X_RVA: usize = rva(0x755224); // dword_755224
const BAR_Y_RVA: usize = rva(0x755228); // dword_755228
const BAR_WIDTH_RVA: usize = rva(0x75522C); // dword_75522C
const BAR_FILL_GLYPH_RVA: usize = rva(0x755230); // dword_755230
const BAR_END_GLYPH_RVA: usize = rva(0x755234); // dword_755234
const BAR_PERCENT_RVA: usize = rva(0x755238); // dword_755238
const FILL_END_RVA: usize = rva(0x7559AC); // seconds: a work variable, here the fill's end x

/// The six globals a caller fills in before drawing a bar or its marker.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    pub x: i32,
    pub y: i32,
    /// Length of the bar at 100 percent, in pixels.
    pub width: i32,
    /// Sprite of the fill; for `blit_bar_marker`, the marker.
    pub fill_glyph: u32,
    pub end_glyph: u32,
    pub percent: i32,
}

impl Bar {
    pub fn read() -> Self {
        unsafe {
            Self {
                x: game_read(BAR_X_RVA),
                y: game_read(BAR_Y_RVA),
                width: game_read(BAR_WIDTH_RVA),
                fill_glyph: game_read(BAR_FILL_GLYPH_RVA),
                end_glyph: game_read(BAR_END_GLYPH_RVA),
                percent: game_read(BAR_PERCENT_RVA),
            }
        }
    }

    pub fn write(&self) {
        unsafe {
            game_write(BAR_X_RVA, self.x);
            game_write(BAR_Y_RVA, self.y);
            game_write(BAR_WIDTH_RVA, self.width);
            game_write(BAR_FILL_GLYPH_RVA, self.fill_glyph);
            game_write(BAR_END_GLYPH_RVA, self.end_glyph);
            game_write(BAR_PERCENT_RVA, self.percent);
        }
    }
}

/// What the original reports when width * percent is 0. Its message pointer is null.
pub const BAR_HAS_NO_LENGTH: FatalError = FatalError {
    code: 0x8070_1010,
    message: "",
};

pub fn set_bar_x(x: i32) {
    unsafe { game_write(BAR_X_RVA, x) };
}

// ---- Rust version ----

/// Where a blitter call goes: the game's surface, or a list.
pub type Blit<'a> = &'a mut dyn FnMut(BlitCall) -> Result<(), FatalError>;

/// Draws on the game's surface.
pub fn on_surface(call: BlitCall) -> Result<(), FatalError> {
    unsafe {
        blit_glyph_on_surface(
            call.glyph,
            call.table as *const u8,
            call.x,
            call.y,
            call.palette as *const u32,
        )
    }
    .map(|_| ())
}

/// The game's side of a bar: the blits through `blit`, the clip and the bar's globals.
pub struct GameBar<'a>(pub Blit<'a>);

impl rc2k_core::gfx::bars::BarCanvas for GameBar<'_> {
    type Error = FatalError;
    fn sprite(&mut self, glyph: u32, x: i32, y: i32, palette: u32) -> Result<(), FatalError> {
        (self.0)(BlitCall::new(glyph, x, y, palette, sprite_table()))
    }
    fn clip_right(&mut self) -> i32 {
        clip().right
    }
    fn set_clip_right(&mut self, x: i32) {
        unsafe { game_write(CLIP_RIGHT_RVA, x) };
    }
    fn keep_fill_end(&mut self, x: i32) {
        unsafe { game_write(FILL_END_RVA, x) };
    }
    fn set_bar_x(&mut self, x: i32) {
        set_bar_x(x);
    }
    fn no_length(&mut self) -> FatalError {
        BAR_HAS_NO_LENGTH
    }
}

impl Bar {
    /// As the core's.
    pub fn core(&self) -> rc2k_core::gfx::bars::Bar {
        rc2k_core::gfx::bars::Bar {
            x: self.x,
            y: self.y,
            width: self.width,
            fill_glyph: self.fill_glyph,
            end_glyph: self.end_glyph,
            percent: self.percent,
        }
    }
}

/// The bar described by the bar globals. Converted: `rc2k_core::gfx::bars::bar`.
pub unsafe fn blit_bar(blit: Blit) -> Result<(), FatalError> {
    rc2k_core::gfx::bars::bar(&mut GameBar(blit), &Bar::read().core())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    // A fatal error: the original, run from the start, meets the same one.
    if let Err(e) = unsafe { blit_bar(&mut on_surface) } {
        return Err(e.into());
    }
    Ok(())
}
