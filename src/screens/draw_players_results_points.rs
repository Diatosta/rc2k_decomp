//! The players' stage results with their points: heads, the stage, and per player name, top
//! speed, car, time or the gap to the first, flag, place and points; three rows when
//! `event.player_count` is 3, else four, so two players get four (see
//! `screens/players_results.rs`, `WITH_POINTS`). Refuses a row's offset past the four, a row's
//! player without its record, name or points, no stage records, a stage line past the line buffer
//! and a speed `screens::draw_spaced_number` refuses.

use crate::errors::Refused;
use crate::menu::results_store::Results;
use crate::screens::players_results::{WITH_POINTS, draw_players_results};
use crate::text::draw_text::Canvas;

pub fn draw(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    results: &Results,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_players_results(o, system, results, &WITH_POINTS, canvas)
}
