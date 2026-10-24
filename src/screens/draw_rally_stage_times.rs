//! `screens::draw_rally_stage_times` (0x48A948): a rally stage's best times: the heading, then up to ten rows of name, time, rank, car and flag (the sprites 4 lower) (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D22). Refuses (before anything) no stage records or no times
//! table. The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are
//! the original's, for reference.

use crate::errors::Refused;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

/// The Rust version (see the header).
pub unsafe fn draw(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe { draw_stage_times(&RALLY_STAGE_TIMES, canvas) }
}
