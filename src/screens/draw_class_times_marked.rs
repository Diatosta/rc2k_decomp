//! The class `arcade.level`'s times, the player's own in the highlight fonts, the sprites 4
//! lower (see `screens/best_scores.rs`). Refuses no table.

use crate::errors::Refused;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

pub fn draw(o: crate::owners::Owners<'static>, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    draw_scores(o, &MARKED_CLASS_TIMES, canvas)
}
