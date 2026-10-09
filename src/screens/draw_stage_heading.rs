//! `screens::draw_stage_heading` (0x466DF3): draws one heading line for a record:
//! string 0x481, a space, "<record +0x40>/6", ": " and the name of the record's track pack
//! entry, at (747, 145) with the font at 0x51E9A4.
//!
//! In:  edi = record
//! Out: carry as `draw_text` leaves it; every register preserved
//! "Stage heading" is a reading of that line; the text of string 0x481 has not been looked at.

use crate::errors::FatalError;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::records::trakpak_entry::record_entry;
use crate::screens::{font_at, font_is_loaded, line_buffer, set_line};
use crate::strings::append_fraction::append_fraction;
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::utils::bytes_str;
use crate::text::draw_text::{Canvas, GameCanvas, RecordingCanvas, draw_text_at};
use crate::text::glyph_kerning::state_ptr;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_stage_heading",
    rva(0x466DF3),
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

// ---- game data ----

const FONT: usize = 0x51E9A4;
const HEADING_STRING: u32 = 0x481;
const OUT_OF: u32 = 6;
const X: i32 = 0x2EB;
const Y: i32 = 0x91;

unsafe fn dword(at: *const u8) -> u32 {
    unsafe { at.cast::<u32>().read_unaligned() }
}

// ---- Rust version ----

/// The line with its NUL. The error is the track pack lookup's.
pub unsafe fn stage_heading_text(record: *const u8) -> Result<Vec<u8>, FatalError> {
    unsafe {
        let mut out = Vec::new();
        push_string(&mut out, HEADING_STRING);
        out.push(b' ');
        out.extend(append_fraction(dword(record.add(0x40)), OUT_OF));
        out.extend_from_slice(b": ");
        let entry = record_entry(record)? as *const u8;
        push_string(&mut out, dword(entry));
        out.push(0);
        Ok(out)
    }
}

/// Builds the line in the game's line buffer and draws it. Returns the carry flag.
pub unsafe fn draw_stage_heading(
    record: *const u8,
    canvas: &mut dyn Canvas,
) -> Result<bool, FatalError> {
    unsafe {
        set_line(&stage_heading_text(record)?);
        Ok(draw_text_at(font_at(FONT), line_buffer(), X, Y, canvas)?.too_wide)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let record = r.edi as *const u8;

    if !shadow() {
        match unsafe { draw_stage_heading(record, &mut GameCanvas) } {
            Ok(too_wide) => {
                r.set_carry(too_wide);
                r.set_zero(!too_wide);
            }
            // A fatal error in the lookup or the blitter: the original, run from the start,
            // rebuilds the line and meets the same error.
            Err(_) => unsafe { call_original(r) },
        }
        return;
    }

    // Shadow: Rust builds the line and lists its blitter calls; the original then builds the
    // line again and really draws.
    let mut canvas = RecordingCanvas::new(true);
    let Ok(too_wide) = (unsafe { draw_stage_heading(record, &mut canvas) }) else {
        unsafe { call_original(r) };
        return;
    };
    let line_r = unsafe { line_bytes() };

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });
    let line_o = unsafe { line_bytes() };

    let ok = o.preserved_except(r, 0)
        && o.carry() == too_wide
        && line_o == line_r
        && unsafe { *state_ptr() } == canvas.state
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "record={:#x}: original carry={} calls={} line={}; rust carry={too_wide} calls={} line={}",
                r.edi,
                o.carry(),
                calls.len(),
                bytes_str(&line_o),
                canvas.calls.len(),
                bytes_str(&line_r)
            )
        });
    }
    r.eflags = o.eflags;
}

/// The text in the line buffer, NUL included (at most 0x200 bytes).
unsafe fn line_bytes() -> Vec<u8> {
    let all = unsafe { from_raw_parts(line_buffer(), 0x200) };
    let len = all
        .iter()
        .position(|&b| b == 0)
        .map_or(all.len(), |n| n + 1);
    all[..len].to_vec()
}

// ---- self-test ----
//
// Uses the game's own font, strings and loaded track pack, so it only runs when those are
// there. The blitter's hook swallows the calls; the line buffer and the kerning table are
// restored.

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    if blob().is_null() || !font_is_loaded(FONT) || !is_table_id(HEADING_STRING) {
        return t.skip("the string table or the font is not loaded");
    }
    // A record of the loaded pack: read the pack's id the same way the lookup does.
    let loaded = unsafe { dword(crate::framework::game_ptr::<u8>(rva(0x734BC6 + 4))) };
    let saved_line = unsafe { from_raw_parts(line_buffer(), 0x200) }.to_vec();
    let saved_state = unsafe { *state_ptr() };

    let mut ran = 0;
    for index in 0..4u32 {
        for stage in [1u32, 6, 12] {
            let mut record = [0u8; 0x44];
            record[4..8].copy_from_slice(&index.to_le_bytes());
            record[8..12].copy_from_slice(&loaded.to_le_bytes());
            record[0x40..0x44].copy_from_slice(&stage.to_le_bytes());

            // The entry's first dword is used as a string id: only go on if it is one.
            let Ok(entry) = (unsafe { record_entry(record.as_ptr()) }) else {
                continue;
            };
            if !is_table_id(unsafe { dword(entry as *const u8) }) {
                continue;
            }

            ran += 1;
            let before = Regs {
                edi: record.as_ptr() as u32,
                ..Regs::default()
            };
            let mut ro = before;
            let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
            let line_o = unsafe { line_bytes() };
            let state_o = unsafe { *state_ptr() };

            unsafe { set_line(&saved_line) };
            let mut canvas = RecordingCanvas::new(false);
            let mine = unsafe { draw_stage_heading(record.as_ptr(), &mut canvas) };
            let line_r = unsafe { line_bytes() };
            unsafe { set_line(&saved_line) };

            let preserved = ro.preserved_except(&before, 0);
            let ok = mine == Ok(ro.carry())
                && line_o == line_r
                && calls_o == canvas.calls
                && state_o == canvas.state
                && preserved;
            t.check(ok, || {
                format!(
                    "index={index} stage={stage}: original carry={} calls={} line={} preserved={preserved}; rust {mine:?} calls={} line={}",
                    ro.carry(),
                    calls_o.len(),
                    bytes_str(&line_o),
                    canvas.calls.len(),
                    bytes_str(&line_r)
                )
            });
        }
    }

    unsafe { *state_ptr() = saved_state };
    if ran == 0 {
        return t.skip("no track pack entry with a usable name is loaded");
    }
    t.finish()
}
