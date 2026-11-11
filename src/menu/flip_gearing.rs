//! `menu::flip_gearing` (0x469916): `flow.shown_gearing` flipped (`xor 1`) and its text into
//! the title buffer: string 0x21F, or 0x221 when it is then 0, and a NUL.
//!
//! Unlike `menu::next_gearing` it does not touch the player's record. Reached from a menu table
//! (0x74BA9C, a left/right handler of an item of the car choice). The menus and draw lists run its
//! `entry` (`menu::table_calls`).

use crate::menu::owner::Screens;
use crate::strings::append_string_by_id::push_string;

pub fn flip_gearing(screens: &Screens) -> Result<(), crate::errors::BadInput> {
    // The logic is in `rc2k_core::menu::steps` (`flip`, `gearing_switch_text`).
    use rc2k_core::menu::steps::{flip, gearing_switch_text};
    let kept = flip(screens.flow.shown_gearing.get());
    screens.flow.shown_gearing.set(kept);
    let mut text = Vec::new();
    push_string(&mut text, gearing_switch_text(kept))?;
    text.push(0);
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .title
        .write_at(0, &text, "the title")
}
