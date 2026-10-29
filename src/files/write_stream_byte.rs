//! `files::write_stream_byte` (0x402D53): a byte into a buffered stream opened to write
//! (the block `files/stream.rs` describes): a full buffer (position 0x2000 or more) first
//! written out (`files::write`, the bytes in it) and emptied; the byte at the position,
//! position and count one more.
//!
//! In:  esi the stream, al the byte
//! Out: carry set when the write failed (or was short; the byte not stored); else carry
//!      clear and ZF of `test eax, eax` (the whole of eax); every register kept
//!
//! Callers: `files::write_stream_block`, `gfx::write_screen_pcx` (six).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// The Rust version: whether it worked (carry clear). `eax` the register (al the byte).
pub unsafe fn write_stream_byte(stream: *mut Stream, eax: u32) -> Result<bool, BadInput> {
    // Converted: `rc2k_core::files::io::write_stream_byte`.
    unsafe {
        Ok(rc2k_core::files::io::write_stream_byte(
            &mut crate::files::handle::GameFiles::<crate::pointers::GamePtr>::new(),
            crate::files::stream::core_stream(stream),
            eax as u8,
        )?)
    }
}
