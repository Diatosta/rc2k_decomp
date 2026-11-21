//! The player's result in a rally stage: the labels, the stage's heading, the player's time
//! (string 0x513 for none) and top speed, the rally's numbers and times, the car's sprite, the name
//! and the flag's sprite. Refuses no current player or name, no rally stage records and what
//! `strings::append_speed` refuses.

use crate::errors::Refused;
use crate::menu::results_store::Results;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

/// Draws the rally stage result of the current player, with the places and points in `results`.
pub fn draw_player_rally_result(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    results: &Results,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let player = crate::menu::player_settings::current(menus)?.get();
    let name = crate::screens::game_screen::player_name_of(crate::menu::player_name::name(menus)?);
    (RALLY_STAGE_TIMES.records)(menus, system)?;
    let top_speed = player.figure;
    let speed = crate::strings::append_speed::append_speed(
        crate::strings::append_string_by_id::StringOwners::of(o),
        top_speed,
        true,
        crate::strings::units::units(o.strings),
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
    let figures = results.read().players[0];
    let figures = rc2k_core::screens::player_result::RallyFigures {
        position: figures.position,
        best: figures.time_bonus,
        column: figures.column(),
    };
    rc2k_core::screens::player_result::rally_result(
        &mut PlayerResultScreen::new(canvas, o),
        &mut |s: &mut PlayerResultScreen| {
            draw_times_heading(o, system, &RALLY_STAGE_TIMES, s.canvas)
        },
        &p,
        &figures,
        crate::strings::append_time::show_hours(o.strings),
    )?;
    Ok(())
}
