//! `text::measure_text` (0x40230B): runs `draw_text` with the font's two result fields zeroed
//! first, and returns them.
//!
//! In:  eax = font, ebx = text, ecx = x
//! Out: carry and eax = 0 as `draw_text` leaves them; ebx = the font's dword at +0x14,
//!      ecx = the one at +0x1C, edx = `draw_text`'s y; esi, edi, ebp preserved
//! With a font in measure mode nothing is drawn, +0x14 counts the lines and +0x1C is the
//! width of the widest one, which is what its twelve callers want. The y passed on is the
//! font's dword at +8. With any other font it would draw, with a null palette: reproduced as
//! it is.

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{BlitCall, record_calls, recording_works};
use crate::text::draw_text::{Canvas, GameCanvas, Outcome, RecordingCanvas, draw_text};
use crate::text::font::Font;
use crate::text::glyph_kerning::state_ptr;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::measure_text", rva(0x40230B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measured {
    pub outcome: Outcome,
    /// The font's dword at +0x14 afterwards: the number of lines, in measure mode.
    pub lines: u32,
    /// The font's dword at +0x1C afterwards: the widest line, in measure mode.
    pub width: u32,
}

/// `font` is changed as the original changes the font object.
pub unsafe fn measure_text(
    font: &mut Font,
    text: *const u8,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, FatalError> {
    font.box_width = 0;
    font.palette = 0;
    let outcome = unsafe { draw_text(font, text, x, font.line_height, canvas) }?;
    Ok(Measured {
        outcome,
        lines: font.palette,
        width: font.box_width,
    })
}

/// For Rust callers, with a font object in game memory: the object is only written if the
/// canvas keeps font changes.
pub unsafe fn measure_text_at(
    font: *mut Font,
    text: *const u8,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, FatalError> {
    unsafe {
        let mut copy = font.read_unaligned();
        let measured = measure_text(&mut copy, text, x, canvas)?;
        if canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(measured)
    }
}

// ---- hook ----

pub fn regs_match(m: &Measured, o: &Regs, before: &Regs) -> bool {
    o.eax == 0
        && o.ebx == m.lines
        && o.ecx == m.width
        && o.edx == m.outcome.y as u32
        && o.carry() == m.outcome.too_wide
        && (o.esi, o.edi, o.ebp) == (before.esi, before.edi, before.ebp)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let font = r.eax as *mut Font;
    let (text, x) = (r.ebx as *const u8, r.ecx as i32);

    if !shadow() {
        match unsafe { measure_text_at(font, text, x, &mut GameCanvas) } {
            Ok(m) => {
                (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
                r.set_carry(m.outcome.too_wide);
                r.set_zero(!m.outcome.too_wide);
            }
            // The blitter met a fatal error: the original, run from the start, meets it too.
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }

    let mut predicted = unsafe { font.read_unaligned() };
    let mut canvas = RecordingCanvas::new(true);
    let mine = match unsafe { measure_text(&mut predicted, text, x, &mut canvas) } {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let same_font = unsafe { font.read_unaligned() } == predicted;
    let same_calls = !recording_works() || calls == canvas.calls;
    let same_kerning = unsafe { *state_ptr() } == canvas.state;
    if !regs_match(&mine, &o, r) || !same_font || !same_calls || !same_kerning {
        HOOK.mismatch(|| {
            format!(
                "font={:#x} text={:#x} x={x}: same_font={same_font} same_calls={same_calls} same_kerning={same_kerning} original ebx={} ecx={} edx={} carry={}; rust {mine:?}",
                r.eax,
                r.ebx,
                o.ebx,
                o.ecx,
                o.edx as i32,
                o.carry()
            )
        });
    }
    let esp = r.esp;
    *r = o;
    r.esp = esp;
}

// ---- self-test: the game's measuring font, if it is loaded ----

const MEASURING_FONT: usize = 0x51E824; // dword_51E824: the font the name editors measure with

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original could really draw");
    }
    let font: *mut Font = crate::screens::font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    if saved_font.glyph_table == 0 {
        return t.skip("the measuring font is not loaded");
    }
    let saved_kerning = unsafe { *state_ptr() };

    let texts: [&[u8]; 6] = [
        b"\0",
        b"a\0",
        b"claudethisone\0",
        b"WWWWWWWWWWWWWWWWWWWWWWWWWWWWWW\0",
        b"two words\0",
        b"MiXeD 123 _\0",
    ];
    for text in texts {
        for x in [0i32, 7] {
            unsafe { font.write_unaligned(saved_font) };
            let before = Regs {
                eax: font as u32,
                ebx: text.as_ptr() as u32,
                ecx: x as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            let calls_o: Vec<BlitCall> = record_calls(true, || unsafe { call_original(&mut ro) });
            let font_o = unsafe { font.read_unaligned() };
            let kerning_o = unsafe { *state_ptr() };

            let mut font_r = saved_font;
            let mut canvas = RecordingCanvas::new(false);
            let mine = unsafe { measure_text(&mut font_r, text.as_ptr(), x, &mut canvas) };

            let ok = mine.is_ok_and(|m| regs_match(&m, &ro, &before))
                && font_o == font_r
                && calls_o == canvas.calls
                && kerning_o == canvas.state;
            t.check(ok, || {
                format!(
                    "text={:?} x={x}: original eax={:#x} ebx={} ecx={} edx={} carry={} calls={}; rust {mine:?} calls={} same_font={}",
                    String::from_utf8_lossy(&text[..text.len() - 1]),
                    ro.eax,
                    ro.ebx,
                    ro.ecx,
                    ro.edx as i32,
                    ro.carry(),
                    calls_o.len(),
                    canvas.calls.len(),
                    font_o == font_r
                )
            });
        }
    }
    unsafe {
        font.write_unaligned(saved_font);
        *state_ptr() = saved_kerning;
    }
    t.finish()
}
