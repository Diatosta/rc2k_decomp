//! A rally stage's best times: the heading, then up to ten rows of name, time, rank, car and flag,
//! the sprites 4 lower (see `screens/best_scores.rs`). Refuses no stage records or no times table.

use crate::errors::Refused;
use crate::screens::best_scores::*;
use crate::text::draw_text::Canvas;

pub fn draw(
    o: crate::owners::Owners<'static>,
    system: &crate::system::owner::System,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    draw_stage_times(o, system, &RALLY_STAGE_TIMES, canvas)
}
