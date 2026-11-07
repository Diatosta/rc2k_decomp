//! `audio::mix_channels` (0x40BECC): the used channels mixed into the work buffer.
//!
//! Takes: the primary buffer's bytes to fill
//!
//! See `audio/mixer.rs`.

use crate::audio::mixer::*;
use crate::audio::store::AudioState;
use crate::errors::Refused;

pub fn mix_channels_in_game(state: &AudioState, bytes: u32) -> Result<(), Refused> {
    mix_channels(
        state,
        bytes,
        state.mix_shift.get(),
        // The first channel's id.
        1,
        state.channel_count.get(),
    )
}
