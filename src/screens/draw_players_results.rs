//! The players' stage results: heads, the stage, and per player (two, three or four rows by
//! `event.player_count`) name, top speed, car, time or the gap to the first, flag and place, 0x1E
//! to the right of the points page's columns (see `screens/players_results.rs`,
//! `WITHOUT_POINTS`). Refuses a row's offset past the four, a
//! row's player without its record or name, no stage records, a stage line past the line buffer
//! and a speed `screens::draw_spaced_number` refuses.

use crate::errors::Refused;
use crate::screens::players_results::{WITHOUT_POINTS, draw_players_results};
use crate::text::draw_text::Canvas;

pub fn draw(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_players_results(o, system, &WITHOUT_POINTS, canvas)
}
