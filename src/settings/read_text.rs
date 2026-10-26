//! `settings::read_text` (0x41155A): reads a settings file into the scratch block.
//!
//! In:  eax = path
//! Out: read: esi = the text, carry clear (zero set); not read: carry set, esi unchanged;
//!      everything else preserved
//! `files::read_file` of up to 0xFFFF bytes to 0x93E8B0, and a NUL after what was read.
//! Five callers: `var\ral.cfg`, `var\ral.zog`, `lang.cfg`, `files\d\gfx\sku` and one more.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::read_file::read_file;
use crate::pointers::GamePtr;
use crate::settings::TEXT_ROOM;

// ---- Rust version ----

/// The text (NUL-terminated) and its length, or None if the file was not read.
/// Refuses what `read_file` refuses.
pub fn read_text(path: *const u8) -> Result<Option<(*const u8, u32)>, BadInput> {
    {
        let text = crate::pointers::host_at(crate::files::scratch::SCRATCH.buffer.addr());
        let read = read_file(path, text, TEXT_ROOM, 0)?;
        // Converted: `rc2k_core::files::settings::text_read`.
        let Some(bytes) = rc2k_core::files::settings::text_read(read.ok, read.bytes_read) else {
            return Ok(None);
        };
        GamePtr::new(crate::pointers::game_address(text))
            .offset(bytes)
            .write(0u8, "the settings text's end")?;
        Ok(Some((text, bytes)))
    }
}
