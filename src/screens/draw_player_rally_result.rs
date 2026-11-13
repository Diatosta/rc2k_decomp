//! The player's result in a rally stage: the labels, the stage's heading, the player's time
//! (string 0x513 for none) and top speed, the rally's numbers and times, the car's sprite, the name
//! and the flag's sprite. Refuses no current player or name, no rally stage records and what
//! `strings::append_speed` refuses.

use crate::errors::Refused;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the labels.
pub fn draw_player_rally_result(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let play = &menus.play;
    let player = crate::menu::player_settings::current(menus)?.get();
    let name = crate::screens::game_screen::player_name_of(crate::menu::player_name::name(menus)?);
    (RALLY_STAGE_TIMES.records)(menus, system)?;
    let top_speed = player.figure;
    let speed = crate::strings::append_speed::append_speed(
        crate::strings::append_string_by_id::StringOwners::of(o),
        top_speed,
        true,
        crate::strings::units::units(),
    )?;
    // The logic is in `rc2k_core::screens::player_result::rally_result`.
    use crate::screens::game_screen::PlayerResultScreen;
    let p = rc2k_core::screens::player_result::Player {
        name,
        nation: player.nationality,
        car: player.car_number,
        time: player.time,
        speed,
    };
    let figures = rc2k_core::screens::player_result::RallyFigures {
        position: play.results.positions[0].get(),
        best: play.results.time_bonus[0].get(),
        column: [
            play.results.points[0].get(),
            play.results.time_bonus[0].get(),
            play.results.extra[0].get(),
            play.results.sums[0].get(),
            play.results.before[0].get(),
            play.results.totals[0].get(),
        ],
    };
    rc2k_core::screens::player_result::rally_result(
        &mut PlayerResultScreen::new(canvas, o),
        &mut |s: &mut PlayerResultScreen| {
            draw_times_heading(o, system, &RALLY_STAGE_TIMES, s.canvas)
        },
        &p,
        &figures,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
