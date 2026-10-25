//! `screens::draw_championship_standings` (0x4A5410; IDA had it as code outside any
//! function, made one 2026-10-19): the championship's standings page: ten rows of the
//! history (`g_champHistory`) from row `g_champFirstRow`.
//!
//! The section's first record is looked up (`records::find_record`) and, in event mode 0x13,
//! its name's place worked out, neither of them used. The column headings: string 0x484 at
//! (0x73, 0xDC) in 0x51E5A4, 0x485 at (0x9C, 0xDC) and 0x4F9 at (0x2DE, 0xDC) in 0x51E584;
//! the six rounds' sprites at y 0x8C, x 0x196 .. 0x2A4 (0x36 apart): 0x90 .. 0x95 in event
//! mode 0x13 (`g_eventMode`), else 0x8A .. 0x8F, with the palette the last heading left
//! (ebx). `g_champRow` from the first row to `g_champRowEnd` (first + 10), each row
//! (`menu::find_champ_position`) from y 0xF0, 0x19 apart, a player's (+4 above 0) in the
//! highlight fonts 0x51E544 / 0x51E524, else 0x51E5A4 / 0x51E584: its position (+0x70) at
//! 0x73, its name (the text at +0x14) at 0x9C, the six rounds' points and the total (+0x38
//! .. +0x50; below 0 a lone "-") at 0x19A .. 0x2DE (0x36 apart); the nationality's sprite
//! (0x1C + +8) at 0x7E and the flag's (+0xC, 0x18 and 0x19 swapped, + 0x44) at 0x154 with
//! the palette the last text left. Each drawer's y is edx as the text drawers leave it.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! In the standings screen's draw list (0x758E5E). The rows are found before anything is drawn (a
//! position not there is the look-up's fatal error, raised then). Refuses (before anything) no
//! history, a section the record table has not (the look-up runs on), and what the look-up refuses.
//! The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::ranked_lookups::find_champ_position;
use crate::screens::best_scores::section_there;
use crate::text::draw_text::Canvas;

// g_champRow
// g_champRowEnd
fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version.
pub unsafe fn draw_championship_standings(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let history = CHAMP_VIEW.champ_history.get();
    if history == 0 {
        return Err(BadInput("no championship history").into());
    }
    if !section_there() {
        return Err(BadInput("a section past the record table").into());
    }
    let first_row = CHAMP_VIEW.first_row.get();
    let end = first_row.wrapping_add(10);
    let mut rows = Vec::new();
    let mut row = first_row;
    loop {
        rows.push(find_champ_position(history, row)?);
        row = row.wrapping_add(1);
        if row == end || rows.len() > 0x400 {
            break;
        }
    }
    // Converted: `rc2k_core::screens::results_pages::championship_standings` (the rows found here).
    use rc2k_core::screens::results_pages::{StandingRow, championship_standings};
    let rows: Vec<StandingRow<u32>> = rows
        .into_iter()
        .map(|e| StandingRow {
            player: (get(e + 4) as i32) > 0,
            position: get(e + 0x70),
            name: e + 0x14,
            points: std::array::from_fn(|k| get(e + 0x38 + 4 * k as u32)),
            nation: get(e + 8),
            car: get(e + 0xC),
        })
        .collect();
    championship_standings(
        &mut crate::screens::game_screen::GameScreen::new(canvas),
        first_row,
        CHAMP_VIEW.event_mode.get() == 0x13,
        &rows,
    )?;
    Ok(())
}
