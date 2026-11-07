//! `settings::read_text` (0x41155A): reads a settings file into the scratch block.
//!
//! Takes the path. Returns the text when it was read. `files::read_file` of up to 0xFFFF bytes to
//! 0x93E8B0, and a NUL after what was read. Five callers: `var\ral.cfg`, `var\ral.zog`, `lang.cfg`,
//! `files\d\gfx\sku` and one more.

use crate::errors::BadInput;
use crate::files::read_file::read_file_to_store;
use crate::pointers::GamePtr;
use crate::settings::TEXT_ROOM;
use crate::system::owner::system;

// ---- Rust version ----

/// The text (NUL-terminated) and its length, or None if the file was not read.
/// Refuses what `read_file` refuses.
pub fn read_text(path: &[u8]) -> Result<Option<(GamePtr, u32)>, BadInput> {
    let system = system();
    {
        let text = GamePtr::to(&system.work.files.buffer);
        let read = read_file_to_store(path, &system.work.files.buffer, TEXT_ROOM, 0)?;
        // Converted: `rc2k_core::files::settings::text_read`.
        let Some(bytes) = rc2k_core::files::settings::text_read(read.ok, read.bytes_read) else {
            return Ok(None);
        };
        // The text up to its NUL.
        text.offset(bytes).write(0u8, "the settings text's end")?;
        Ok(Some((text, bytes)))
    }
}
