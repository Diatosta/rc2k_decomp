//! `screens::draw_championship_table` (0x4A4E1E): the championship standings: heads, the six
//! rounds' flags and ten rows of position, name, points, total, car and flag (see
//! `screens/championship_table.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758E56). Refuses (before anything) no history table; a
//! position not in it is the original's fatal error 0x8070A7FA. Hook removed 2026-10-22 (typed
//! tables): the menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::championship_table::draw_championship_table;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_championship_table(canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_championship_table", r, &rust)
}
