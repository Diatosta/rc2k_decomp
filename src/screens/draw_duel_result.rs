//! `screens::draw_duel_result` (0x48647A): the two-player stage result: the heads (strings
//! 0x213 / 0x214, 0x20E, 0x217, 0x208, 0x210, 0x211, 0x212), the faster of players 1 and 2
//! first (`dword_758720` / `dword_758724` the two's byte offsets in `g_players`, 0 / 4 or
//! swapped when player 2's time +0x88 is below player 1's; strings 0x215 / 0x216 by it), the
//! stage's name (+0x38 of its section's first record) and stage line (string 0x482 ..) at
//! x 0x2EB, then per row the time (string 0x4FA for none, 0x36EE80; else fixed pitch at
//! 0x2AD), the car and flag sprites, the name (+0x18); by player 1 and 2 (not by order) the
//! top speeds (+0x94), the times `dword_758758` / `dword_75875C`, the strings
//! `dword_7588D2[dword_758768 - 1]` / `[dword_75876C - 1]`, and two columns of six numbers
//! (`dword_758778` .. at x 0x1C4, `dword_75877C` .. at 0x2C8).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E86). The sprites' palettes are ebx as the original
//! leaves it. Refuses (before anything) players 1 or 2 missing or without a name, no stage
//! records, a stage line past the line buffer, and a string index past `dword_7588D2`'s 21
//! (the original reads the -1 ends or past them). Compared in shadow mode (`dispatch_draw`;
//! the two offsets and the line buffer as state). Self-test: the page's values as they are,
//! with each order and "no time", nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::menu::player::player_at;
use crate::screens::draw_menu_item_text::text_unavailable;
use crate::screens::harness::{Left, check_draw, dispatch_draw};
use crate::screens::{font_at, line_buffer};
use crate::service::harness::{Region, at, capture, restore};
use crate::text::draw_string_by_id::draw_string_by_id_at;
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_duel_result", rva(0x48647A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FIRST_RVA: usize = rva(0x758720); // dword_758720: byte offset in g_players
const SECOND_RVA: usize = rva(0x758724); // dword_758724
const NO_TIME: u32 = 0x36EE80;

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

fn flag_glyph(flag: u32) -> u32 {
    (if (0x18..=0x19).contains(&flag) {
        flag ^ 1
    } else {
        flag
    })
    .wrapping_add(0x44)
}

/// The stage's section's first record and the stage's record (`off_735512[dword_7558C8]
/// [dword_7558D0]`, + 0xA4 x `dword_7558CC`); none: 0.
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

/// `dword_7588D2[v - 1]`, refused outside its 21 entries.
fn label(v: u32) -> Result<u32, Refused> {
    if !(1..=21).contains(&v) {
        return Err(BadInput("a string index past dword_7588D2's 21").into());
    }
    Ok(k(0x7588D2 + 4 * (v as usize - 1)))
}

unsafe fn line(bytes: &[u8]) -> *mut u8 {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
    line_buffer()
}

fn nul(mut v: Vec<u8>) -> Vec<u8> {
    v.push(0);
    v
}

