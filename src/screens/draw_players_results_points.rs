//! `screens::draw_players_results_points` (0x4836CF): the players' stage results with their points: heads, the stage, and per player (three rows when `event.player_count` is 3, else four: two players get four) name, top speed, car, time or the gap to the first, flag, place and points (see `screens/players_results.rs`, `WITH_POINTS`).
//!
//!
//! In a menu loop's draw list (0x758E8E). The sprites' palettes are ebx as the original leaves it.
//! Refuses (before anything) a row's offset past the four, a row's player without its record or
//! name or points, no stage records, a stage line past the line buffer, and a speed
//! `screens::draw_spaced_number` refuses. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::screens::players_results::{WITH_POINTS, draw_players_results};
use crate::text::draw_text::Canvas;

pub fn draw(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_players_results(o, system, &WITH_POINTS, canvas)
}
