//! The championship's standings page: ten rows of the history from the first row, with
//! the column headings, the six rounds' sprites, and per row the position, name, six rounds'
//! points and total (a lone "-" below 0), the nationality's sprite and the flag; a player's own
//! row in the highlight fonts. The rows are found before anything is drawn (a position not there
//! is the look-up's fatal error). Refuses no history, a section the record table lacks and what
//! the look-up refuses.

use crate::errors::{BadInput, Refused};
use crate::menu::ranked_lookups::find_champ_position;
use crate::screens::best_scores::section_there;
use crate::screens::game_screen::{HeldText, held_name};
use crate::text::draw_text::Canvas;

pub fn draw_championship_standings(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let play = &menus.play;
    if !section_there(menus, system)? {
        return Err(BadInput("a section past the record table").into());
    }
    let first_row = play.champ_view.first_row.get();
    let end = first_row.wrapping_add(10);
    let mut rows = Vec::new();
    let mut row = first_row;
    loop {
        rows.push(find_champ_position(row)?);
        row = row.wrapping_add(1);
        if row == end || rows.len() > 0x400 {
            break;
        }
    }
    // The logic is in `rc2k_core::screens::results_pages::championship_standings` (the rows found
    // here).
    use rc2k_core::screens::results_pages::{StandingRow, championship_standings};
    let rows: Vec<StandingRow<HeldText>> = rows
        .into_iter()
        .map(|e| {
            Ok(StandingRow {
                player: (e.player as i32) > 0,
                position: e.position,
                name: held_name(e.name_text())?,
                points: e.overall,
                nation: e.nation,
                car: e.car,
            })
        })
        .collect::<Result<_, BadInput>>()?;
    championship_standings(
        &mut crate::screens::game_screen::GameScreen::<_, _, HeldText>::new(canvas, o),
        first_row,
        play.champ_view.event_mode.get() == 0x13,
        &rows,
    )?;
    Ok(())
}
