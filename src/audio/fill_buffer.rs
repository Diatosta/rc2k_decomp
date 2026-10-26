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
use crate::audio::store::AUDIO;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

/// The voice's buffer filled; the lost buffer's flag is `g_soundBufferLost`, the parts go
/// to `g_lockedParts`. True for the carry clear.
pub unsafe fn fill_buffer(v: &Voice) -> Result<bool, Refused> {
    let parts = AUDIO.locked_parts[0].addr() as u32;
    if a3d() {
        return Err(BadInput("A3D (the chunk at 0x40CDE8)").into());
    }
    // Converted: `rc2k_core::audio::buffers` (`fills`, `fill_copies`, `lost`).
    use rc2k_core::audio::buffers::{fill_copies, fills, lost as lost_memory};
    if !fills(v) {
        return Ok(true);
    }
    unsafe {
        let failed = |r: i32| {
            if lost_memory(r) {
                AUDIO.sound_buffer_lost.set(1);
            }
            false
        };
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer, 0x2C));
        let r = lock(
            v.buffer,
            0,
            v.length,
            parts,
            parts + 4,
            parts + 8,
            parts + 12,
            0,
        );
        if r != 0 {
            return Ok(failed(r));
        }
        let locked = AUDIO.locked_parts.each_ref().map(|d| d.get());
        let [p1, n1, p2, n2] = locked;
        let (first, second) = fill_copies(v.data, locked);
        for (from, to, bytes) in std::iter::once(first).chain(second) {
            crate::pointers::GamePtr::new(from).copy_to(
                crate::pointers::GamePtr::new(to),
                bytes as usize,
                "a voice's sound into its buffer",
            )?;
        }
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(v.buffer, 0x4C));
        let r = unlock(v.buffer, p1, n1, p2, n2);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}
