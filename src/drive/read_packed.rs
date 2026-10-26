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

use crate::errors::Refused;
use crate::files::handle::read;
use crate::files::scratch::SCRATCH;

/// The Rust version: (eax, CF, ZF when it unpacked).
// Converted: `rc2k_core::drive::race_steps::packed_bytes` (the unpacking:
// `rc2k_core::drive::rle`).
pub fn read_packed(
    file: u32,
    buffer: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        let (n, ok) = { read(file, crate::pointers::host(buffer), count) };
        return Ok((n, !ok, None));
    };
    let (n, ok) = { read(file, crate::pointers::host(SCRATCH.buffer.addr()), packed) };
    if !ok {
        return Ok((n, true, None));
    }
    let out = crate::drive::unpack_rle::unpack_rle(SCRATCH.buffer.addr(), buffer)?;
    Ok((out, false, Some(out == 0)))
}
