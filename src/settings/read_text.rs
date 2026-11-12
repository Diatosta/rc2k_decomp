//! `settings::read_text` (0x41155A): reads a settings file into the scratch block.
//!
//! Takes the path. Returns the text when it was read. `files::read_file` of up to 0xFFFF bytes to
//! 0x93E8B0, and a NUL after what was read. Five callers: `var\ral.cfg`, `var\ral.zog`, `lang.cfg`,
//! `files\d\gfx\sku` and one more.

use crate::errors::BadInput;
use crate::files::read_file::read_file_to_store;
use crate::settings::TEXT_ROOM;
use crate::system::owner::System;

/// The scratch block with the text (NUL-terminated) at its start, or None if the file was
/// not read. Refuses what `read_file` refuses.
pub fn read_text(system: &System, path: &[u8]) -> Result<Option<Vec<u8>>, BadInput> {
    {
        let read = read_file_to_store(path, &system.work.files.buffer, TEXT_ROOM, 0)?;
        // Converted: `rc2k_core::files::settings::text_read`.
        let Some(bytes) = rc2k_core::files::settings::text_read(read.ok, read.bytes_read) else {
            return Ok(None);
        };
        // The text up to its NUL.
        let buffer = &system.work.files.buffer;
        buffer.write_at(bytes as usize, &[0u8], "the settings text's end")?;
        Ok(Some(buffer.read().to_vec()))
    }
}
