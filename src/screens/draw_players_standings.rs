//! The players' standings: the heads, then four rows (three when `event.player_count` is 3;
//! two players still get four, kept) in the order of `results.order`: name, car sprite, flag,
//! place and points. Refuses a row's player without its record, name or points.

use crate::errors::{BadInput, Refused};
use crate::menu::captions_store::CaptionTables;
use crate::text::draw_text::Canvas;

pub fn draw_players_standings(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let rows = if menus.screens.event.player_count.get() == 3 {
        3
    } else {
        4
    };
    let mut totals = [0u32; 4];
    for (row, total) in totals.iter_mut().enumerate().take(rows) {
        if menus.play.players.players[row]
            .get()
            .name_player()
            .is_none()
        {
            return Err(BadInput("a standings row without its name").into());
        }
        *total = CaptionTables::total_of(menus, row)?.get();
    }
    // The logic is in `rc2k_core::screens::players_results::standings`.
    let player = |row: usize| menus.play.players.players[row].get();
    let rows: Vec<
        rc2k_core::screens::players_results::Standing<crate::screens::game_screen::HeldText>,
    > = (0..rows)
        .map(|row| {
            Ok(rc2k_core::screens::players_results::Standing {
                name: crate::screens::game_screen::player_name_of(
                    crate::menu::player_names::name_of(&player(row))?,
                ),
                nation: player(row).nationality,
                car: player(row).car_number,
                points: totals[row],
            })
        })
        .collect::<Result<_, BadInput>>()?;
    rc2k_core::screens::players_results::standings(
        &mut crate::screens::game_screen::GameScreen::<_, _, crate::screens::game_screen::HeldText>::new(canvas, o),
        &rows,
    )?;
    Ok(())
}
