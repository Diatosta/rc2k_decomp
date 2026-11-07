//! `files::write_stream_block` (0x402DC9): ecx bytes from ebx into a buffered stream, one
//! by one (`files::write_stream_byte`), until one fails.
//!
//! In:  esi the stream, ebx the bytes, ecx their count
//! Out: eax the bytes written; carry set when one failed; else carry clear and ZF of
//!      `test eax, eax` with al the last byte (ZF set for a count of 0); ebx, ecx, edx kept
//!
//! Callers: `gfx::write_screen_pcx`, 0x403D27, 0x403D89, 0x403D9B.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::files::stream::Stream;

/// The Rust version: the bytes written, and the original's two flags.
pub fn write_stream_block(
    stream: &mut Stream,
    bytes: &[u8],
    eax: u32,
) -> Result<(u32, bool, bool), BadInput> {
    Ok(rc2k_core::files::io::write_stream_block(
        &mut crate::files::handle::GameFiles::new(),
        stream,
        bytes,
        eax,
    )?)
}

/// All of `bytes` written: whether it worked.
pub fn write_stream_all(stream: &mut Stream, bytes: &[u8]) -> Result<bool, BadInput> {
    Ok(!write_stream_block(stream, bytes, 0)?.1)
}
