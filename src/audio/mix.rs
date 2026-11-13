//! A turn of the game's mixer into the primary buffer: the bytes the play cursor moved since the
//! last turn (at most 40 ms; more are cut and the write cursor put that far ahead) are mixed
//! (`audio::mix_channels`), the buffer locked at the write cursor, filled (`audio::write_mix`) and
//! unlocked. A failing call ends it with failure, and a lost buffer sets the lost flag.

use crate::audio::own::AudioOwn;
use crate::errors::Refused;
use crate::session::PlainDword;

const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

/// True for no failure.
pub fn mix(state: &AudioOwn) -> Result<bool, Refused> {
    use crate::platform_sound as ds;
    {
        let primary = state.primary_buffer.get();
        let failed = |r: i32| {
            if r == BUFFER_LOST {
                state.sound_buffer_lost.set(1);
            }
            false
        };
        let old = state.play_cursor.get();
        match ds::current_position(primary) {
            Ok((play, _)) => state.play_cursor.set(play),
            Err(r) => return Ok(failed(r)),
        }
        use rc2k_core::audio::streams::{cursor_moved, round};
        let size = state.primary_size.get();
        let Some((bytes, write_at)) =
            cursor_moved(old, state.play_cursor.get(), size, state.mix_most.get())
        else {
            return Ok(true);
        };
        if let Some(at) = write_at {
            state.write_cursor.set(at);
        }
        crate::audio::mix_channels::mix_channels_in_game(state, bytes)?;
        match ds::lock(primary, state.write_cursor.get(), bytes, 0) {
            Ok(parts) => {
                for (cell, part) in state.locked_parts.iter().zip(parts) {
                    cell.set(part);
                }
            }
            Err(r) => return Ok(failed(r)),
        }
        state
            .write_cursor
            .set(round(bytes.wrapping_add(state.write_cursor.get()), size));
        crate::audio::write_mix::write_mix_in_game(state)?;
        let r = ds::unlock(primary, state.locked_parts.each_ref().map(PlainDword::get));
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}
