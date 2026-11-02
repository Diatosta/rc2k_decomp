//! `screens::car_caption` (0x469A0E): the car shown's name as the caption.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The shown car's name (`CarDescription::name`, the entry `dword_755990` through `dword_75A65C`)
//! and a NUL into `g_captionText`. In four draw lists (the car specification screens). Refuses an
//! entry past the records, one without its record, and a caption longer than its place. The menus
//! and draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

use crate::errors::Refused;
use crate::menu::car_description::{read, shown_description};
use crate::strings::append_string_by_id::append_string_by_id;

// ---- game data ----

// dword_755990
const CAPTION_LEN: usize = 0x100;

// ---- Rust version ----

/// A string and its NUL into a place of the game's, if it fits.
pub fn put_string(id: u32, place: crate::pointers::GamePtr, room: usize) -> Result<(), Refused> {
    let mut text = Vec::new();
    append_string_by_id(id, &mut |bytes| text.extend_from_slice(bytes))?;
    // Converted: `rc2k_core::screens::captions::fit`.
    let text = rc2k_core::screens::captions::fit(text, room)?;
    place.write_bytes(&text, "a caption")?;
    Ok(())
}

pub fn car_caption() -> Result<(), Refused> {
    let id = read(shown_description()?)?.name;
    put_string(
        id,
        crate::pointers::GamePtr::to(&crate::screens::text_buffers_store::TEXT_BUFFERS.caption),
        CAPTION_LEN,
    )
}
