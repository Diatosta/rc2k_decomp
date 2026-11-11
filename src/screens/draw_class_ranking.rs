//! `screens::draw_class_ranking` (0x47C16A): a page of the times of one class
//! (`event.class`) from the list `champ_view.ranked_times` (entries of 0x9C bytes, -1 ends it;
//! +4 the driver's kind or player, +8 the nationality, +0xC the name (a string id when +4
//! is 0, else a text), +0x10 the flag, +0x18 the time, +0x1C the class), ten rows from the
//! place `champ_view.count` on.
//!
//!
//! `g_firstPlayerTime` the class's first time (`menu::find_class_time`, ebx 1). The rows
//! before the place are passed over (counting the class's entries from 1 until the place).
//! Each row (`seconds` its place, and one past the last when the list ends after it, y from 0xF0, 0x19 apart, each draw's y handed on as edx
//! is): the name at x 0xFA (font 0x51E584; a string for kind 0, else the text; the player
//! `champ_view.ranked_player` in 0x51E524 with the fonts below changed to theirs), the time at 0x258
//! (`draw_time_fixed_pitch`, 0x51E564 / 0x51E504), the place at 0xBE (0x51E5A4 / 0x51E544)
//! and below the first the gap (`glyph`): string 0x579 at 0x276 and the time at 0x2E4;
//! then `draw_work.scratch_b0` = y + 4, the nationality's sprite (0x1C + +8) at x 0xD2 and the
//! flag's (`+0x10` with 0x18 and 0x19 swapped, + 0x44) at 0x1AE, both from `g_spriteTable`
//! with ebx as the draws leave it as the palette.
//!
//! Refuses (before anything) a list without its end in 0x400 entries, a page that starts past the
//! list (the original reads past its end), a row drawn just before the end mark (the original then
//! leaves with a dword still pushed, so its return takes the wrong ones: the original's
//! slip), a name past the line buffer. In the draw list 0x758E46, whose address nothing in RAL.EXE
//! holds (every byte searched, 2026-10-20): unreached. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::{BadInput, Refused};
use crate::menu::find_class_time::find_class_time;
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;
use rc2k_core::saves::entries::BestTime;

// g_firstPlayerTime
// seconds
// glyph
/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(BestTime, u32)>, Option<u32>);

/// The rows to draw, worked out before anything is drawn.
fn rows(menus: &Menus) -> Result<Rows, Refused> {
    // Converted: `rc2k_core::screens::results_pages::class_rows`.
    crate::menu::standings::with_a(|t| {
        let found = rc2k_core::screens::results_pages::class_rows(
            |n| t.entries.get(n as usize).map_or(0, |e| e.head.group),
            |n| t.first_at(n as usize).is_none_or(|v| v == u32::MAX),
            menus.screens.event.class.get(),
            menus.play.champ_view.count.get(),
        );
        let (rows, after) = found.map_err(BadInput::from)?;
        Ok((
            rows.into_iter()
                .map(|(n, place)| (t.entries[n as usize], place))
                .collect(),
            after,
        ))
    })?
}

pub fn draw_class_ranking(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let (rows, last_place) = rows(menus)?;
    let first_time = find_class_time(menus.screens.event.class.get(), true)?;
    // Converted: `rc2k_core::screens::results_pages::class_ranking` (the rows found here).
    use rc2k_core::screens::results_pages::{Name, RankedRow, class_ranking};
    let player = menus.play.champ_view.ranked_player.get();
    // A player's entry names the player's name (`name_at`), a driver's a string.
    let rows: Vec<RankedRow<crate::screens::game_screen::HeldText>> = rows
        .iter()
        .map(|&(e, place)| {
            let kind = e.head.player;
            Ok(RankedRow {
                place,
                mine: kind != 0 && kind == player,
                name: if kind == 0 {
                    Name::String(e.head.name_id)
                } else {
                    Name::Text(crate::screens::game_screen::player_name(
                        menus,
                        e.head.name_id,
                    )?)
                },
                time: e.head.rally_time,
                nation: e.head.nation,
                car: e.head.car,
            })
        })
        .collect::<Result<_, BadInput>>()?;
    class_ranking(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas),
        first_time,
        crate::strings::append_time::show_hours(),
        &rows,
        last_place,
    )?;
    Ok(())
}
