//! `screens::draw_class_times_marked` (0x48AB87): the class `dword_7587EC`'s times, the
//! player's own (+0x40 1) in the highlight fonts, the sprites 4 lower (see
//! `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x758D32). Refuses (before anything) no table. The menus and draw
//! lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

/// The Rust version (see the header).
pub fn draw(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    draw_scores(&MARKED_CLASS_TIMES, canvas)
}
