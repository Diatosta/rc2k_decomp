//! `audio::mix_channels` (0x40BECC): the used channels mixed into the work buffer.
//!
//! In:  eax = the primary buffer's bytes to fill
//! Out: eax = the same (ebx the frames, ecx 0, edx, esi, edi, ebp the loops' leftovers:
//!      the one caller, `audio::mix`, keeps its own around it)
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
