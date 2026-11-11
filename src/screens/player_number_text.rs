//! `screens::player_number_text` (0x46A359): the current player's title, as a text of the
//! car choice.
//!
//!
//! String 0x191 + the current player's number (`event.current_player`), a space, string 0x170, a NUL, into
//! the number text buffer. In six entry and draw lists (the car choice's). Refuses a text longer than its
//! place (0x100). The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::strings::append_string_by_id::append_string_by_id;

const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x191;
const SECOND_STRING: u32 = 0x170;

pub fn player_number_text(menus: &Menus) -> Result<(), Refused> {
    let n: u32 = menus.screens.event.current_player.get();
    let mut text = Vec::new();
    let mut emit = |bytes: &[u8]| text.extend_from_slice(bytes);
    append_string_by_id(FIRST_STRING.wrapping_add(n), &mut emit)?;
    text.push(b' ');
    let mut emit = |bytes: &[u8]| text.extend_from_slice(bytes);
    append_string_by_id(SECOND_STRING, &mut emit)?;
    // Converted: `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, TEXT_LEN)?;
    crate::screens::text_buffers_store::TEXT_BUFFERS
        .number_text
        .write_at(0, &text, "the number text")?;
    Ok(())
}
