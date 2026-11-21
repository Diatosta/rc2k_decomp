//! `menu::flip_gearing`: `flow.shown_gearing` flipped (`xor 1`) and its text into
//! the title buffer: string 0x21F, or 0x221 when it is then 0, and a NUL.
//!
//! Unlike `menu::next_gearing` it does not touch the player's record. Reached from a menu table (a
//! left/right handler of an item of the car choice).

use crate::menu::owner::Screens;
use crate::strings::append_string_by_id::push_string;

pub fn flip_gearing(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    screens: &Screens,
    text_buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), crate::errors::BadInput> {
    use rc2k_core::menu::steps::{flip, gearing_switch_text};
    let kept = flip(screens.flow.read().shown_gearing);
    screens.flow.write().shown_gearing = kept;
    let mut text = Vec::new();
    push_string(so, &mut text, gearing_switch_text(kept))?;
    text.push(0);
    text_buffers.title.write_at(0, &text, "the title")
}
