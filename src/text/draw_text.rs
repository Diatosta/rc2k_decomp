//! `text::draw_text` (0x4023D5): draws NUL-terminated text with a bitmap font, line by line.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: carry set if a line was wider than the box (`FLAG_FAIL_IF_WIDER`), else clear;
//!      eax = 0, ebx = text after the NUL (the start of the failed line on carry),
//!      ecx = x as given, edx = y of the last line; esi, edi and ebp preserved
//!
//! Bytes from `first_char` to `last_char` are glyphs, bytes above are ignored, bytes below are
//! control codes:
//!   0x00        end of text
//!   0x0A        new line: back to x, down by the line height
//!   0x0E n      x += n (signed byte)
//!   0x0F n      y += n (signed byte); ignored while measuring
//!   0x10.. n    extended glyph (((code - 0x10) << 8 | n) >> 1) + number of normal glyphs
//!   other       x -= code
//! Except in `MODE_AT_X` each line is measured before it is drawn. In `MODE_MEASURE` nothing
//! is drawn and the font object is updated instead.

use crate::bridge::{Raw, read_through};
use crate::errors::FatalError;
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{
    BlitCall, blit_glyph_on_surface, glyph_record, record_calls, recording_works,
};
use crate::gfx::surface::surface;
use crate::text::font::{FLAG_FAIL_IF_WIDER, FLAG_KERNING, Font};
use crate::text::glyph_kerning::{State, build_table, state_ptr, test_glyphs};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::draw_text", rva(0x4023D5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// What drawing text needs from outside: the kerning row table and the blitter.
pub trait Canvas {
    /// False for a canvas that only predicts: font objects in game memory are then left alone.
    fn keeps_font_changes(&self) -> bool;
    /// The kerning row table the canvas keeps.
    fn kerning(&mut self) -> &mut State;
    unsafe fn blit_glyph(&mut self, call: BlitCall) -> Result<(), FatalError>;
}

/// The game's own kerning table and surface.
pub struct GameCanvas;

impl Canvas for GameCanvas {
    fn keeps_font_changes(&self) -> bool {
        true
    }
    fn kerning(&mut self) -> &mut State {
        unsafe { &mut *state_ptr() }
    }
    unsafe fn blit_glyph(&mut self, c: BlitCall) -> Result<(), FatalError> {
        unsafe {
            blit_glyph_on_surface(
                c.glyph,
                c.table as *const u8,
                c.x,
                c.y,
                c.palette as *const u32,
            )
        }
        .map(|_| ())
    }
}

/// Draws nothing: keeps a private kerning table and lists the blitter calls.
pub struct RecordingCanvas {
    pub state: State,
    pub calls: Vec<BlitCall>,
    /// Make the blitter's three fatal-error checks, as a real call would.
    pub check_glyphs: bool,
}

impl RecordingCanvas {
    /// Starts from the game's current kerning table.
    pub fn new(check_glyphs: bool) -> Self {
        Self {
            state: unsafe { *state_ptr() },
            calls: Vec::new(),
            check_glyphs,
        }
    }
}

impl Canvas for RecordingCanvas {
    fn keeps_font_changes(&self) -> bool {
        false
    }
    fn kerning(&mut self) -> &mut State {
        &mut self.state
    }
    unsafe fn blit_glyph(&mut self, c: BlitCall) -> Result<(), FatalError> {
        if self.check_glyphs {
            unsafe { glyph_record(&surface(), c.table as *const u8, c.glyph) }?;
        }
        self.calls.push(c);
        Ok(())
    }
}

/// How `draw_text` ended. The original also returns eax = 0.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Outcome {
    /// A line was wider than the box and the font has `FLAG_FAIL_IF_WIDER`: the carry flag.
    pub too_wide: bool,
    /// After the NUL, or the start of the line that was too wide: ebx.
    pub text: *const u8,
    /// The x that was passed in: ecx.
    pub x: i32,
    /// The y of the last line, font offset included: edx.
    pub y: i32,
}

/// A canvas of this crate as the core's.
pub struct Adapt<'a>(&'a mut dyn Canvas);

pub enum AdaptError {
    Fatal(FatalError),
    Invalid(rc2k_core::Invalid),
}

impl From<rc2k_core::Invalid> for AdaptError {
    fn from(e: rc2k_core::Invalid) -> Self {
        AdaptError::Invalid(e)
    }
}

