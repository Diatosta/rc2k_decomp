//! The players' stage results pages (`sub_4836CF` with the points column, `sub_48D32E`
//! without; stream A, part F1, 2026-10-18). Both draw the same columns, one loop over the rows
//! each (rows in the order `dword_758720` .. `dword_75872C`, byte offsets into `g_players`,
//! 0x17 apart from y 0xFE), shifted 0x1E right on `sub_48D32E`'s page; their differences are
//! the `Layout`. The original's slips are kept (bugs.md): `sub_4836CF` draws four rows for two
//! players, tests the first row's time with `jl` (the others with equality) and puts the third
//! row's time at 0x1B2.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::{font_at, line_buffer};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};

pub const NO_TIME: u32 = 0x36EE80;
const GAP_RVA: usize = rva(0x7559B8); // dword_7559B8: the gap, the points, the spaced length
const FIRST_TIME_RVA: usize = rva(0x7559BC); // dword_7559BC

/// What differs between the two pages.
pub struct Layout {
    /// Added to every column's x but the stage's and the "no time" string's.
    pub dx: i32,
    /// Two players give two rows (`sub_48D32E`) or four (`sub_4836CF`).
    pub two_rows_for_two: bool,
    /// The first row's "no time" test: signed `>= NO_TIME` (`jl`) or `== NO_TIME`.
    pub first_time_signed: bool,
    /// The third row's time's x (0x1B2 on `sub_4836CF`, the column's 0x1E4 + dx on the other).
    pub third_time_x: i32,
    /// The head string 0x21A at 0x2BB and the points column (`off_758992`).
    pub points: bool,
    /// After the third row's flag, sprite `dword_7559B4` there too (`sub_48D32E`).
    pub third_row_extra: bool,
}

pub const WITH_POINTS: Layout = Layout {
    dx: 0,
    two_rows_for_two: false,
    first_time_signed: true,
    third_time_x: 0x1B2,
    points: true,
    third_row_extra: false,
};

pub const WITHOUT_POINTS: Layout = Layout {
    dx: 0x1E,
    two_rows_for_two: true,
    first_time_signed: false,
    third_time_x: 0x202,
    points: false,
    third_row_extra: true,
};

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn rows(layout: &Layout) -> usize {
    match k(0x755984) {
        3 => 3,
        2 if layout.two_rows_for_two => 2,
        _ => 4,
    }
}

fn offset(row: usize) -> u32 {
    k(0x758720 + 4 * row)
}

/// The row's player record, refused for an offset past the four or a missing record or name.
fn player(row: usize) -> Result<u32, Refused> {
    let o = offset(row);
    if o >= 16 || !o.is_multiple_of(4) {
        return Err(BadInput("a row's offset past g_players' four").into());
    }
    let p = unsafe { get(game_addr(rva(0x75647C)) as u32 + o) };
    if p == 0 || unsafe { get(p + 0x18) } == 0 {
        return Err(BadInput("a row's player without its record or name").into());
    }
    Ok(p)
}

/// The row's points' place (`off_758992[offset]`).
fn points_at(row: usize) -> Result<u32, Refused> {
    let p = unsafe { get(game_addr(rva(0x758992)) as u32 + offset(row)) };
    if p == 0 || p == u32::MAX {
        return Err(BadInput("a row without its points").into());
    }
    Ok(p)
}

/// The section's first record and the stage's (`off_735512[dword_7558C8][dword_7558D0]`, +
/// 0xA4 x `dword_7558CC`); none: 0.
fn records() -> (u32, u32) {
    unsafe {
        let table = get(game_addr(rva(0x735512)) as u32 + 4 * k(0x7558C8));
        if table == 0 {
            return (0, 0);
        }
        let base = get(table + 4 * k(0x7558D0));
        (base, base.wrapping_add(k(0x7558CC).wrapping_mul(0xA4)))
    }
}

fn nul(mut v: Vec<u8>) -> Vec<u8> {
    v.push(0);
    v
}

unsafe fn line(bytes: &[u8]) -> *mut u8 {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
    line_buffer()
}

fn flag_glyph(flag: u32) -> u32 {
    (if (0x18..=0x19).contains(&flag) {
        flag ^ 1
    } else {
        flag
    })
    .wrapping_add(0x44)
}

/// The rows' players, the section's first record, the stage line and the speeds.
struct Prepared {
    players: Vec<u32>,
    base: u32,
    stage: Vec<u8>,
    speeds: Vec<Vec<u8>>,
}

