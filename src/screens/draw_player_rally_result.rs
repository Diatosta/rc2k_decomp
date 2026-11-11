//! `screens::draw_player_rally_result` (0x4894AD): the player's result in a rally stage: the
//! labels (string 0x213 at (0xED, 0xE7), strings 0x20E .. 0x212 at x 0x155), the stage's
//! heading (`screens::draw_rally_times_heading`'s), the player's time (+0x88; string 0x513
//! for none, 0x980000) and top speed (+0x94), the rally's numbers and times
//! (`results.time_bonus` .. `results.totals`), the car's sprite, the name and the flag's sprite (their
//! palettes ebx as the original leaves it: the last text past its NUL).
//!
//!
//! In a menu loop's draw list (0x758E76). Refuses (before anything) no current player or name, no
//! rally stage records, and what `strings::append_speed` refuses. The menus and draw lists run its
//! `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the labels.
pub fn draw_player_rally_result(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let play = &menus.play;
    let player = crate::menu::player_settings::current(menus)?.get();
    let name =
        crate::screens::game_screen::player_name_of(menus, crate::menu::player_name::name(menus)?);
    (RALLY_STAGE_TIMES.records)(menus)?;
    let top_speed = player.figure;
    let speed = crate::strings::append_speed::append_speed(
        top_speed,
        true,
        crate::strings::units::units(),
    )?;
    // Converted: `rc2k_core::screens::player_result::rally_result`.
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
        &mut PlayerResultScreen::new(canvas),
        &mut |s: &mut PlayerResultScreen| draw_times_heading(menus, &RALLY_STAGE_TIMES, s.canvas),
        &p,
        &figures,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
