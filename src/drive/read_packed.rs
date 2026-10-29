//! `drive::read_packed` (0x41EEB5): ecx bytes from the file eax into ebx: a count of 0 or
//! more is a plain read (its `jns` goes straight to `files::read`); below 0, -ecx packed
//! bytes read into `g_pngInfo` and unpacked into ebx (`drive::unpack_rle`), eax the bytes
//! unpacked.
//!
//! In:  eax the file, ebx the buffer, ecx the count (below 0: packed)
//! Out: eax the bytes read or unpacked, CF set when the read failed (ZF from `test eax`
//!      after an unpacking); a plain read keeps ebx, ecx, edx, a packed one every register
//!      but eax
//!
//! Caller: `drive::read_replay_frame`. Refuses what `unpack_rle` refuses (nothing written
//! then but the packed bytes).

use crate::errors::{BadInput, Refused};
use crate::files::handle::read_slice;
use crate::files::scratch::SCRATCH;
use crate::race::viewer_buffers_store::{BLOCK_BYTES, VIEWER_BUFFERS};

/// The Rust version, into the replay block (`dword_9613A4`, the ebx of both callers):
/// (eax, CF, ZF when it unpacked). A plain read of more than the block holds is refused
/// before anything (the original reads on past it), as is a packed one past the scratch.
// Converted: `rc2k_core::drive::race_steps::packed_bytes` (the unpacking:
// `rc2k_core::drive::rle`).
pub fn read_packed(file: u32, count: u32) -> Result<(u32, bool, Option<bool>), Refused> {
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        if count as usize > BLOCK_BYTES {
            return Err(BadInput("a replay block read past the block").into());
        }
        let mut bytes = vec![0u8; count as usize];
        let (n, ok) = read_slice(file, &mut bytes);
        VIEWER_BUFFERS.put_in_block(0, &bytes[..n as usize], "a replay block read")?;
        return Ok((n, !ok, None));
    };
    let mut bytes = vec![0u8; packed as usize];
    if bytes.len() > crate::files::scratch::Scratch::LEN {
        return Err(BadInput("a packed replay block past the scratch").into());
    }
    let (n, ok) = read_slice(file, &mut bytes);
    SCRATCH
        .buffer
        .write_at(0, &bytes[..n as usize], "a packed replay block")?;
    if !ok {
        return Ok((n, true, None));
    }
    let out = crate::drive::unpack_rle::unpack_rle(&SCRATCH.buffer.read())?;
    Ok((out, false, Some(out == 0)))
}
