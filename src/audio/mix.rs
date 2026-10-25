//! `audio::mix` (0x40BDC5): a turn of the game's mixer into the primary buffer.
//!
//! In:  nothing
//! Out: every register preserved (the carry of a failure is not read by its caller)
//!
//! `GetCurrentPosition` (+0x10) of `g_primaryBuffer` into `g_playCursor`; the bytes the
//! play cursor moved since the last turn (round the buffer's `g_primarySize`); nothing
//! for none; more than `g_mixMost` (40 ms) are cut to it and the write cursor put that far
//! ahead of the play cursor. Then `audio::mix_channels` for them, `Lock` (+0x2C) from
//! `g_writeCursor` into `g_lockedParts`, the write cursor moved on round the buffer,
//! `audio::write_mix`, `Unlock` (+0x4C). A failing call ends it with the carry, and
//! `DSERR_BUFFERLOST` sets `g_soundBufferLost`. (A `GetStatus` at 0x40BE9D is in it but not
//! reached.) One caller, the mixer's way of the turn.

use crate::audio::store::AUDIO;
use crate::errors::Refused;

const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

/// True for no failure.
pub fn mix() -> Result<bool, Refused> {
    unsafe {
        let primary = AUDIO.primary_buffer.get();
        let failed = |r: i32| {
            if r == BUFFER_LOST {
                AUDIO.sound_buffer_lost.set(1);
            }
            false
        };
        let old = AUDIO.play_cursor.get();
        let position: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x10));
        let r = position(primary, AUDIO.play_cursor.addr() as u32, 0);
        if r != 0 {
            return Ok(failed(r));
        }
        // Converted: `rc2k_core::audio::streams::cursor_moved` and `round`.
        use rc2k_core::audio::streams::{cursor_moved, round};
        let size = AUDIO.primary_size.get();
        let Some((bytes, write_at)) =
            cursor_moved(old, AUDIO.play_cursor.get(), size, AUDIO.mix_most.get())
        else {
            return Ok(true);
        };
        if let Some(at) = write_at {
            AUDIO.write_cursor.set(at);
        }
        crate::audio::mix_channels::mix_channels_in_game(bytes)?;
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x2C));
        let p = AUDIO.locked_parts[0].addr() as u32;
        let r = lock(
            primary,
            AUDIO.write_cursor.get(),
            bytes,
            p,
            p + 4,
            p + 8,
            p + 12,
            0,
        );
        if r != 0 {
            return Ok(failed(r));
        }
        AUDIO
            .write_cursor
            .set(round(bytes.wrapping_add(AUDIO.write_cursor.get()), size));
        crate::audio::write_mix::write_mix_in_game()?;
        let [p1, n1, p2, n2] = AUDIO.locked_parts.each_ref().map(|d| d.get());
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x4C));
        let r = unlock(primary, p1, n1, p2, n2);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}
