//! `audio::make_buffer` (0x40B5CF): a voice's DirectSound buffer made.
//!
//! Takes: the voice
//!
//! Nothing (carry clear) unless voices have buffers of their own (`g_soundMode` bit
//! 0x10000). No `g_directSound`: carry. A voice that shares data gets a duplicate of the
//! other's buffer (`DuplicateSoundBuffer`, +0x14); if that fails, or for any other voice,
//! `audio::describe_wave_format` and `CreateSoundBuffer` (+0x0C) with
//! `g_bufferDescription`: size 0x14, flags 0xE0 (frequency, pan and volume controls), + 8
//! (in software) unless 3D with bit 0x800000, + 2 (static) for the voice's flags bit 0, +
//! 0x10 (3D) in 3D, the length, `g_waveFormat`; failing: "AUD: 1-DSBCreate failed" and the
//! result to the game's log, carry. Then the volume, pan and frequency
//! (`audio::set_buffer_*`). In 3D: the 3D interface (`QueryInterface` with
//! `g_iidSound3dBuffer`; failing: "AUD: 1-DSBQuery3D failed" logged, carry), the place,
//! `audio::set_buffer_min_distance`, `g_commit3d` 1, `audio::open_voice_manager` with bit
//! 0x800000, with EAX (bit 0x1000000) `audio::open_eax` and, unless that sets the carry,
//! `audio::set_eax_room`. The openings read the 3D buffer through a "holder" value that
//! is the caller's own value (`property_sets::Holder`), or the frequency after the create
//! path, or the voice's 3D buffer after the voice manager's call (the game's bug, kept;
//! `open_eax` refuses one that is no pointer). Callers:
//! command 3 and `audio::start`.
//! Refuses A3D (the chunk at 0x40CC88), and what the settings refuse, checked first.

use crate::audio::buffer_settings::{a3d, set_buffer_frequency, set_buffer_pan, set_buffer_volume};
use crate::audio::describe_wave_format::describe_wave_format;
use crate::audio::own::AudioOwn;
use crate::audio::property_sets::Holder;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

const QUERY_FAILED_RVA: &std::ffi::CStr = c"AUD: 1-DSBQuery3D failed"; // "AUD: 1-DSBQuery3D failed"
pub const MODE_OWN_BUFFERS: u32 = 0x1_0000;
use rc2k_core::audio::voices::MODE_HARDWARE_3D;
const MODE_EAX: u32 = 0x100_0000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BufferDescription {
    pub size: u32,
    pub flags: u32,
    pub bytes: u32,
    pub reserved: u32,
    /// The wave format's address (the original kept it). Not used: nothing reads it, the
    /// format goes to the platform as a value.
    pub format: u32,
}

/// True for the carry clear. `holder`: the caller's value, which the EAX parts read the 3D
/// buffer through (replaced as below, as the original does).
pub fn make_buffer(state: &AudioOwn, at: u32, holder: Holder) -> Result<bool, Refused> {
    if a3d(state) {
        return Err(BadInput("A3D (the chunk at 0x40CC88)").into());
    }
    let mode = state.sound_mode.get();
    if mode & MODE_OWN_BUFFERS == 0 {
        return Ok(true);
    }
    let ds = state.direct_sound.get();
    if ds == 0 {
        return Ok(false);
    }
    update_voice(state, at, |v| -> Result<bool, Refused> {
        // What the settings would refuse, before anything is made.
        let level = (v.volume as i32).wrapping_mul(v.volume_factor as i32) >> 6;
        if !(0..=0x40).contains(&level) || !(-0x40..=0x40).contains(&v.pan) {
            return Err(BadInput("a volume or pan past the volume table").into());
        }
        // The holder value at each call, as the original has it: the caller's, the
        // frequency after the create path, the voice's 3D buffer after
        // `audio::open_voice_manager`'s.
        let mut holder = holder;
        let mut made = false;
        if v.source != 0 {
            match crate::platform_sound::duplicate_sound_buffer(voice_of(state, v.source).buffer) {
                Ok(b) => {
                    v.buffer = b;
                    made = true;
                }
                Err(_) => v.source = 0,
            }
        }
        if !made {
            holder = Holder::Value;
            describe_wave_format(state, v.format, v.frequency)?;
            // Converted: `rc2k_core::audio::voices::buffer_flags`.
            let flags = rc2k_core::audio::voices::buffer_flags(mode, v.flags);
            state.buffer_description.set(BufferDescription {
                size: 0x14,
                flags,
                bytes: v.length,
                reserved: 0,
                format: 0,
            });
            v.buffer = crate::platform_sound::create_sound_buffer(
                flags,
                v.length,
                Some(crate::audio::describe_wave_format::platform_format(state)),
            );
        }
        set_buffer_volume(state, v)?;
        set_buffer_pan(state, v)?;
        set_buffer_frequency(state, v)?;
        if mode & MODE_3D != 0 {
            match crate::platform_sound::query_interface(
                v.buffer,
                state.iid_sound_3d_buffer.get_u32(0),
            ) {
                Ok(b) => v.buffer_3d = b,
                Err(r) => {
                    v.buffer_3d = 0;
                    return Ok(log_failure(QUERY_FAILED_RVA, r));
                }
            }
            crate::audio::buffer_settings::set_buffer_place(v)?;
            crate::audio::set_buffer_min_distance::set_buffer_min_distance(v)?;
            state.commit_3d.set(1);
            if mode & MODE_HARDWARE_3D != 0 {
                holder = stored_buffer_3d(state, at, v);
                crate::audio::open_voice_manager::open_voice_manager(state, holder)?;
            }
            if mode & MODE_EAX != 0 && crate::audio::open_eax::open_eax(state, holder)? {
                crate::audio::set_eax_room::in_game(state)?;
            }
        }
        Ok(true)
    })
}

/// The failure's text and result to the game's log; false (the carry).
fn log_failure(text: &std::ffi::CStr, result: i32) -> bool {
    crate::logfile::log_write::log_nul(&text, false, 1);
    crate::logfile::log_hex8::log_hex8(result as u32, true, 1);
    false
}
