//! The championship's standings table (`sub_4A4E1E` -> `sub_4A256C`; stream A, part F1,
//! 2026-10-18): ten rows from position `dword_7556B8` of class `dword_7558F4` in the
//! championship history (`dword_7556F0`: 0x78-byte entries, -1 ends them; +4 the player's
//! own, +8 the car, +0xC the flag, +0x10 the class, +0x14 the name, +0x54 .. +0x68 six
//! rounds' points, below 0 none, +0x6C the total, +0x70 the position).

use crate::errors::{FatalError, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::{font_at, line_buffer};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

fn w(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

const NOT_FOUND: FatalError = FatalError {
    code: 0x8070_A7FA,
    message: "Position in GB_ChampHST Not Found (see Log)",
};

/// `screens::find_history_entry` (0x4A256C): from entry edi on, the first of class ebx and
/// position eax (the entry edi points at itself is looked at before the end test); none
/// before an entry of -1 is fatal error 0x8070A7FA.
pub unsafe fn find_history_entry(
    mut entry: u32,
    class: u32,
    position: u32,
) -> Result<u32, Refused> {
    unsafe {
        loop {
            if get(entry + 0x10) == class && get(entry + 0x70) == position {
                return Ok(entry);
            }
            entry = entry.wrapping_add(0x78);
            if get(entry) == u32::MAX {
                return Err(NOT_FOUND.into());
            }
        }
    }
}

/// `screens::draw_championship_table` (0x4A4E1E): the heads (strings 0x484, 0x485, 0x4F9 at
/// y 0xDC), the six rounds' flags (sprites 0x90 .. 0x95 in event mode 0x13, else 0x8A .. 0x8F,
/// at y 0x8C), then the ten rows (`dword_755579` from `dword_7556B8` to `dword_7556BC` =
/// it + 10): position, name, the rounds' points ("-" alone for below 0), the total, the car's
/// and flag's sprites; the player's own (+4 above 0) in the highlight fonts; each text from
/// the last one's y, rows 0x19 apart from y 0xF0. The sprites' palettes ebx as the original
/// leaves it. (It also looks the section's first record up and never uses it: left out.)
/// Every register kept. Refuses (before anything) no history table; a position not in it is
/// the original's fatal error.
pub unsafe fn draw_championship_table(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let table = k(0x7556F0);
        if table == 0 {
            return Err(crate::errors::BadInput("no championship history").into());
        }
        let mut palette = 0;
        for (id, x, font) in [
            (0x484u32, 0x73, 0x51E5A4),
            (0x485, 0x9C, 0x51E584),
            (0x4F9, 0x2DE, 0x51E584),
        ] {
            palette = draw_string_by_id_at(id, font_at(font), x, 0xDC, canvas)?.text as u32;
        }
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        let first = if k(0x7556D0) == 0x13 { 0x90 } else { 0x8A };
        for n in 0..6u32 {
            canvas.blit_glyph(BlitCall::new(
                first + n,
                0x196 + 0x36 * n as i32,
                0x8C,
                palette,
                sprites,
            ))?;
        }
        w(0x755579, k(0x7556B8));
        w(0x7556BC, k(0x7556B8).wrapping_add(10));
        let mut y = 0xF0i32;
        loop {
            let entry = find_history_entry(k(0x7556F0), k(0x7558F4), k(0x755579))?;
            let mine = (get(entry + 4) as i32) > 0;
            let (pos_font, font) = if mine {
                (0x51E544, 0x51E524)
            } else {
                (0x51E5A4, 0x51E584)
            };
            let number = |v: u32, signed: bool| -> Vec<u8> {
                let mut b = if signed && (v as i32) < 0 {
                    b"-".to_vec()
                } else {
                    crate::strings::append_u32::append_u32(v)
                };
                b.push(0);
                b
            };
            let text = |bytes: Vec<u8>,
                        f: usize,
                        x: i32,
                        y: i32,
                        canvas: &mut dyn Canvas|
             -> Result<(i32, u32), Refused> {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len());
                let out = draw_text_at(font_at(f), line_buffer(), x, y, canvas)?;
                Ok((out.y, out.text as u32))
            };
            (y, _) = text(number(get(entry + 0x70), false), pos_font, 0x73, y, canvas)?;
            y = draw_text_at(font_at(font), (entry + 0x14) as *const u8, 0x9C, y, canvas)?.y;
            for (n, x) in [0x19A, 0x1D0, 0x206, 0x23C, 0x272, 0x2A8]
                .into_iter()
                .enumerate()
            {
                (y, _) = text(
                    number(get(entry + 0x54 + 4 * n as u32), true),
                    font,
                    x,
                    y,
                    canvas,
                )?;
            }
            let palette;
            (y, palette) = text(number(get(entry + 0x6C), false), font, 0x2DE, y, canvas)?;
            canvas.blit_glyph(BlitCall::new(
                get(entry + 8).wrapping_add(0x1C),
                0x7E,
                y,
                palette,
                sprites,
            ))?;
            let mut flag = get(entry + 0xC);
            if (0x18..=0x19).contains(&flag) {
                flag ^= 1;
            }
            canvas.blit_glyph(BlitCall::new(
                flag.wrapping_add(0x44),
                0x154,
                y,
                palette,
                sprites,
            ))?;
            y = y.wrapping_add(0x19);
            w(0x755579, k(0x755579).wrapping_add(1));
            if k(0x755579) == k(0x7556BC) {
                return Ok(());
            }
        }
    }
}

/// Whether every row's position is in the history (for the tests).
pub unsafe fn championship_table_ready() -> bool {
    unsafe {
        let table = k(0x7556F0);
        table != 0
            && (0..10).all(|n| {
                find_history_entry(table, k(0x7558F4), k(0x7556B8).wrapping_add(n)).is_ok()
            })
    }
}
