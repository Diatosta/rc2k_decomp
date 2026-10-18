//! A rally stage's information page (`sub_489AC8` -> `sub_4744A9`; stream A, part F1,
//! 2026-10-18): the stage record (`off_73555A[dword_7587EC]` + 0xA4 x `dword_7587F0`) and
//! its track-pack entry: start and finish times, the time allowed, the length, the surface's
//! shares, the weather, the description, the stage's number, and the best and fastest.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::{font_at, line_buffer};
use crate::text::draw_text::{Canvas, draw_text_at};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The page's stage record (none: 0).
pub fn stage_record() -> u32 {
    unsafe {
        let base =
            ((game_addr(rva(0x73555A)) as u32 + 4 * k(0x7587EC)) as *const u32).read_unaligned();
        if base == 0 {
            return 0;
        }
        base.wrapping_add(k(0x7587F0).wrapping_mul(0xA4))
    }
}

/// `bytes` and its NUL into the line buffer, drawn.
unsafe fn line(
    bytes: Vec<u8>,
    font: usize,
    x: i32,
    y: i32,
    canvas: &mut dyn Canvas,
) -> Result<u32, Refused> {
    unsafe {
        let mut bytes = bytes;
        bytes.push(0);
        if bytes.len() > 0x100 {
            return Err(BadInput("a line past the line buffer").into());
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len());
        Ok(draw_text_at(font_at(font), line_buffer(), x, y, canvas)?.text as u32)
    }
}

fn flag_glyph(flag: u32) -> u32 {
    (if (0x18..=0x19).contains(&flag) {
        flag ^ 1
    } else {
        flag
    })
    .wrapping_add(0x44)
}

/// `screens::draw_stage_best_and_fastest` (0x4744A9): the "best" and "fastest" lines
/// (strings 0x34B / 0x34C, ": ", string 0, at x 0x188, y 0x1FC / 0x213): the best of the
/// times at edi (its car, name, flag and time) and the stage record's fastest (+0x74 car, +0x54
/// name, +0x78 flag, +0x30 speed). The fastest's car sprite gets `dword_7587EC` as its palette
/// (ebx as the original loads it). Every register kept.
pub unsafe fn draw_stage_best_and_fastest(
    times: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use crate::strings::append_string_by_id::push_string;
    unsafe {
        let record = stage_record();
        if record == 0 {
            return Err(BadInput("no stage record").into());
        }
        let mut speed = crate::strings::append_speed::append_speed(
            get(record + 0x30),
            true,
            crate::strings::units::units(),
        )?;
        let mut palette = 0;
        for (id, y) in [(0x34Bu32, 0x1FC), (0x34C, 0x213)] {
            let mut bytes = Vec::new();
            push_string(&mut bytes, id);
            bytes.extend(b":  ");
            push_string(&mut bytes, 0);
            palette = line(bytes, 0x51E5A4, 0x188, y, canvas)?;
        }
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        canvas.blit_glyph(BlitCall::new(
            0x1C + get(times + 8),
            0x18A,
            0x1FC,
            palette,
            sprites,
        ))?;
        let out = draw_text_at(
            font_at(0x51E584),
            (times + 0x20) as *const u8,
            0x1A6,
            0x1FC,
            canvas,
        )?;
        canvas.blit_glyph(BlitCall::new(
            flag_glyph(get(times + 0x10)),
            0x266,
            0x1FC,
            out.text as u32,
            sprites,
        ))?;
        let mut time = crate::strings::append_time::append_time(
            get(times + 0x14),
            crate::strings::append_time::show_hours(),
        );
        time.push(0);
        std::ptr::copy_nonoverlapping(time.as_ptr(), line_buffer(), time.len());
        crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at(
            font_at(0x51E564),
            line_buffer(),
            0x314,
            0x1FC,
            canvas,
        )?;
        canvas.blit_glyph(BlitCall::new(
            0x1C + get(record + 0x74),
            0x18A,
            0x213,
            k(0x7587EC),
            sprites,
        ))?;
        let out = draw_text_at(
            font_at(0x51E584),
            (record + 0x54) as *const u8,
            0x1A6,
            0x213,
            canvas,
        )?;
        canvas.blit_glyph(BlitCall::new(
            flag_glyph(get(record + 0x78)),
            0x266,
            0x213,
            out.text as u32,
            sprites,
        ))?;
        line(std::mem::take(&mut speed), 0x51E584, 0x2A8, 0x213, canvas)?;
        Ok(())
    }
}

/// `screens::draw_rally_stage_info` (0x489AC8): the heading
/// (`screens::draw_rally_times_heading`'s), the stage panel's body
/// (`stage_panel::draw_stage_body`: the map's markers with `dword_7587EC` as their palette,
/// times, length, surface, weather, description and number; entry +0x34 kept in
/// `dword_75ACD0`'s second dword), then `draw_stage_best_and_fastest` on its times
/// (`dword_7556E0` + 0x618 x record +0x3C). Every register kept. Refuses (before anything) no
/// stage record or times table.
pub unsafe fn draw_rally_stage_info(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
    unsafe {
        let record = stage_record();
        if record == 0 || k(0x7556E0) == 0 {
            return Err(BadInput("no stage record or times table").into());
        }
        draw_times_heading(&RALLY_STAGE_TIMES, canvas)?;
        // the two headings in `stru_51E564` here (the other stage pages: `stru_51E5C4`)
        crate::screens::stage_panel::draw_stage_body_in(
            record,
            k(0x7587EC),
            true,
            0x51E564,
            canvas,
        )?;
        let times = k(0x7556E0).wrapping_add(get(record + 0x3C).wrapping_mul(0x618));
        draw_stage_best_and_fastest(times, canvas)
    }
}
