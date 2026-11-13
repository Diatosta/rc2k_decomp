//! Run-length unpacking from a packed buffer into another: a signed count byte; below 0, that
//! many bytes copied; above 0, the next byte repeated count + 1 times; 0 ends it. Refuses
//! (before anything) more than 0x100000 bytes out (a guard against a broken frame) or more than
//! the block holds.

use crate::errors::Refused;
use crate::race::viewer_buffers_store::ViewerBuffers;
use rc2k_core::drive::rle;

/// From the packed bytes into the replay block: the bytes written. Bytes past the block are refused
/// before anything.
pub fn unpack_rle(buffers: &ViewerBuffers, packed: &[u8]) -> Result<u32, Refused> {
    // The bytes come in order; they are put in one write (the core checks the whole first).
    let mut out: Vec<u8> = Vec::new();
    let written = rle::unpack_rle(packed, |k, b| {
        if out.len() <= k as usize {
            out.resize(k as usize + 1, 0);
        }
        out[k as usize] = b;
    })?;
    buffers.put_in_block(0, &out, "the unpacked replay bytes")?;
    Ok(written)
}