pub unsafe fn draw_duel_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    use crate::strings::append_time::{append_time, show_hours};
    use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
    unsafe {
        let (p1, p2) = (player_at(0), player_at(1));
        if p1 == 0 || p2 == 0 || get(p1 + 0x18) == 0 || get(p2 + 0x18) == 0 {
            return Err(BadInput("player 1 or 2 missing or without a name").into());
        }
        let (base, record) = records();
        if base == 0 {
            return Err(BadInput("no stage records").into());
        }
        let stage = crate::screens::page_settings::stage_line_bytes(0x482, record)?;
        if stage.len() > 0x100 {
            return Err(BadInput("a stage line past the line buffer").into());
        }
        let (label1, label2) = (label(k(0x758768))?, label(k(0x75876C))?);
        let units = crate::strings::units::units();
        let speeds = [
            nul(crate::strings::append_speed::append_speed(
                get(p1 + 0x94),
                true,
                units,
            )?),
            nul(crate::strings::append_speed::append_speed(
                get(p2 + 0x94),
                true,
                units,
            )?),
        ];
        let (big, small) = (font_at(0x51E584), font_at(0x51E5A4));
        draw_string_by_id_at(0x213, big, 0x135, 0x114, canvas)?;
        draw_string_by_id_at(0x214, big, 0x236, 0x114, canvas)?;
        for (id, y) in [
            (0x20E, 0x12B),
            (0x217, 0x142),
            (0x208, 0x159),
            (0x210, 0x187),
            (0x211, 0x19E),
            (0x212, 0x1CD),
        ] {
            draw_string_by_id_at(id, small, 0xFD, y, canvas)?;
        }
        let swapped = get(p2 + 0x88) < get(p1 + 0x88);
        let (top, bottom) = if swapped {
            (0x216, 0x215)
        } else {
            (0x215, 0x216)
        };
        draw_string_by_id_at(top, small, 0xFD, 0xD0, canvas)?;
        draw_string_by_id_at(bottom, small, 0xFD, 0xE7, canvas)?;
        game_write(FIRST_RVA, if swapped { 4u32 } else { 0 });
        game_write(SECOND_RVA, if swapped { 0u32 } else { 4 });
        let order = if swapped { [p2, p1] } else { [p1, p2] };
        draw_string_by_id_at(get(base + 0x38), font_at(0x51E9A4), 0x2EB, 0x77, canvas)?;
        draw_text_at(font_at(0x51E9A4), line(&stage), 0x2EB, 0x91, canvas)?;
        for (p, y) in order.iter().zip([0xD0, 0xE7]) {
            let time = get(p + 0x88);
            if time == NO_TIME {
                draw_string_by_id_at(0x4FA, font_at(0x51E564), 0x281, y, canvas)?;
            } else {
                draw_time_fixed_pitch_at(
                    font_at(0x51E564),
                    line(&nul(append_time(time, show_hours()))),
                    0x2AD,
                    y,
                    canvas,
                )?;
            }
        }
        draw_text_at(big, line(&speeds[0]), 0x135, 0x159, canvas)?;
        let palette = draw_text_at(big, line(&speeds[1]), 0x236, 0x159, canvas)?.text as u32;
        let sprites = crate::gfx::blit_marker_pair::sprite_table();
        for (p, y) in order.iter().zip([0xD0, 0xE7]) {
            canvas.blit_glyph(BlitCall::new(
                get(p + 4).wrapping_add(0x1C),
                0x135,
                y,
                palette,
                sprites,
            ))?;
        }
        let mut palette = 0;
        for (p, y) in order.iter().zip([0xD0, 0xE7]) {
            palette = draw_text_at(big, get(p + 0x18) as *const u8, 0x152, y, canvas)?.text as u32;
        }
        for (p, y) in order.iter().zip([0xCF, 0xE7]) {
            canvas.blit_glyph(BlitCall::new(
                flag_glyph(get(p + 8)),
                0x1FC,
                y,
                palette,
                sprites,
            ))?;
        }
        draw_text_at(
            big,
            line(&nul(append_time(k(0x758758), show_hours()))),
            0x135,
            0x142,
            canvas,
        )?;
        draw_text_at(
            big,
            line(&nul(append_time(k(0x75875C), show_hours()))),
            0x236,
            0x142,
            canvas,
        )?;
        draw_string_by_id_at(label1, big, 0x135, 0x12B, canvas)?;
        draw_string_by_id_at(label2, big, 0x236, 0x12B, canvas)?;
        let rows = [0x12B, 0x142, 0x159, 0x187, 0x19F, 0x1CD];
        for (x, values) in [
            (
                0x1C4,
                [0x758778, 0x758758, 0x758788, 0x758798, 0x7587A8, 0x7587B8],
            ),
            (
                0x2C8,
                [0x75877C, 0x75875C, 0x75878C, 0x75879C, 0x7587AC, 0x7587BC],
            ),
        ] {
            for (a, y) in values.into_iter().zip(rows) {
                draw_text_at(
                    big,
                    line(&nul(crate::strings::append_u32::append_u32(k(a)))),
                    x,
                    y,
                    canvas,
                )?;
            }
        }
        Ok(())
    }
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_duel_result(canvas) }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&[
        (line_buffer() as u32, 0x100),
        (game_addr(FIRST_RVA) as u32, 8),
        (crate::strings::text_from_ids::text() as u32, 0x100),
    ])
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// Whether the page has what it reads.
unsafe fn ready() -> bool {
    unsafe {
        let (p1, p2) = (player_at(0), player_at(1));
        let (base, record) = records();
        p1 != 0
            && p2 != 0
            && get(p1 + 0x18) != 0
            && get(p2 + 0x18) != 0
            && base != 0
            && crate::screens::page_settings::stage_line_bytes(0x482, record).is_ok()
            && label(k(0x758768)).is_ok()
            && label(k(0x75876C)).is_ok()
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    if !unsafe { ready() } {
        return t.skip("no two players with names, stage records or labels");
    }
    let (p1, p2) = (player_at(0), player_at(1));
    if unsafe {
        [p1, p2]
            .iter()
            .any(|&p| crate::strings::append_speed::traps(get(p + 0x94)))
    } {
        return t.skip("a top speed the original traps on (mid-race data)");
    }
    let regions: Vec<Region> = vec![
        at(FIRST_RVA),
        at(SECOND_RVA),
        (p1 + 0x88, 4),
        (p2 + 0x88, 4),
    ];
    let saved = capture(&regions);
    let times = unsafe { (get(p1 + 0x88), get(p2 + 0x88)) };
    for (a, b) in [
        times,
        (6000, 5000),
        (5000, 6000),
        (NO_TIME, 5000),
        (5000, NO_TIME),
    ] {
        check_draw(
            &mut t,
            &format!("times {a} {b}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                ((p1 + 0x88) as *mut u32).write_unaligned(a);
                ((p2 + 0x88) as *mut u32).write_unaligned(b);
            },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
