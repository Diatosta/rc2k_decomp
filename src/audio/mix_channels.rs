//! `audio::mix_channels` (0x40BECC): the used channels mixed into the work buffer.
//!
//! In:  eax = the primary buffer's bytes to fill
//! Out: eax = the same (ebx the frames, ecx 0, edx, esi, edi, ebp the loops' leftovers:
//!      the one caller, `audio::mix`, keeps its own around it)
//!
//! See `audio/mixer.rs`.

use crate::audio::mixer::*;
use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::store::game_addr;

pub fn mix_channels_in_game(bytes: u32) -> Result<(), Refused> {
    unsafe {
        mix_channels(
            bytes,
            AUDIO.mix_shift.get(),
            game_addr(MIX_BUFFER_RVA) as u32,
            AUDIO.channels[0].addr() as u32,
            AUDIO.channel_count.get(),
        )
    }
}
