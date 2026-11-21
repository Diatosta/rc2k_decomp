//! The current player's title, as a text of the car choice: string 0x191, the player's
//! number, a space and string 0x170. Refuses a text longer than its place (0x100).

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::strings::append_string_by_id::append_string_by_id;

const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x191;
const SECOND_STRING: u32 = 0x170;

pub fn player_number_text(
    so: crate::strings::append_string_by_id::StringOwners<'_>,
    menus: &Menus,
    buffers: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let n: u32 = menus.screens.event.read().current_player;
    let mut text = Vec::new();
    let mut emit = |bytes: &[u8]| text.extend_from_slice(bytes);
    append_string_by_id(so, FIRST_STRING.wrapping_add(n), &mut emit)?;
    text.push(b' ');
    let mut emit = |bytes: &[u8]| text.extend_from_slice(bytes);
    append_string_by_id(so, SECOND_STRING, &mut emit)?;
    // The logic is in `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, TEXT_LEN)?;
    buffers.number_text.write_at(0, &text, "the number text")?;
    Ok(())
}
