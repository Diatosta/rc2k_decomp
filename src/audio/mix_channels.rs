//! `audio::mix_channels` (0x40BECC): the used channels mixed into the work buffer.
//!
//! In:  eax = the primary buffer's bytes to fill
//! Out: eax = the same (ebx the frames, ecx 0, edx, esi, edi, ebp the loops' leftovers:
//!      the one caller, `audio::mix`, keeps its own around it)
//!
//! See `audio/mixer.rs`. Compared in shadow mode (the work buffer and the channels).
//! Self-test: a redirected copy whose shift, buffer and channels are the test's, 120
//! channels' states by lot.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::mixer::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{game_addr, game_read};

pub unsafe fn mix_channels_in_game(bytes: u32) -> Result<(), Refused> {
    let w = World::game();
    unsafe {
        mix_channels(
            bytes,
            game_read(MIX_SHIFT_RVA),
            game_addr(MIX_BUFFER_RVA) as u32,
            w.channels,
            read(w.channel_count),
        )
    }
}
