//! `gfx::blit_bar` (0x46B574): draws a bar filled to a percentage: the fill sprite with the
//! clip rectangle's right edge pulled in to where the fill ends, then an end piece there.
//!
//! In:  nothing in registers; the bar is described by six globals (see `Bar`)
//! Out: every register preserved
//! Nothing is drawn at 0 percent. If width * percent is 0 with a non-zero percent the original
//! reports fatal error 0x80701010. The blitter is given the fill's end x as its palette,
//! which only works because these sprites do not use one; it is reproduced as is.
//! Also leaves the fill's end x in `seconds` (0x7559AC), a work variable.

use crate::errors::FatalError;
use crate::gfx::bar_store::BAR;
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::surface::clip;
use crate::menu::draw_work_store::DRAW_WORK;

// ---- game data ----

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
        Self {
            x: BAR.x.get() as i32,
            y: BAR.y.get() as i32,
            width: BAR.width.get() as i32,
            fill_glyph: BAR.fill_glyph.get(),
            end_glyph: BAR.end_glyph.get(),
            percent: BAR.percent.get() as i32,
        }
    }

    pub fn write(&self) {
        BAR.x.set(self.x as u32);
        BAR.y.set(self.y as u32);
        BAR.width.set(self.width as u32);
        BAR.fill_glyph.set(self.fill_glyph);
        BAR.end_glyph.set(self.end_glyph);
        BAR.percent.set(self.percent as u32);
    }
}

/// What the original reports when width * percent is 0. Its message pointer is null.
pub const BAR_HAS_NO_LENGTH: FatalError = FatalError {
    code: 0x8070_1010,
    message: "",
};

pub fn set_bar_x(x: i32) {
    BAR.x.set(x as u32);
}

// ---- Rust version ----

/// Where a blitter call goes: the game's surface, or a list.
pub type Blit<'a> = &'a mut dyn FnMut(BlitCall) -> Result<(), FatalError>;

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
        crate::gfx::blit::BLIT.clip.right.set(x as u32);
    }
    fn keep_fill_end(&mut self, x: i32) {
        DRAW_WORK.seconds.set(x as u32);
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
