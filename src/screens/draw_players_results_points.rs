//! `screens::draw_players_results_points` (0x4836CF): the players' stage results with their points: heads, the stage, and per player (three rows when `dword_755984` is 3, else four: two players get four) name, top speed, car, time or the gap to the first, flag, place and points (see `screens/players_results.rs`, `WITH_POINTS`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E8E). The sprites' palettes are ebx as the original leaves
//! it. Refuses (before anything) a row's offset past the four, a row's player without its
//! record or name or points, no stage records, a stage line past the line buffer, and a speed
//! `screens::draw_spaced_number` refuses. Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::screens::players_results::{WITH_POINTS, draw_players_results};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_players_results(&WITH_POINTS, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_players_results_points", r, &rust)
}
