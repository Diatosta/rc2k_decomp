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
pub unsafe fn put_string(id: u32, place: usize, room: usize) -> Result<(), Refused> {
    unsafe {
        let mut text = Vec::new();
        append_string_by_id(id, &mut |p, n| {
            text.extend_from_slice(std::slice::from_raw_parts(p, n))
        });
        // Converted: `rc2k_core::screens::captions::fit`.
        let text = rc2k_core::screens::captions::fit(text, room)?;
        std::ptr::copy_nonoverlapping(text.as_ptr(), place as *mut u8, text.len());
        Ok(())
    }
}

pub unsafe fn car_caption() -> Result<(), Refused> {
    unsafe {
        let id = read(shown_description()?).name;
        put_string(
            id,
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .caption
                .addr(),
            CAPTION_LEN,
        )
    }
}
