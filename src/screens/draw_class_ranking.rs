//! `screens::draw_class_ranking` (0x47C16A): a page of the times of one class
//! (`dword_7558F4`) from the list at `dword_7556EC` (entries of 0x9C bytes, -1 ends it;
//! +4 the driver's kind or player, +8 the nationality, +0xC the name (a string id when +4
//! is 0, else a text), +0x10 the flag, +0x18 the time, +0x1C the class), ten rows from the
//! place `dword_7556B4` on.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! `g_firstPlayerTime` the class's first time (`menu::find_class_time`, ebx 1). The rows
//! before the place are passed over (counting the class's entries from 1 until the place).
//! Each row (`seconds` its place, and one past the last when the list ends after it, y from 0xF0, 0x19 apart, each draw's y handed on as edx
//! is): the name at x 0xFA (font 0x51E584; a string for kind 0, else the text; the player
//! `dword_7556C0` in 0x51E524 with the fonts below changed to theirs), the time at 0x258
//! (`draw_time_fixed_pitch`, 0x51E564 / 0x51E504), the place at 0xBE (0x51E5A4 / 0x51E544)
//! and below the first the gap (`glyph`): string 0x579 at 0x276 and the time at 0x2E4;
//! then `dword_7559B0` = y + 4, the nationality's sprite (0x1C + +8) at x 0xD2 and the
//! flag's (`+0x10` with 0x18 and 0x19 swapped, + 0x44) at 0x1AE, both from `g_spriteTable`
//! with ebx as the draws leave it as the palette.
//!
//! Refuses (before anything) a list without its end in 0x400 entries, a page that starts past
//! the list (the original reads past its end), a row drawn just before the end mark (the
//! original then leaves with a dword still pushed: its `popa` and `retn` take the wrong ones,
//! the original's slip), a name past the line buffer. In the draw list 0x758E46, whose address
//! nothing in RAL.EXE holds (every byte searched, 2026-10-20): unreached. Hook removed
//! 2026-10-22 (typed tables): the menus and draw lists run its `entry` (`menu::table_calls`);
//! the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::find_class_time::{ENTRY_LEN, find_class_time};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

const LIST_RVA: usize = rva(0x7556EC); // dword_7556EC
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const FIRST_PLACE_RVA: usize = rva(0x7556B4); // dword_7556B4
const PLAYER_RVA: usize = rva(0x7556C0); // dword_7556C0
// g_firstPlayerTime
// seconds
// glyph
// dword_7559B0
fn g(a: usize) -> u32 {
    unsafe { game_read(a) }
}
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(u32, u32)>, Option<u32>);

/// The rows to draw, worked out before anything is drawn.
fn rows() -> Result<Rows, Refused> {
    // Converted: `rc2k_core::screens::results_pages::class_rows`.
    let list = g(LIST_RVA);
    let at = |n: u32| list.wrapping_add(n.wrapping_mul(ENTRY_LEN));
    let (rows, after) = rc2k_core::screens::results_pages::class_rows(
        |n| get(at(n) + 0x1C),
        |n| get(at(n)) == u32::MAX,
        g(CLASS_RVA),
        g(FIRST_PLACE_RVA),
    )
    .map_err(BadInput::from)?;
    Ok((
        rows.into_iter().map(|(n, place)| (at(n), place)).collect(),
        after,
    ))
}

/// The Rust version.
pub unsafe fn draw_class_ranking(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (rows, last_place) = rows()?;
    let first_time = find_class_time(g(LIST_RVA), g(CLASS_RVA), true)?;
    // Converted: `rc2k_core::screens::results_pages::class_ranking` (the rows found here).
    use rc2k_core::screens::results_pages::{Name, RankedRow, class_ranking};
    let player = g(PLAYER_RVA);
    let rows: Vec<RankedRow<u32>> = rows
        .iter()
        .map(|&(e, place)| {
            let kind = get(e + 4);
            RankedRow {
                place,
                mine: kind != 0 && kind == player,
                name: if kind == 0 {
                    Name::String(get(e + 0xC))
                } else {
                    Name::Text(get(e + 0xC))
                },
                time: get(e + 0x18),
                nation: get(e + 8),
                car: get(e + 0x10),
            }
        })
        .collect();
    class_ranking(
        &mut crate::screens::game_screen::GameScreen::new(canvas),
        first_time,
        crate::strings::append_time::show_hours(),
        &rows,
        last_place,
    )?;
    Ok(())
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_class_ranking(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_class_ranking", r, &rust)
}
