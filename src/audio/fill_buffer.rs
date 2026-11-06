//! `audio::fill_buffer` (0x40B82B): a voice's sample data copied into its buffer.
//!
//! In:  esi = the voice
//! Out: carry = failed; every register preserved
//!
//! Only for a voice with a buffer of its own (none shared): `Lock` (+0x2C) from 0 for its
//! length into `g_lockedParts` (0x506A98: two pointers and their sizes), the data copied
//! into the first part and, if there is one, on into the second, `Unlock` (+0x4C) with
//! them. A failing `Lock` or `Unlock` sets the carry, and with `DSERR_BUFFERLOST`
//! (0x88780096) `g_soundBufferLost`. Callers: command 3 and `sub_40A7C9` (the lost
//! buffers' restoring).
//! Refuses A3D (the chunk at 0x40CDE8).

use crate::audio::buffer_settings::a3d;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// The voice's buffer filled; the lost buffer's flag is `g_soundBufferLost`, the parts go
/// to `g_lockedParts`. True for the carry clear.
pub fn fill_buffer(v: &Voice) -> Result<bool, Refused> {
    use crate::platform_sound as ds;
    if a3d() {
        return Err(BadInput("A3D (the chunk at 0x40CDE8)").into());
    }
    // Converted: `rc2k_core::audio::buffers` (`fills`, `fill_copies`, `lost`).
    use rc2k_core::audio::buffers::{fill_copies, fills, lost as lost_memory};
    if !fills(v) {
        return Ok(true);
    }
    {
        let failed = |r: i32| {
            if lost_memory(r) {
                crate::audio::owner::audio().state.sound_buffer_lost.set(1);
            }
            false
        };
        let locked = match ds::lock(v.buffer, 0, v.length, 0) {
            Ok(parts) => parts,
            Err(r) => return Ok(failed(r)),
        };
        for (cell, part) in crate::audio::owner::audio()
            .state
            .locked_parts
            .iter()
            .zip(locked)
        {
            cell.set(part);
        }
        // The copies from the data's start (an id: `audio::sample_data`) into the parts.
        let (first, second) = fill_copies(0, locked);
        let copies: Vec<Vec<u8>> = std::iter::once(first)
            .chain(second)
            .map(|(from, _, bytes)| {
                crate::audio::sample_data::with_data(v.data, |data| {
                    data.get(from as usize..(from + bytes) as usize)
                        .map(<[u8]>::to_vec)
                        .ok_or(crate::errors::BadInput("a voice's sound past its memory"))
                })?
            })
            .collect::<Result<_, _>>()?;
        ds::with_parts(v.buffer, locked, |a, b| {
            a.copy_from_slice(&copies[0]);
            if let Some(c) = copies.get(1) {
                b.copy_from_slice(c);
            }
        })
        .map_err(|_| crate::errors::BadInput("a locked part that is not the buffer's"))?;
        let r = ds::unlock(v.buffer, locked);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}
