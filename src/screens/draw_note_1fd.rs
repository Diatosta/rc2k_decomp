//! String 0x1FD drawn at (0x19A, 0x118). Only in the draw list of
//! `menu::unused_cspecs_loop`, which nothing reaches.

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

pub fn draw_note_1fd(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    // The logic is in `rc2k_core::screens::notes::NOTE_1FD`.
    crate::screens::game_screen::draw_note(o, canvas, &rc2k_core::screens::notes::NOTE_1FD)?;
    Ok(())
}
