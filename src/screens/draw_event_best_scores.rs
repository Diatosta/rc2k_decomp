//! A best scores page (group `event.class`) with its heading (the section's name in event
//! mode 0x13, else string 0x35E + the group) and the player's own scores in the highlight fonts
//! (see `screens/best_scores.rs`). Refuses no table.

use crate::errors::Refused;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

pub fn draw(o: crate::owners::Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    draw_scores_with_heading(o, &EVENT_BEST_SCORES, true, canvas)
}