impl rc2k_core::text::Canvas for Adapt<'_> {
    type Error = AdaptError;
    fn kerning(&mut self) -> &mut State {
        self.0.kerning()
    }
    fn blit(&mut self, font: &Font, glyph: u32, x: i32, y: i32) -> Result<(), AdaptError> {
        let call = BlitCall::new(glyph, x, y, font.palette, font.glyph_table);
        unsafe { self.0.blit_glyph(call) }.map_err(AdaptError::Fatal)
    }
}

/// A core text function run on a text and a font's glyph table in game memory, with a
/// canvas of this crate.
pub unsafe fn run_core<T>(
    font: &Font,
    text: *const u8,
    canvas: &mut dyn Canvas,
    f: impl FnOnce(&Raw, &Raw, &mut Adapt) -> Result<T, AdaptError>,
) -> Result<T, FatalError> {
    let (words, glyphs) = unsafe { (Raw::new(text), Raw::new(font.glyph_table as *const u8)) };
    match f(&words, &glyphs, &mut Adapt(canvas)) {
        Ok(value) => Ok(value),
        Err(AdaptError::Fatal(e)) => Err(e),
        Err(AdaptError::Invalid(e)) => Ok(read_through(Err(e))),
    }
}

/// The core's outcome with the text as a pointer.
pub fn outcome(text: *const u8, out: rc2k_core::text::Outcome) -> Outcome {
    Outcome {
        too_wide: out.too_wide,
        text: text.wrapping_add(out.next),
        x: out.x,
        y: out.y,
    }
}

