//! `drive::unpack_rle` (0x41F080): run-length unpacking from esi to edi: a signed count
//! byte; below 0, that many bytes copied; above 0, the next byte repeated count + 1
//! times; 0 ends it.
//!
//! In:  esi the packed bytes, edi where they go
//! Out: ecx the bytes written; eax, esi, edi kept (pushed); no flags read
//!
//! Caller: `drive::read_packed`. Refuses (before anything) more than 0x100000 bytes out
//! (a guard against a broken frame).

use crate::errors::Refused;
use crate::pointers::GamePtr;
use rc2k_core::drive::rle;

/// The Rust version: the bytes written.
// Converted: `rc2k_core::drive::rle::unpack_rle`.
pub fn unpack_rle(from: u32, to: u32) -> Result<u32, Refused> {
    let packed = GamePtr::new(from).bytes("packed replay bytes")?;
    // The bytes come in order; they are put in one write (the core checks the whole first).
    let mut out: Vec<u8> = Vec::new();
    let written = rle::unpack_rle(&packed, |k, b| {
        if out.len() <= k as usize {
            out.resize(k as usize + 1, 0);
        }
        out[k as usize] = b;
    })?;
    if !out.is_empty() {
        GamePtr::new(to).write_bytes(&out, "the unpacked replay bytes")?;
    }
    Ok(written)
}
