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
//! Refuses (before anything) a list without its end in 0x400 entries, a page that starts past the
//! list (the original reads past its end), a row drawn just before the end mark (the original then
//! leaves with a dword still pushed: its `popa` and `retn` take the wrong ones, the original's
//! slip), a name past the line buffer. In the draw list 0x758E46, whose address nothing in RAL.EXE
//! holds (every byte searched, 2026-10-20): unreached. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::memory::blocks::Place;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_flow::Fault;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::find_class_time::{ENTRY_LEN, find_class_time};
use crate::text::draw_text::Canvas;

// g_firstPlayerTime
// seconds
// glyph
// dword_7559B0
/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(Place, u32)>, Option<u32>);

/// The rows to draw, worked out before anything is drawn.
fn rows() -> Result<Rows, Refused> {
    // Converted: `rc2k_core::screens::results_pages::class_rows`.
    let list = Place::Owned(CHAMP_VIEW.ranked_times.need("no ranked list")?);
    let at = |n: u32| list.add(n.wrapping_mul(ENTRY_LEN));
    let f = Fault::default();
    let found = rc2k_core::screens::results_pages::class_rows(
        |n| f.get_at(at(n).add(0x1C)),
        |n| f.get_at(at(n)) == u32::MAX,
        EVENT_STATE.class.get(),
        CHAMP_VIEW.count.get(),
    );
    f.check()?;
    let (rows, after) = found.map_err(BadInput::from)?;
    Ok((
        rows.into_iter().map(|(n, place)| (at(n), place)).collect(),
        after,
    ))
}

/// The Rust version.
pub fn draw_class_ranking(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (rows, last_place) = rows()?;
    let list = Place::Owned(CHAMP_VIEW.ranked_times.need("no ranked list")?);
    let first_time = find_class_time(list, EVENT_STATE.class.get(), true)?;
    // Converted: `rc2k_core::screens::results_pages::class_ranking` (the rows found here).
    use rc2k_core::screens::results_pages::{Name, RankedRow, class_ranking};
    let player = CHAMP_VIEW.ranked_player.get();
    let f = Fault::default();
    let rows: Vec<RankedRow<u32>> = rows
        .iter()
        .map(|&(e, place)| {
            let kind = f.get_at(e.add(4));
            RankedRow {
                place,
                mine: kind != 0 && kind == player,
                name: if kind == 0 {
                    Name::String(f.get_at(e.add(0xC)))
                } else {
                    Name::Text(f.get_at(e.add(0xC)))
                },
                time: f.get_at(e.add(0x18)),
                nation: f.get_at(e.add(8)),
                car: f.get_at(e.add(0x10)),
            }
        })
        .collect();
    f.check()?;
    class_ranking(
        &mut crate::screens::game_screen::GameScreen::new(canvas),
        first_time,
        crate::strings::append_time::show_hours(),
        &rows,
        last_place,
    )?;
    Ok(())
}
