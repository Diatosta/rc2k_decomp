//! `screens::draw_player_rally_result` (0x4894AD): the player's result in a rally stage: the
//! labels (string 0x213 at (0xED, 0xE7), strings 0x20E .. 0x212 at x 0x155), the stage's
//! heading (`screens::draw_rally_times_heading`'s), the player's time (+0x88; string 0x513
//! for none, 0x980000) and top speed (+0x94), the rally's numbers and times
//! (`dword_758758` .. `dword_7587B8`), the car's sprite, the name and the flag's sprite (their
//! palettes ebx as the original leaves it: the last text past its NUL).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E76). Refuses (before anything) no current player or name, no
//! rally stage records, and what `strings::append_speed` refuses. The menus and draw lists run its
//! `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::results_store::RESULT_FIGURES;
use crate::screens::best_scores::{RALLY_STAGE_TIMES, draw_times_heading};
use crate::text::draw_text::Canvas;

/// (id, x, y, font) of the labels.
/// The Rust version.
pub fn draw_player_rally_result(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    let p = crate::menu::player_settings::current()?;
    let fault = crate::menu::event_flow::Fault::default();
    let f = |o: u32| p.get().word(o as usize);
    let name = crate::menu::player_name::name()?;
    if (RALLY_STAGE_TIMES.records)()? == 0 {
        return Err(crate::errors::BadInput("no rally stage records").into());
    }
    let top_speed = f(0x94);
    fault.check()?;
    let speed = crate::strings::append_speed::append_speed(
        top_speed,
        true,
        crate::strings::units::units(),
    )?;
    // Converted: `rc2k_core::screens::player_result::rally_result`.
    use crate::screens::game_screen::PlayerResultScreen;
    let p = rc2k_core::screens::player_result::Player {
        name,
        nation: f(4),
        car: f(8),
        time: f(0x88),
        speed,
    };
    fault.check()?;
    let figures = rc2k_core::screens::player_result::RallyFigures {
        position: RESULT_FIGURES.positions[0].get(),
        best: RESULT_FIGURES.time_bonus[0].get(),
        column: [
            RESULT_FIGURES.points[0].get(),
            RESULT_FIGURES.time_bonus[0].get(),
            RESULT_FIGURES.extra[0].get(),
            RESULT_FIGURES.sums[0].get(),
            RESULT_FIGURES.before[0].get(),
            RESULT_FIGURES.totals[0].get(),
        ],
    };
    rc2k_core::screens::player_result::rally_result(
        &mut PlayerResultScreen::new(canvas),
        &mut |s: &mut PlayerResultScreen| draw_times_heading(&RALLY_STAGE_TIMES, s.canvas),
        &p,
        &figures,
        crate::strings::append_time::show_hours(),
    )?;
    Ok(())
}
