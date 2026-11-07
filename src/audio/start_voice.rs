//! `audio::start_voice` (0x40A9D1): command 1 on the audio thread: a sound started,
//! changed or stopped.
//!
//! In:  eax = the sound's number (the sender's ax, sign-extended), esi = the command's
//!      record (`sound/queue.rs`: +4 eax, +8 volume, +0xC pan, +0x10 frequency, +0x14 ..
//!      +0x1C the place)
//!
//! See `audio/voices.rs`. Nothing for a negative number, a sound not loaded, or one whose
//! first voice is not. With a volume and a voice of kind 0 (or, for another kind or no
//! volume, a voice not playing, unless the sender's eax has bit 31): the first of its
//! copies that is free gets a channel, a free one while `g_channelsUsed` is under
//! `g_channelCount` (then counted), else the channel of the playing voice of the lowest
//! priority below its own (signed; none: nothing), which goes to the stopping list; the
//! voice goes to the front of the playing list, state 1, and its channel names it. A
//! playing voice with no volume is stopped: its channel freed and counted down, the voice
//! to the stopping list. Then, for a voice started or playing: the volume (`DEFAULT`: the
//! record's; held to 0x40, unsigned), the pan (its low byte, signed, held to -0x40 ..
//! 0x40; `pan_sign` -1 if bit 7 of its two low bytes differ), the frequency (also
//! `OWN_FREQUENCY`; held to 100 .. 200,000, signed) and, in 3D, the place's three floats,
//! each marked dirty if it changed; kind 3 marks a restart; and the play bit is set.
//! Refuses, before anything is written: a number of 0x1CD or more (its own or the chosen
//! voice's); no channels left and no voice playing; a playing list that does not end
//! within 0x200 voices; a playing voice to stop without its channel.

use crate::audio::owner::Audio;
use crate::audio::voices::{GamePool, lists, store_lists};
use crate::errors::Refused;

// ---- Rust version ----

/// Converted: `rc2k_core::audio::voices::start_voice`; nothing is written when it refuses.
pub fn start_voice(audio: &Audio, sound: i32, cmd: &[u32; 8]) -> Result<(), Refused> {
    let mut l = lists(&audio.state);
    let mode = audio.state.sound_mode.get();
    rc2k_core::audio::voices::start_voice(&mut GamePool::new(audio), &mut l, mode, sound, cmd)?;
    store_lists(&audio.state, &l);
    Ok(())
}
