//! Ten rows of a section's best times (six sections): name, time, number, the nationality's
//! flag and the car's sprite, with a player's own row in the highlight fonts. The second, wrong
//! sprite once drawn over each car is gone (fixed: "The section best times drew a second, wrong
//! sprite" in `wiki/Fixed-bugs.md`). Refuses a section past the six or with no table and a line
//! past the line buffer. Only in the draw list of `menu::unused_section_times_loop_1` / `_2`,
//! which nothing reaches.

use crate::errors::{BadInput, Refused};
use crate::strings::append_time::show_hours;
use crate::text::draw_text::Canvas;

const SECTIONS: u32 = 6;

pub fn draw_section_best_times(
    o: crate::owners::Owners<'static>,
    saves: &crate::saves::owner::Saves,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let section: u32 = menus.screens.event.read().section;
    if section >= SECTIONS {
        return Err(BadInput("a section past the six best-times tables").into());
    }
    let table = saves.rally_times.table(section as usize)?;
    // The logic is in `rc2k_core::screens::best_scores::section_best_times`.
    use rc2k_core::screens::best_scores::SectionRow;
    // The rallies' tables are the game's (`RALLY_TIMES`), not blocks.
    // The names are read now: the rows are drawn below, in this call.
    let names: [crate::screens::game_screen::HeldText; 10] = {
        let mut held = Vec::with_capacity(10);
        for e in &table[..10] {
            held.push(crate::screens::game_screen::held_field(&e.name)?);
        }
        held.try_into()
            .map_err(|_| BadInput("ten best times' names"))?
    };
    let rows: [SectionRow<crate::screens::game_screen::HeldText>; 10] = std::array::from_fn(|n| {
        let e = &table[n];
        SectionRow {
            name: names[n],
            time: e.rally_time.get(),
            nation: e.nation.get(),
            car: e.car.get(),
            player: e.is_player.get() == 1,
        }
    });
    rc2k_core::screens::best_scores::section_best_times(
        &mut crate::screens::best_scores::ScoresScreen::new(canvas, o),
        &rows,
        show_hours(o.strings),
    )?;
    Ok(())
}