/// `font` is changed only in `MODE_MEASURE`. An error can only come from the blitter; by then
/// earlier glyphs are drawn. Converted: `rc2k_core::text::draw`.
pub unsafe fn draw_text(
    font: &mut Font,
    text: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, FatalError> {
    let copy = *font;
    let out = unsafe {
        run_core(&copy, text, canvas, |words, glyphs, c| {
            rc2k_core::text::draw(font, words, glyphs, x, y, c)
        })
    }?;
    Ok(outcome(text, out))
}

/// `draw_text` with a font object in game memory, for Rust callers. The object is only
/// written (it only changes in `MODE_MEASURE`) if the canvas keeps font changes.
pub unsafe fn draw_text_at(
    font: *mut Font,
    text: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, FatalError> {
    unsafe {
        let before = font.read_unaligned();
        let mut copy = before;
        let out = draw_text(&mut copy, text, x, y, canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(out)
    }
}

// ---- hook ----

/// What the Rust version, run without drawing, says a call will do. The text hooks compare
/// this with what the original then really does.
pub struct Prediction {
    pub outcome: Outcome,
    pub font: Font,
    pub state: State,
    pub calls: Vec<BlitCall>,
}

impl Prediction {
    /// Runs the Rust version on a copy of the font and of the kerning table.
    pub unsafe fn new(
        font: *const Font,
        text: *const u8,
        x: i32,
        y: i32,
        check_glyphs: bool,
    ) -> Result<Self, FatalError> {
        let mut copy = unsafe { font.read_unaligned() };
        let mut canvas = RecordingCanvas::new(check_glyphs);
        let outcome = unsafe { draw_text(&mut copy, text, x, y, &mut canvas) }?;
        Ok(Self {
            outcome,
            font: copy,
            state: canvas.state,
            calls: canvas.calls,
        })
    }

    /// True if the font object, the kerning table and the blitter calls came out as predicted.
    /// The calls are only known while the blitter's hook is enabled.
    pub unsafe fn side_effects_match(&self, font: *const Font, calls: &[BlitCall]) -> bool {
        let same_memory =
            unsafe { font.read_unaligned() == self.font && *state_ptr() == self.state };
        same_memory && (!recording_works() || calls == self.calls)
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let font = r.eax as *mut Font;
    let (text, x, y) = (r.ebx as *const u8, r.ecx as i32, r.edx as i32);

    if !shadow() {
        let mut copy = unsafe { font.read_unaligned() };
        match unsafe { draw_text(&mut copy, text, x, y, &mut GameCanvas) } {
            Ok(out) => {
                if copy != unsafe { font.read_unaligned() } {
                    unsafe { font.write_unaligned(copy) };
                }
                (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
                r.set_carry(out.too_wide);
                r.set_zero(!out.too_wide);
            }
            // The blitter met a fatal error. Running the original from the start is safe:
            // it redraws the same glyphs and resets the kerning table itself.
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }

    let mine = match unsafe { Prediction::new(font, text, x, y, true) } {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });

    let out = mine.outcome;
    let regs_ok = o.eax == 0
        && o.ebx == out.text as u32
        && o.ecx == out.x as u32
        && o.edx == out.y as u32
        && o.carry() == out.too_wide;
    let effects_ok = unsafe { mine.side_effects_match(font, &calls) };
    if !regs_ok || !effects_ok {
        HOOK.mismatch(|| {
            format!(
                "font={:#x} text={:#x} x={x} y={y}: regs_ok={regs_ok} effects_ok={effects_ok} original eax={:#x} ebx={:#x} ecx={} edx={} carry={} calls={} rust {out:?} calls={}",
                r.eax,
                r.ebx,
                o.eax,
                o.ebx,
                o.ecx as i32,
                o.edx as i32,
                o.carry(),
                calls.len(),
                mine.calls.len()
            )
        });
    }
    (r.eax, r.ebx, r.ecx, r.edx, r.eflags) = (o.eax, o.ebx, o.ecx, o.edx, o.eflags);
}

// ---- self-test ----
//
// The original is run with the blitter's hook swallowing its calls, so nothing is drawn. The
// kerning table is the game's own: it is saved first and put back at the end.

/// A font over a 14-glyph test table (the kerning test's seven glyphs, twice): 'a'..'g' are
/// glyphs 0..6, and extended codes reach glyphs 7..13.
pub(crate) fn test_table() -> Vec<u8> {
    let mut glyphs = test_glyphs();
    glyphs.extend(test_glyphs());
    build_table(&glyphs)
}

pub(crate) fn test_font(table: &[u8], mode: u8, flags: u8, box_width: u32) -> Font {
    Font {
        first_char: b'a',
        last_char: b'g',
        mode,
        flags,
        spacing: 1,
        line_height: 9,
        y_offset: 2,
        extended_spacing: 6,
        palette: 0x1234_5678,
        glyph_table: table.as_ptr() as u32,
        box_width,
    }
}

/// Texts over that font: every control code, extended glyphs, empty lines, ignored bytes.
pub(crate) const TEST_TEXTS: [&[u8]; 12] = [
    b"\0",
    b"abc\0",
    b"gfedcba\0",
    b"a\ng\0",
    b"ab\n\ncd\0",
    b"ab\x0e\x05c\0",
    b"ab\x0e\xfbc\0",
    b"a\x0f\x03b\x0f\xfdc\0",
    b"a\x03b\x0dc\0",
    b"a\x10\x04b\x10\x0dc\0",
    b"azb~c\xffd\0",
    b"\x10\x00\x0e\x02a\n\x10\x0c\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let table = test_table();
    let saved_state = unsafe { *state_ptr() };

    for mode in 0..=5u8 {
        for flags in [
            0,
            FLAG_FAIL_IF_WIDER,
            FLAG_KERNING,
            FLAG_FAIL_IF_WIDER | FLAG_KERNING,
        ] {
            for box_width in [0u32, 10, 1000] {
                for text in TEST_TEXTS {
                    let (x, y) = (100, 50);
                    let mut font_o = test_font(&table, mode, flags, box_width);
                    let before = Regs {
                        eax: &mut font_o as *mut Font as u32,
                        ebx: text.as_ptr() as u32,
                        ecx: x as u32,
                        edx: y as u32,
                        ..Regs::SENTINEL
                    };
                    let mut ro = before;
                    let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
                    let state_o = unsafe { *state_ptr() };

                    let mut font_r = test_font(&table, mode, flags, box_width);
                    let mut canvas = RecordingCanvas::new(false);
                    let mine = unsafe { draw_text(&mut font_r, text.as_ptr(), x, y, &mut canvas) };

                    let preserved = ro.preserved_except(&before, EAX | EBX | ECX | EDX);
                    let ok = mine.is_ok_and(|out| {
                        ro.eax == 0
                            && ro.ebx == out.text as u32
                            && ro.ecx == out.x as u32
                            && ro.edx == out.y as u32
                            && ro.carry() == out.too_wide
                    }) && font_o == font_r
                        && calls_o == canvas.calls
                        && (flags & FLAG_KERNING == 0 || state_o == canvas.state)
                        && preserved;
                    t.check(ok, || {
                        format!(
                            "mode={mode} flags={flags} box={box_width} text={:?}: original eax={:#x} ebx=+{} ecx={} edx={} carry={} calls={:?} font={font_o:?} preserved={preserved}; rust {mine:?} calls={:?} font={font_r:?}",
                            text.escape_ascii().to_string(),
                            ro.eax,
                            ro.ebx.wrapping_sub(text.as_ptr() as u32),
                            ro.ecx as i32,
                            ro.edx as i32,
                            ro.carry(),
                            calls_o,
                            canvas.calls
                        )
                    });
                }
            }
        }
    }

    unsafe { *state_ptr() = saved_state };
    t.finish()
}
