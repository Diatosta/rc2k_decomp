//! `text::draw_string_by_id` (0x40234A): builds the text of a string id in the shared text
//! buffer, NUL-terminates it and draws it with `draw_text`.
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y
//! Out: as `draw_text` leaves it: carry, eax = 0, ebx = position in the text buffer, ecx = x,
//!      edx = y; esi, edi and ebp preserved
//! A text longer than the buffer is refused: the original writes past it.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::blit_glyph::{record_calls, recording_works};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::string_table::{LOW_IDS, blob, entry_count, ptr_low};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::strings::utils::bytes_str;
use crate::text::draw_text::{Canvas, GameCanvas, Outcome, RecordingCanvas, draw_text};
use crate::text::font::Font;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::draw_string_by_id", rva(0x40234A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The text of string `id` with its NUL, as the original leaves it in the text buffer.
pub unsafe fn string_text(id: u32) -> Result<Vec<u8>, BadInput> {
    let mut out = Vec::new();
    unsafe { append_string_by_id(id, &mut |p, n| out.extend_from_slice(from_raw_parts(p, n))) };
    out.push(0);
    if out.len() > TEXT_LEN {
        return Err(BadInput(
            "text longer than the text buffer: the original writes past it",
        ));
    }
    Ok(out)
}

/// `buffer` is where the text is built: the game's text buffer, or a private one of at least
/// `TEXT_LEN` bytes when the game's must not be touched. `Outcome::text` points into it.
pub unsafe fn draw_string_by_id(
    id: u32,
    font: &mut Font,
    x: i32,
    y: i32,
    buffer: *mut u8,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    unsafe {
        let bytes = string_text(id)?;
        copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        Ok(draw_text(font, buffer, x, y, canvas)?)
    }
}

