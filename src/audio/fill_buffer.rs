//! Copies a voice's sample data into its buffer: only for a voice with a buffer of its own (none
//! shared). The buffer is locked, the data copied into the locked parts and unlocked; a failing
//! lock or unlock fails, and a lost buffer sets the lost flag. Refuses A3D.

use crate::audio::buffer_settings::a3d;
use crate::audio::own::AudioOwn;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// Fills the voice's buffer. True when it worked.
pub fn fill_buffer(state: &AudioOwn, v: &Voice) -> Result<bool, Refused> {
    use crate::platform_sound as ds;
    if a3d(state) {
        return Err(BadInput("A3D (the chunk at 0x40CDE8)").into());
    }
    use rc2k_core::audio::buffers::{fill_copies, fills, lost as lost_memory};
    if !fills(v) {
        return Ok(true);
    }
    {
        let failed = |r: i32| {
            if lost_memory(r) {
                state.sound_buffer_lost.set(1);
            }
            false
        };
        let locked = match ds::lock(v.buffer, 0, v.length, 0) {
            Ok(parts) => parts,
            Err(r) => return Ok(failed(r)),
        };
        for (cell, part) in state.locked_parts.iter().zip(locked) {
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
