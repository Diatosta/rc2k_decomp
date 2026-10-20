//! `text::draw_clock_fixed_pitch` (0x465F42): as `text::draw_time_fixed_pitch` for a time
//! of day: the first five bytes of a text ("09:05") one at a time in fixed columns, at x -
//! 100 plus 0, 13, 23, 33 and 46.
//!
//! In:  eax = font, ebx = text, ecx = x of the right edge, edx = y
//! Out: the flags of the original's last `add ecx, 0Ah` (after the fifth character):
//!      carry when x - 54 + 10 passes 2^32, zero when it is 0; every register preserved
//! Each byte is copied to text + 0x40 with a NUL after it and drawn from there. Three
//! callers (the itinerary pages).

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas, test_font, test_table};
use crate::text::draw_text_keep_regs::draw_text_keep_regs;
use crate::text::font::{FLAG_FAIL_IF_WIDER, FLAG_KERNING, Font};
use crate::text::glyph_kerning::state_ptr;
use rc2k_core::text::{CLOCK_STEPS, columns};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "text::draw_clock_fixed_pitch",
    rva(0x465F42),
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

/// Where the one-character text is built, relative to the text.
pub const CELL_OFFSET: usize = crate::text::draw_time_fixed_pitch::CELL_OFFSET;

/// Returns the flags the original leaves: (carry, zero) of its last `add` (draw_text's
/// own carry is lost there).
pub unsafe fn draw_clock_fixed_pitch(
    font: &mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), FatalError> {
    let at = columns(x, &CLOCK_STEPS);
    unsafe {
        let cell = text.add(CELL_OFFSET);
        // Converted: `rc2k_core::text::{columns, CLOCK_STEPS}`.
        for (i, &x) in at.iter().enumerate() {
            *cell = *text.add(i);
            *cell.add(1) = 0;
            draw_text_keep_regs(font, cell, x, y, canvas)?;
        }
    }
    // The flags of the original's last `add`, after the fifth character.
    let (sum, carry) = (at[4] as u32).overflowing_add(CLOCK_STEPS[4] as u32);
    Ok((carry, sum == 0))
}

/// The same with a font object in game memory, for Rust callers.
pub unsafe fn draw_clock_fixed_pitch_at(
    font: *mut Font,
    text: *mut u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<(bool, bool), FatalError> {
    unsafe {
        let before = font.read_unaligned();
        let mut copy = before;
        let too_wide = draw_clock_fixed_pitch(&mut copy, text, x, y, canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(too_wide)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let font = r.eax as *mut Font;
    let (text, x, y) = (r.ebx as *mut u8, r.ecx as i32, r.edx as i32);

    if !shadow() {
        match unsafe { draw_clock_fixed_pitch_at(font, text, x, y, &mut GameCanvas) } {
            Ok((carry, zero)) => {
                r.set_carry(carry);
                r.set_zero(zero);
            }
            // A fatal error in the blitter. The original, run from the start, redraws the
            // same characters.
            Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
        }
        return;
    }

    // Shadow: Rust lists its blitter calls and builds the same cell the original will.
    let mut font_r = unsafe { font.read_unaligned() };
    let mut canvas = RecordingCanvas::new(true);
    let too_wide = match unsafe { draw_clock_fixed_pitch(&mut font_r, text, x, y, &mut canvas) } {
        Ok(value) => value,
        Err(e) => {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
            return;
        }
    };
    let cell_r = unsafe { [*text.add(CELL_OFFSET), *text.add(CELL_OFFSET + 1)] };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let cell_o = unsafe { [*text.add(CELL_OFFSET), *text.add(CELL_OFFSET + 1)] };

    let ok = o.preserved_except(r, 0)
        && o.carry() == too_wide.0
        && cell_o == cell_r
        && unsafe { font.read_unaligned() == font_r && *state_ptr() == canvas.state }
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "text={:#x} x={x} y={y}: original carry={} calls={} rust carry={} calls={}",
                r.ebx,
                o.carry(),
                calls.len(),
                too_wide.0,
                canvas.calls.len()
            )
        });
    }
    r.eflags = o.eflags;
}

// ---- self-test: the draw_text fixtures, with the blitter's hook swallowing the calls ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    let table = test_table();
    let saved_state = unsafe { *state_ptr() };
    // Eight bytes each; a NUL or an ignored byte in the middle is still stepped over.
    let texts: [&[u8; 8]; 5] = [
        b"abcdefga",
        b"gfedcbaa",
        b"ab\0defga",
        b"\0\0\0\0\0\0\0\0",
        b"az~b\x03c\xffd",
    ];

    for mode in [0u8, 1, 3, 4] {
        for flags in [0, FLAG_FAIL_IF_WIDER, FLAG_KERNING] {
            for text in texts {
                let (x, y) = (300, 50);
                let mut buf_o = [0xAAu8; 0x60];
                buf_o[..8].copy_from_slice(text);
                let mut font_o = test_font(&table, mode, flags, 6);
                let before = Regs {
                    eax: &mut font_o as *mut Font as u32,
                    ebx: buf_o.as_mut_ptr() as u32,
                    ecx: x as u32,
                    edx: y as u32,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });

                let mut buf_r = [0xAAu8; 0x60];
                buf_r[..8].copy_from_slice(text);
                let mut font_r = test_font(&table, mode, flags, 6);
                let mut canvas = RecordingCanvas::new(false);
                let mine = unsafe {
                    draw_clock_fixed_pitch(&mut font_r, buf_r.as_mut_ptr(), x, y, &mut canvas)
                };

                let preserved = ro.preserved_except(&before, 0);
                let ok = mine == Ok((ro.carry(), ro.eflags & 0x40 != 0))
                    && buf_o == buf_r
                    && font_o == font_r
                    && calls_o == canvas.calls
                    && preserved;
                t.check(ok, || {
                    format!(
                        "mode={mode} flags={flags} text={:?}: original carry={} calls={} preserved={preserved}; rust {mine:?} calls={}",
                        text.escape_ascii().to_string(),
                        ro.carry(),
                        calls_o.len(),
                        canvas.calls.len()
                    )
                });
            }
        }
    }

    unsafe { *state_ptr() = saved_state };
    t.finish()
}