/// The same with a font object in game memory and the game's text buffer, for Rust callers.
pub unsafe fn draw_string_by_id_at(
    id: u32,
    font: *mut Font,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<Outcome, Refused> {
    unsafe {
        let before = font.read_unaligned();
        let mut copy = before;
        let out = draw_string_by_id(id, &mut copy, x, y, text(), canvas)?;
        if copy != before && canvas.keeps_font_changes() {
            font.write_unaligned(copy);
        }
        Ok(out)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (id, font, x, y) = (r.eax, r.ebx as *mut Font, r.ecx as i32, r.edx as i32);

    if !shadow() {
        let mut copy = unsafe { font.read_unaligned() };
        match unsafe { draw_string_by_id(id, &mut copy, x, y, text(), &mut GameCanvas) } {
            Ok(out) => {
                if copy != unsafe { font.read_unaligned() } {
                    unsafe { font.write_unaligned(copy) };
                }
                (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
                r.set_carry(out.too_wide);
                r.set_zero(!out.too_wide);
            }
            // Refused text, or a fatal error in the blitter. Running the original from the
            // start is safe: it rebuilds the same text and redraws the same glyphs.
            Err(_) => unsafe { call_original(r) },
        }
        return;
    }

    // Shadow: Rust works in a private buffer and draws nothing.
    let mut private = vec![0u8; TEXT_LEN];
    let mut font_r = unsafe { font.read_unaligned() };
    let mut canvas = RecordingCanvas::new(true);
    let Ok(out) =
        (unsafe { draw_string_by_id(id, &mut font_r, x, y, private.as_mut_ptr(), &mut canvas) })
    else {
        unsafe { call_original(r) };
        return;
    };
    let len = private.iter().position(|&b| b == 0).map_or(0, |n| n + 1);

    let mut o = *r;
    let calls = record_calls(false, || unsafe { call_original(&mut o) });

    let ok = o.eax == 0
        && o.ebx.wrapping_sub(text() as u32)
            == (out.text as u32).wrapping_sub(private.as_ptr() as u32)
        && o.ecx == out.x as u32
        && o.edx == out.y as u32
        && o.carry() == out.too_wide
        && unsafe { from_raw_parts(text(), len) } == &private[..len]
        && unsafe { font.read_unaligned() } == font_r
        && (!recording_works() || calls == canvas.calls);
    if !ok {
        HOOK.mismatch(|| {
            format!(
                "id={id:#x} x={x} y={y}: original eax={:#x} ebx=+{} ecx={} edx={} carry={} calls={} text={}; rust {out:?} calls={} text={}",
                o.eax,
                o.ebx.wrapping_sub(text() as u32),
                o.ecx as i32,
                o.edx as i32,
                o.carry(),
                calls.len(),
                bytes_str(unsafe { from_raw_parts(text(), len) }),
                canvas.calls.len(),
                bytes_str(&private[..len])
            )
        });
    }
    (r.eax, r.ebx, r.ecx, r.edx, r.eflags) = (o.eax, o.ebx, o.ecx, o.edx, o.eflags);
}

// ---- self-test ----
//
// Real strings from the game's table, drawn with a private font whose glyph table covers
// every glyph index a byte or an extended code can produce. The blitter's hook swallows the
// calls, so nothing is drawn. The game's text buffer is restored after each case.

/// `count` glyph records without rows: only their x offset and width matter here.
fn header_only_table(count: usize) -> Vec<u8> {
    let mut table = Vec::with_capacity(count * 8);
    for i in 0..count {
        table.extend_from_slice(&((count * 4 + i * 4) as u32).to_le_bytes());
    }
    for i in 0..count {
        // x offset, y offset, width (0 for every fifth glyph), no rows
        table.extend([
            (i % 7) as u8 + 1,
            0,
            if i % 5 == 0 { 0 } else { (i % 11) as u8 + 2 },
            0,
        ]);
    }
    table
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !recording_works() {
        return t.skip("the gfx::blit_glyph hook is off, so the original would really draw");
    }
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let entries = entry_count() as u32;
    // Only ids that take the string-table branch.
    let from_table = |id: u32| {
        id < entries
            && id & 0x1000 == 0
            && !(LOW_IDS.contains(&id) && unsafe { !ptr_low(id).is_null() })
    };
    // 224 normal glyphs (0x20..=0xFF) plus every extended index: ((0x0F << 8 | 0xFF) >> 1) + 224.
    let table = header_only_table(2400);
    let mut private = vec![0u8; TEXT_LEN];

    for id in (0..entries).step_by(53).filter(|&id| from_table(id)) {
        for (mode, box_width) in [(0u8, 0u32), (1, 300), (3, 300)] {
            let font = Font {
                first_char: 0x20,
                last_char: 0xFF,
                mode,
                flags: 0,
                spacing: 1,
                line_height: 12,
                y_offset: 0,
                extended_spacing: 6,
                palette: 0x1234_5678,
                glyph_table: table.as_ptr() as u32,
                box_width,
            };
            let (x, y) = (40, 30);
            let mut font_r = font;
            let mut canvas = RecordingCanvas::new(false);
            let Ok(out) = (unsafe {
                draw_string_by_id(id, &mut font_r, x, y, private.as_mut_ptr(), &mut canvas)
            }) else {
                continue;
            };
            let len = private.iter().position(|&b| b == 0).map_or(0, |n| n + 1);

            let saved = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
            let mut font_o = font;
            let before = Regs {
                eax: id,
                ebx: &mut font_o as *mut Font as u32,
                ecx: x as u32,
                edx: y as u32,
                ..Regs::default()
            };
            let mut ro = before;
            let calls_o = record_calls(true, || unsafe { call_original(&mut ro) });
            let text_o = unsafe { from_raw_parts(text(), len) }.to_vec();
            unsafe { copy_nonoverlapping(saved.as_ptr(), text(), TEXT_LEN) };

            let preserved = ro.preserved_except(&before, EAX | EBX | ECX | EDX);
            let ok = ro.eax == 0
                && ro.ebx.wrapping_sub(text() as u32)
                    == (out.text as u32).wrapping_sub(private.as_ptr() as u32)
                && ro.ecx == out.x as u32
                && ro.edx == out.y as u32
                && ro.carry() == out.too_wide
                && text_o == private[..len]
                && font_o == font_r
                && calls_o == canvas.calls
                && preserved;
            t.check(ok, || {
                format!(
                    "id={id:#x} mode={mode}: original eax={:#x} ebx=+{} ecx={} edx={} carry={} calls={} text={} preserved={preserved}; rust {out:?} calls={} text={}",
                    ro.eax,
                    ro.ebx.wrapping_sub(text() as u32),
                    ro.ecx as i32,
                    ro.edx as i32,
                    ro.carry(),
                    calls_o.len(),
                    bytes_str(&text_o),
                    canvas.calls.len(),
                    bytes_str(&private[..len])
                )
            });
        }
    }
    t.finish()
}
