//! `screens::car_caption` (0x469A0E): the car shown's name as the caption.
//!
//!
//! The shown car's name (`CarDescription::name`, the entry `event.shown_car` through the carousel's car list)
//! and a NUL into `g_captionText`. In four draw lists (the car specification screens). Refuses an
//! entry past the records, one without its record, and a caption longer than its place. The menus
//! and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::car_description::{read, shown_description};
use crate::menu::owner::Menus;
use crate::strings::append_string_by_id::{StringOwners, append_string_by_id};

// ---- game data ----

// `event.shown_car`
const CAPTION_LEN: usize = 0x100;

/// A string and its NUL into a text buffer of the game's, if it fits.
pub fn put_string(
    o: StringOwners<'_>,
    id: u32,
    buffer: &crate::session::Bytes<0x100>,
    room: usize,
) -> Result<(), Refused> {
    let mut text = Vec::new();
    append_string_by_id(o, id, &mut |bytes| text.extend_from_slice(bytes))?;
    // The logic is in `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, room)?;
    buffer.write_at(0, &text, "a caption")?;
    Ok(())
}

pub fn car_caption(
    o: StringOwners<'_>,
    menus: &Menus,
    text: &crate::screens::text_buffers_store::TextBuffers,
) -> Result<(), Refused> {
    let id = read(shown_description(menus)?)?.name;
    put_string(o, id, &text.caption, CAPTION_LEN)
}
