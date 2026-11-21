//! `menu::clear_choice_marks`: the four choice marks set to -1.
//!
//! Four dwords, one a player by the look of it (their meaning is not read). In two entry lists (the
//! car choice's).

use crate::menu::owner::Screens;

pub(super) fn clear_choice_marks(screens: &Screens) -> Result<(), crate::errors::BadInput> {
    screens.carousel.write().pictures.held = [u32::MAX; 4];
    Ok(())
}
