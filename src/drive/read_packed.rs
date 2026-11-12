//! `drive::read_packed` (0x41EEB5): reads bytes from a file into a buffer: a count of 0 or more is
//! a plain read (`files::read`); below 0, minus the count of packed bytes are read into
//! `g_pngInfo` and unpacked into the buffer (`drive::unpack_rle`), returning the bytes unpacked.
//!
//! Takes: the file, the buffer, the count (below 0: packed)
//!
//! Caller: `drive::read_replay_frame`. Refuses what `unpack_rle` refuses (nothing written
//! then but the packed bytes).

use crate::errors::{BadInput, Refused};
use crate::files::handle::read_slice;
use crate::race::owner::Race;
use crate::race::viewer_buffers_store::BLOCK_BYTES;

/// Into the replay block:. A plain read of more than the block holds is refused before anything
/// (the original reads on past it), as is a packed one past the scratch.
// Converted: `rc2k_core::drive::race_steps::packed_bytes` (the unpacking:
// `rc2k_core::drive::rle`).
pub fn read_packed(
    race: &Race,
    file: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    let system = crate::system::owner::system();
    let work = &system.work;
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        if count as usize > BLOCK_BYTES {
            return Err(BadInput("a replay block read past the block").into());
        }
        let mut bytes = vec![0u8; count as usize];
        let (n, ok) = read_slice(file, &mut bytes);
        race.viewer_buffers()
            .put_in_block(0, &bytes[..n as usize], "a replay block read")?;
        return Ok((n, !ok, None));
    };
    let (n, ok) = work
        .files
        .read_from(file, packed, "a packed replay block")?;
    if !ok {
        return Ok((n, true, None));
    }
    let out =
        crate::drive::unpack_rle::unpack_rle(race.viewer_buffers(), &work.files.buffer.read())?;
    Ok((out, false, Some(out == 0)))
}
