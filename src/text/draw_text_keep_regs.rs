//! `text::draw_text_keep_regs` (0x402303): `draw_text` between `pusha` and `popa`.
//!
//! In:  eax = font, ebx = text, ecx = x, edx = y
//! Out: carry as `draw_text` leaves it; every register preserved

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::text::draw_text::{
    Canvas, GameCanvas, Prediction, RecordingCanvas, TEST_TEXTS, draw_text, test_font, test_table,
};
use crate::text::font::{FLAG_FAIL_IF_WIDER, FLAG_KERNING, Font};
use crate::text::glyph_kerning::state_ptr;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

// The first 5 bytes end inside a `call`, so the jump is written by hand and the original runs
// from a private copy (its call at +1 relocated).
pub static HOOK: RawHook =
    RawHook::new("text::draw_text_keep_regs", rva(0x402303), thunk, &ORIGINAL)
        .private_copy(0x08)
        .relocate_calls(&[0x01])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// Returns whether a line was too wide: the carry flag.
pub unsafe fn draw_text_keep_regs(
    font: &mut Font,
    text: *const u8,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<bool, FatalError> {
    Ok(unsafe { draw_text(font, text, x, y, canvas) }?.too_wide)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let font = r.eax as *mut Font;
    let (text, x, y) = (r.ebx as *const u8, r.ecx as i32, r.edx as i32);

    if !shadow() {
        let mut copy = unsafe { font.read_unaligned() };
        match unsafe { draw_text_keep_regs(&mut copy, text, x, y, &mut GameCanvas) } {
            Ok(too_wide) => {
                if copy != unsafe { font.read_unaligned() } {
                    unsafe { font.write_unaligned(copy) };
                }
                r.set_carry(too_wide);
                r.set_zero(!too_wide);
            }
            // A fatal error in the blitter: see draw_text.
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

    let regs_ok = o.preserved_except(r, 0) && o.carry() == mine.outcome.too_wide;
    let effects_ok = unsafe { mine.side_effects_match(font, &calls) };
    if !regs_ok || !effects_ok {
        HOOK.mismatch(|| {
            format!(
                "font={:#x} text={:#x} x={x} y={y}: regs_ok={regs_ok} effects_ok={effects_ok} original carry={} calls={} rust carry={} calls={}",
                r.eax,
                r.ebx,
                o.carry(),
                calls.len(),
                mine.outcome.too_wide,
                mine.calls.len()
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

    for mode in [0u8, 1, 3, 4] {
        for flags in [0, FLAG_FAIL_IF_WIDER, FLAG_KERNING] {
            for text in TEST_TEXTS {
                let (x, y) = (100, 50);
                let mut font_o = test_font(&table, mode, flags, 10);
                let before = Regs {
                    eax: &mut font_o as *mut Font as u32,
                    ebx: text.as_ptr() as u32,
                    ecx: x as u32,
                    edx: y as u32,
                    ..Regs::SENTINEL
                };
                let mut ro = before;
                let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });

                let mut font_r = test_font(&table, mode, flags, 10);
                let mut canvas = RecordingCanvas::new(false);
                let mine =
                    unsafe { draw_text_keep_regs(&mut font_r, text.as_ptr(), x, y, &mut canvas) };

                let preserved = ro.preserved_except(&before, 0);
                let ok = mine == Ok(ro.carry())
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
