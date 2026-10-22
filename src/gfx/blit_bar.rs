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
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{
    BlitCall, blit_glyph_on_surface, glyph_record, record_calls, recording_works,
};
use crate::gfx::blit_marker_pair::sprite_table;
use crate::gfx::surface::{CLIP_RIGHT_RVA, clip, surface};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::blit_bar", rva(0x46B574), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

    /// Pixels from x to where `percent` of the width ends: an unsigned division by 100.
    pub fn length(&self) -> Result<i32, FatalError> {
        let product = self.width.wrapping_mul(self.percent);
        if product == 0 {
            return Err(BAR_HAS_NO_LENGTH);
        }
        Ok((product as u32 / 100) as i32)
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

// ---- hook ----

/// A `Blit` that lists the calls and makes the blitter's own checks, without drawing.
pub fn checked_list(
    calls: &mut Vec<BlitCall>,
) -> impl FnMut(BlitCall) -> Result<(), FatalError> + '_ {
    |call| {
        unsafe { glyph_record(&surface(), call.table as *const u8, call.glyph) }?;
        calls.push(call);
        Ok(())
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();

    if !shadow() {
        // A fatal error: the original, run from the start, meets the same one.
        if let Err(e) = unsafe { blit_bar(&mut on_surface) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }

    let mut mine = Vec::new();
    if let Err(e) = unsafe { blit_bar(&mut checked_list(&mut mine)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }
    let (bar, clip_before) = (Bar::read(), clip().right);
    let end_r: i32 = unsafe { game_read(FILL_END_RVA) };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let end_o: i32 = unsafe { game_read(FILL_END_RVA) };

    let ok = o.preserved_except(r, 0)
        && (bar.percent == 0 || end_o == end_r)
        && clip().right == clip_before
        && (!recording_works() || calls == mine);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "{bar:?}: original calls={calls:?} end={end_o}; rust calls={mine:?} end={end_r}"
            )
        });
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    // A fatal error: the original, run from the start, meets the same one.
    if let Err(e) = unsafe { blit_bar(&mut on_surface) } {
        return Err(e.into());
    }
    Ok(())
}

// ---- self-test: the blitter's hook swallows the calls, so nothing is drawn ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let saved = (Bar::read(), unsafe { game_read::<i32>(FILL_END_RVA) });
    let clip_before = clip().right;

    // Never a non-zero percent with a product of 0: that is the fatal error.
    for (width, percent) in [
        (120, 0),
        (120, 1),
        (120, 50),
        (120, 99),
        (120, 100),
        (120, 150),
        (7, 33),
        (1, 1),
        (120, -10),
        (-120, 25),
    ] {
        let bar = Bar {
            x: 200,
            y: 150,
            width,
            fill_glyph: 0x40,
            end_glyph: 0x41,
            percent,
        };
        bar.write();
        unsafe { game_write(FILL_END_RVA, 0x5EED_0001i32) };
        let before = Regs::SENTINEL;
        let mut ro = before;
        let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
        let end_o: i32 = unsafe { game_read(FILL_END_RVA) };
        let clip_o = clip().right;

        bar.write();
        unsafe { game_write(FILL_END_RVA, 0x5EED_0001i32) };
        let mut calls_r = Vec::new();
        let mine = unsafe {
            blit_bar(&mut |call| {
                calls_r.push(call);
                Ok(())
            })
        };

        let end_r: i32 = unsafe { game_read(FILL_END_RVA) };
        let preserved = ro.preserved_except(&before, 0);
        let ok = mine.is_ok()
            && calls_o == calls_r
            && end_o == end_r
            && clip_o == clip_before
            && clip().right == clip_before
            && Bar::read() == bar
            && preserved;
        t.check(ok, || {
            format!(
                "width={width} percent={percent}: original calls={calls_o:?} end={end_o} clip_right={clip_o} preserved={preserved}; rust {mine:?} calls={calls_r:?} end={end_r}"
            )
        });
    }

    saved.0.write();
    unsafe { game_write(FILL_END_RVA, saved.1) };
    t.finish()
}
