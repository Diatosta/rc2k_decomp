//! `menu::clear_choice_marks` (0x490346): the four choice marks set to -1.
//!
//! Four dwords, one a player by the look of it (their meaning is not read). In two entry lists
//! (0x759406, 0x759412: the car choice's). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::menu::owner::Screens;

pub fn clear_choice_marks(screens: &Screens) -> Result<(), crate::errors::BadInput> {
    for held in &screens.state.carousel_pictures.held {
        held.set(u32::MAX);
    }
    Ok(())
}
