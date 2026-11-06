//! `sound::stream_playing` (0x40C4D5): the music stream's playing flag (`dword_506960`,
//! `g_soundStream.playing`) compared with 1: zero set when it plays, carry when it is 0.
//!
//! In:  nothing
//! Out: the flags of `cmp dword_506960, 1`; every register kept
//!
//! Callers: `sub_46F7AD` (`jz`), `sub_47F314` (`jnz`); `sound::step_car_stream` has it
//! inlined.

use crate::audio::owner::audio;

/// The Rust version: (zero, carry) as the original's `cmp` leaves them.
pub fn stream_playing() -> (bool, bool) {
    let playing = audio().state.sound_stream.get().playing;
    (playing == 1, playing < 1)
}
