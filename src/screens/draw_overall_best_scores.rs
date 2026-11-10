//! `screens::draw_overall_best_scores` (0x4A8FA2): the overall best scores (group 4) with its heading
//! (the section's name in event mode 0x13, else string 0x362), the player's own scores in
//! the highlight fonts (see `screens/best_scores.rs`).
//!
//!
//! In a menu loop's draw list (0x758D42). Refuses (before anything) no table. The menus and draw
//! lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

pub fn draw(menus: &Menus, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    draw_scores_with_heading(menus, &OVERALL_BEST_SCORES, false, canvas)
}
