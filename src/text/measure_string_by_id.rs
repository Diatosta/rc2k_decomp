//! `text::measure_string_by_id` (0x40232C): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and measures it with `measure_text`.
//!
//! In:  eax = string id, ebx = font (one in measure mode), ecx = x
//! Out: as `measure_text`: ebx = lines, ecx = the widest line, edx = y, eax = 0, carry;
//!      esi, edi, ebp preserved
//! 13 callers. A text longer than the buffer is refused: the original writes past it.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::string_text;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas};
use crate::text::font::Font;
use crate::text::glyph_kerning::state_ptr;
use crate::text::measure_text::{Measured, measure_text, regs_match};
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "text::measure_string_by_id",
    rva(0x40232C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes. `font` is changed as the original changes the font object.
pub unsafe fn measure_string_by_id(
    id: u32,
    font: &mut Font,
    x: i32,
    buffer: *mut u8,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    unsafe {
        let bytes = string_text(id)?;
        copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        Ok(measure_text(font, buffer, x, canvas)?)
    }
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
/// The font object is only written if the canvas keeps font changes.
pub unsafe fn measure_string_by_id_at(
    id: u32,
    font: *mut Font,
    x: i32,
    canvas: &mut dyn Canvas,
) -> Result<Measured, Refused> {
    if font.is_null() {
        return Err(BadInput("no font").into());
    }
    unsafe {
        let mut copy = font.read_unaligned();
        let measured = measure_string_by_id(id, &mut copy, x, text(), canvas)?;
        if canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(measured)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (id, font, x) = (r.eax, r.ebx as *mut Font, r.ecx as i32);

    if !shadow() {
        match unsafe { measure_string_by_id_at(id, font, x, &mut GameCanvas) } {
            Ok(m) => {
                (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
                r.set_carry(m.outcome.too_wide);
                r.set_zero(!m.outcome.too_wide);
            }
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }

    // Shadow: Rust works in a private buffer, on a copy of the font, and draws nothing.
    if font.is_null() {
        unsafe { call_original(r) };
        return;
    }
    let mut private = vec![0u8; TEXT_LEN];
    let mut predicted = unsafe { font.read_unaligned() };
    let mut canvas = RecordingCanvas::new(true);
    let mine = match unsafe {
        measure_string_by_id(id, &mut predicted, x, private.as_mut_ptr(), &mut canvas)
    } {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let len = private.iter().position(|&b| b == 0).map_or(0, |n| n + 1);

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let same_text = unsafe { from_raw_parts(text(), len) } == &private[..len];
    let same_font = unsafe { font.read_unaligned() } == predicted;
    let same_calls = !recording_works() || calls == canvas.calls;
    let same_kerning = unsafe { *state_ptr() } == canvas.state;
    if !regs_match(&mine, &o, r) || !same_text || !same_font || !same_calls || !same_kerning {
        HOOK.mismatch(|| {
            format!(
                "id={id:#x} font={:#x} x={x}: same_text={same_text} same_font={same_font} same_calls={same_calls} same_kerning={same_kerning} original ebx={} ecx={} edx={} carry={}; rust {mine:?}",
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

// ---- self-test: the game's measuring font and strings ----

pub const MEASURING_FONT: usize = 0x51E824; // dword_51E824: the font the name editors measure with
pub const TEST_STRINGS: [u32; 3] = [0x4E5, 0x4E6, 0x4E7];

/// Why a test that measures the game's strings cannot run, or None if it can.
pub fn measuring_unavailable() -> Option<&'static str> {
    if !recording_works() {
        return Some("the gfx::blit_glyph hook is off, so the original could really draw");
    }
    if blob().is_null() || !TEST_STRINGS.iter().all(|&id| is_table_id(id)) {
        return Some("the strings are not loaded");
    }
    (!crate::screens::font_is_loaded(MEASURING_FONT)).then_some("the measuring font is not loaded")
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = measuring_unavailable() {
        return t.skip(why);
    }
    let font: *mut Font = crate::screens::font_at(MEASURING_FONT);
    let saved_font = unsafe { font.read_unaligned() };
    let saved_text = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    let saved_kerning = unsafe { *state_ptr() };

    for id in TEST_STRINGS {
        for x in [0i32, 7] {
            unsafe {
                font.write_unaligned(saved_font);
                std::ptr::write_bytes(text(), 0xAA, TEXT_LEN - 1);
                *text().add(TEXT_LEN - 1) = 0;
                *state_ptr() = saved_kerning;
            }
            let before = Regs {
                eax: id,
                ebx: font as u32,
                ecx: x as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
            let font_o = unsafe { font.read_unaligned() };
            let text_o = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
            let kerning_o = unsafe { *state_ptr() };

            let mut font_r = saved_font;
            let mut text_r = vec![0xAAu8; TEXT_LEN];
            text_r[TEXT_LEN - 1] = 0;
            unsafe { *state_ptr() = saved_kerning };
            let mut canvas = RecordingCanvas::new(false);
            let mine = unsafe {
                measure_string_by_id(id, &mut font_r, x, text_r.as_mut_ptr(), &mut canvas)
            };

            let ok = mine.as_ref().is_ok_and(|m| regs_match(m, &ro, &before))
                && font_o == font_r
                && text_o == text_r
                && calls_o == canvas.calls
                && kerning_o == canvas.state;
            t.check(ok, || {
                format!(
                    "id={id:#x} x={x}: original eax={:#x} ebx={} ecx={} edx={} carry={} calls={}; rust {mine:?} calls={} same_font={} same_text={}",
                    ro.eax,
                    ro.ebx,
                    ro.ecx,
                    ro.edx as i32,
                    ro.carry(),
                    calls_o.len(),
                    canvas.calls.len(),
                    font_o == font_r,
                    text_o == text_r
                )
            });
        }
    }
    unsafe {
        font.write_unaligned(saved_font);
        copy_nonoverlapping(saved_text.as_ptr(), text(), TEXT_LEN);
        *state_ptr() = saved_kerning;
    }
    t.finish()
}