/// What the page needs, checked before anything is written: the rows' players (and points),
/// the stage line, the speeds (3 .. 0x80 characters for `draw_spaced_number`).
unsafe fn prepare(layout: &Layout) -> Result<Prepared, Refused> {
    unsafe {
        let n = rows(layout);
        let players = (0..n).map(player).collect::<Result<Vec<_>, _>>()?;
        if layout.points {
            for row in 0..n {
                points_at(row)?;
            }
        }
        let (base, record) = records();
        if base == 0 {
            return Err(BadInput("no stage records").into());
        }
        let stage = crate::screens::page_settings::stage_line_bytes(0x482, record)?;
        if stage.len() > 0x100 {
            return Err(BadInput("a stage line past the line buffer").into());
        }
        let units = crate::strings::units::units();
        let mut speeds = Vec::new();
        for p in &players {
            let s = crate::strings::append_speed::append_speed(get(p + 0x94), false, units)?;
            if !(3..=0x80).contains(&s.len()) {
                return Err(BadInput("a speed draw_spaced_number refuses").into());
            }
            speeds.push(s);
        }
        Ok(Prepared {
            players,
            base,
            stage,
            speeds,
        })
    }
}

/// Whether the page would refuse (for the tests).
pub unsafe fn refuses(layout: &Layout) -> bool {
    unsafe { prepare(layout) }.is_err()
}

/// The page.
pub unsafe fn draw_players_results(
    layout: &Layout,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
    unsafe {
        let Prepared {
            players,
            base,
            stage,
            speeds,
        } = prepare(layout)?;
        let dx = layout.dx;
        let (font, small, stage_font) = (font_at(0x51E584), font_at(0x51E564), font_at(0x51E9A4));
        let y = |row: usize| 0xFE + 0x17 * row as i32;
        let units_head = if crate::strings::units::units() != 0 {
            0x574
        } else {
            0x573
        };
        let mut heads = vec![
            (0x218, 0x55 + dx),
            (0x219, 0x9C + dx),
            (0x21C, 0x176 + dx),
            (units_head, 0x25A + dx),
        ];
        if layout.points {
            heads.push((0x21A, 0x2BB));
        }
        for (id, x) in heads {
            draw_string_by_id_at(id, font, x, 0xE7, canvas)?;
        }
        draw_string_by_id_at(get(base + 0x38), stage_font, 0x2EB, 0x77, canvas)?;
        draw_text_at(stage_font, line(&stage), 0x2EB, 0x91, canvas)?;
        for (row, p) in players.iter().enumerate() {
            draw_text_at(font, get(p + 0x18) as *const u8, 0x9C + dx, y(row), canvas)?;
        }
        for (row, speed) in speeds.iter().enumerate() {
            let text = line(&nul(speed.clone()));
            crate::screens::draw_spaced_number::draw_spaced_number(
                small,
                text,
                text.add(speed.len()),
                0x25A + dx,
                y(row),
                canvas,
            )?;
        }
        // Every sprite's palette is ebx as the original leaves it: g_lineBuffer after the
        // spaced numbers, then after the times.
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        let mut palette = line_buffer() as u32;
        for (row, p) in players.iter().enumerate() {
            canvas.blit_glyph(BlitCall::new(
                get(p + 4).wrapping_add(0x1C),
                0x7F + dx,
                y(row),
                palette,
                sprites,
            ))?;
        }
        for (row, p) in players.iter().enumerate() {
            let time = get(p + 0x88);
            let none = if row == 0 && layout.first_time_signed {
                time as i32 >= NO_TIME as i32
            } else {
                time == NO_TIME
            };
            if none {
                palette = draw_string_by_id_at(0x4FA, small, 0x1B8, y(row), canvas)?.text as u32;
            } else {
                let x = if row == 2 {
                    layout.third_time_x
                } else {
                    0x1E4 + dx
                };
                draw_time_fixed_pitch_at(
                    small,
                    line(&nul(append_time(time, show_hours()))),
                    x,
                    y(row),
                    canvas,
                )?;
                palette = line_buffer() as u32;
            }
        }
        for (row, p) in players.iter().enumerate() {
            canvas.blit_glyph(BlitCall::new(
                flag_glyph(get(p + 8)),
                0x141 + dx,
                y(row),
                palette,
                sprites,
            ))?;
            if row == 2 && layout.third_row_extra {
                canvas.blit_glyph(BlitCall::new(
                    k(0x7559B4),
                    0x141 + dx,
                    y(row),
                    palette,
                    sprites,
                ))?;
            }
        }
        let first = get(players[0] + 0x88);
        game_write(FIRST_TIME_RVA, first);
        for (row, p) in players.iter().enumerate().skip(1) {
            let time = get(p + 0x88);
            if time == NO_TIME {
                continue;
            }
            let gap = time.wrapping_sub(first);
            game_write(GAP_RVA, gap);
            draw_string_by_id_at(0x579, font, 0x1DF + dx, y(row), canvas)?;
            draw_time_fixed_pitch_at(
                small,
                line(&nul(append_time(gap, show_hours()))),
                0x257 + dx,
                y(row),
                canvas,
            )?;
        }
        for row in 0..players.len() {
            draw_string_by_id_at(0x4FE + row as u32, font, 0x55 + dx, y(row), canvas)?;
        }
        if layout.points {
            for row in 0..players.len() {
                let points = get(points_at(row)?);
                game_write(GAP_RVA, points);
                draw_text_at(
                    font,
                    line(&nul(crate::strings::append_u32::append_u32(points))),
                    0x2BB,
                    y(row),
                    canvas,
                )?;
            }
        }
        Ok(())
    }
}
