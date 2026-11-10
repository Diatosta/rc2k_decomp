//! `screens::draw_players_results` (0x48D32E): the players' stage results: heads, the stage, and per player (two, three or four rows by `dword_755984`) name, top speed, car, time or the gap to the first, flag (and `dword_7559B4`'s sprite on the third row) and place, 0x1E to the right of the points page's columns (see `screens/players_results.rs`, `WITHOUT_POINTS`).
//!
//!
//! In a menu loop's draw list (0x758E96). The sprites' palettes are ebx as the original leaves it.
//! Refuses (before anything) a row's offset past the four, a row's player without its record or
//! name, no stage records, a stage line past the line buffer, and a speed
//! `screens::draw_spaced_number` refuses. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::screens::players_results::{WITHOUT_POINTS, draw_players_results};
use crate::text::draw_text::Canvas;

pub fn draw(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    draw_players_results(menus, &WITHOUT_POINTS, canvas)
}
