//! `menu::flip_gearing` (0x469916): `dword_7554F0` flipped (`xor 1`) and its text into
//! `byte_A8BF54`: string 0x21F, or 0x221 when it is then 0, and a NUL.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`; the original loads eax just before the
//! `popa`)
//!
//! Unlike `menu::next_gearing` it does not touch the player's record. Reached from a menu table
//! (0x74BA9C, a left/right handler of an item of the car choice). The menus and draw lists run its
//! `entry` (`menu::table_calls`); the registers above are the original's, for reference.

use crate::strings::append_string_by_id::push_string;

/// The Rust version (see the header).
pub fn flip_gearing() -> Result<(), crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::steps` (`flip`, `gearing_switch_text`).
    use rc2k_core::menu::steps::{flip, gearing_switch_text};
    let kept = flip(crate::menu::owner::menus().screens.flow.shown_gearing.get());
    crate::menu::owner::menus()
        .screens
        .flow
        .shown_gearing
        .set(kept);
    let mut text = Vec::new();
    push_string(&mut text, gearing_switch_text(kept))?;
    text.push(0);
    crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.title)
        .write_bytes(&text, "the title")
}
