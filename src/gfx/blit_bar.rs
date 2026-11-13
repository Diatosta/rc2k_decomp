//! Draws a bar filled to a percentage: the fill sprite with the clip rectangle's right edge pulled
//! in to where the fill ends, then an end piece there.
//! Nothing is drawn at 0 percent. If width * percent is 0 with a non-zero percent a fatal error is
//! reported. The blitter is given the fill's end x as its palette, which only works because these
//! sprites do not use one: kept as is. Also leaves the fill's end x in `seconds`, a work variable.

use crate::errors::{FatalError, Refused};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::owner::Gfx;
use crate::gfx::surface::clip;
use crate::menu::owner::Menus;

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
    pub fn read(gfx: &Gfx) -> Self {
        let drawing = &gfx.drawing;
        Self {
            x: drawing.bar.x.get() as i32,
            y: drawing.bar.y.get() as i32,
            width: drawing.bar.width.get() as i32,
            fill_glyph: drawing.bar.fill_glyph.get(),
            end_glyph: drawing.bar.end_glyph.get(),
            percent: drawing.bar.percent.get() as i32,
        }
    }

    /// Puts the bar in the bar globals.
    pub fn write_in(&self, gfx: &Gfx) {
        let drawing = &gfx.drawing;
        drawing.bar.x.set(self.x as u32);
        drawing.bar.y.set(self.y as u32);
        drawing.bar.width.set(self.width as u32);
        drawing.bar.fill_glyph.set(self.fill_glyph);
        drawing.bar.end_glyph.set(self.end_glyph);
        drawing.bar.percent.set(self.percent as u32);
    }
}

/// The fatal error for width * percent being 0. Its message pointer is null.
pub const BAR_HAS_NO_LENGTH: FatalError = FatalError {
    code: 0x8070_1010,
    message: "",
};

/// Where a blitter call goes: the game's surface, or a list.
pub type Blit<'a> = &'a mut dyn FnMut(BlitCall) -> Result<(), Refused>;

/// The game's side of a bar: the blits through `blit`, the clip and the bar's globals.
pub struct GameBar<'a> {
    pub blit: Blit<'a>,
    pub gfx: &'a Gfx,
    pub menus: &'a Menus,
}

impl rc2k_core::gfx::bars::BarCanvas for GameBar<'_> {
    type Error = Refused;
    fn sprite(&mut self, glyph: u32, x: i32, y: i32) -> Result<(), Refused> {
        (self.blit)(BlitCall::sprite(
            self.menus,
            glyph,
            x,
            y,
            clip(self.gfx).right,
        ))
    }
    fn clip_right(&mut self) -> i32 {
        clip(self.gfx).right
    }
    fn set_clip_right(&mut self, x: i32) {
        self.gfx.drawing.blit.clip.right.set(x as u32);
    }
    fn keep_fill_end(&mut self, x: i32) {
        self.menus.screens.draw_work.seconds.set(x as u32);
    }
    fn set_bar_x(&mut self, x: i32) {
        self.gfx.drawing.bar.x.set(x as u32);
    }
    fn no_length(&mut self) -> Refused {
        BAR_HAS_NO_LENGTH.into()
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

/// The bar described by the bar globals.
pub fn blit_bar_in(gfx: &Gfx, menus: &Menus, blit: Blit) -> Result<(), Refused> {
    rc2k_core::gfx::bars::bar(&mut GameBar { blit, gfx, menus }, &Bar::read(gfx).core())
}
