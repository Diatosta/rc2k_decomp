//! A page of the times of one class (`event.shown.class`) from the ranking tables (-1 ends
//! the list), ten rows from the place `champ_view.rows.count`: name (a string for kind 0, else a
//! text; the player's own in the highlight fonts), time, place and, below the first, the gap to it,
//! then the nationality's and the flag's sprites.
//!
//! Refuses a list without its end in 0x400 entries, a page that starts past the list, a row
//! drawn just before the end mark and a name past the line buffer. Unreached: no draw list that
//! holds it is used.

use crate::errors::{BadInput, Refused};
use crate::menu::champ_store::ChampView;
use crate::menu::event_store::EventState;
use crate::menu::find_class_time::find_class_time;
use crate::text::draw_text::Canvas;
use rc2k_core::saves::entries::BestTime;

/// Each row's entry and place, and the place left after the last.
type Rows = (Vec<(BestTime, u32)>, Option<u32>);

/// The rows to draw, worked out before anything is drawn.
fn rows(
    champ: &ChampView,
    event: &EventState,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Rows, Refused> {
    // The logic is in `rc2k_core::screens::results_pages::class_rows`.
    crate::menu::standings::with_a(tables, |t| {
        let found = rc2k_core::screens::results_pages::class_rows(
            |n| t.entries.get(n as usize).map_or(0, |e| e.head.group),
            |n| t.first_at(n as usize).is_none_or(|v| v == u32::MAX),
            event.shown.class.get(),
            champ.rows.count.get(),
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

pub fn draw_class_ranking(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let menus = o.menus;
    let (rows, last_place) = rows(&menus.play.champ_view, &menus.screens.event, tables)?;
    let first_time = find_class_time(menus.screens.event.shown.class.get(), true, tables)?;
    // The logic is in `rc2k_core::screens::results_pages::class_ranking` (the rows found here).
    use rc2k_core::screens::results_pages::{Name, RankedRow, class_ranking};
    let player = menus.play.champ_view.rows.ranked_player.get();
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
                    Name::Text(crate::screens::game_screen::player_name(e.head.name_id)?)
                },
                time: e.head.rally_time,
                nation: e.head.nation,
                car: e.head.car,
            })
        })
        .collect::<Result<_, BadInput>>()?;
    class_ranking(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas, o),
        first_time,
        crate::strings::append_time::show_hours(),
        &rows,
        last_place,
    )?;
    Ok(())
}
