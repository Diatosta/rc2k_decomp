//! Makes a voice's DirectSound buffer: nothing (success) unless voices have buffers of their own.
//! A voice that shares data gets a duplicate of the other's buffer; if that fails, or for any
//! other voice, the wave format is described and the buffer created (frequency, pan and volume
//! controls, in software unless 3D, static for the voice's flags bit 0, 3D in 3D); failing:
//! "AUD: 1-DSBCreate failed" and the result are logged. Then the volume, pan and frequency are
//! set. The openings read the 3D buffer through a "holder" value that is the caller's own value
//! (`property_sets::Holder`), or the frequency after the create path, or the voice's 3D buffer
//! after the voice manager's call (a game bug kept; `open_eax` refuses one that
//! is no pointer). Refuses what the settings refuse, checked first.

use crate::audio::buffer_settings::{set_buffer_frequency, set_buffer_pan, set_buffer_volume};
use crate::audio::describe_wave_format::describe_wave_format;
use crate::audio::own::AudioOwn;
use crate::audio::property_sets::Holder;
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};

const QUERY_FAILED: &std::ffi::CStr = c"AUD: 1-DSBQuery3D failed";
pub(super) const MODE_OWN_BUFFERS: u32 = 0x1_0000;
use rc2k_core::audio::voices::MODE_HARDWARE_3D;
const MODE_EAX: u32 = 0x100_0000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BufferDescription {
    pub size: u32,
    pub flags: u32,
    pub bytes: u32,
    pub reserved: u32,
    /// Not used: the format goes to the platform as a value.
    pub format: u32,
}

crate::memory::cells::plain_record!(BufferDescription = 0x14);
/// True when it worked.
pub fn make_buffer(state: &AudioOwn, at: u32, holder: Holder) -> Result<bool, Refused> {
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
        let mut holder = holder;
        let mut made = false;
        if v.source != 0 {
            match crate::platform::sound::duplicate_sound_buffer(voice_of(state, v.source).buffer) {
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
            let flags = rc2k_core::audio::voices::buffer_flags(mode, v.flags);
            state.buffer_description.set(BufferDescription {
                size: 0x14,
                flags,
                bytes: v.length,
                reserved: 0,
                format: 0,
            });
            v.buffer = crate::platform::sound::create_sound_buffer(
                flags,
                v.length,
                Some(crate::audio::describe_wave_format::platform_format(state)),
            );
        }
        set_buffer_volume(state, v)?;
        set_buffer_pan(state, v)?;
        set_buffer_frequency(v)?;
        if mode & MODE_3D != 0 {
            match crate::platform::sound::query_interface(
                v.buffer,
                state.iid_sound_3d_buffer.get_u32(0),
            ) {
                Ok(b) => v.buffer_3d = b,
                Err(r) => {
                    v.buffer_3d = 0;
                    return Ok(log_failure(QUERY_FAILED, r));
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

/// The failure's text and result to the log; false.
fn log_failure(text: &std::ffi::CStr, result: i32) -> bool {
    crate::logfile::log_write::log_nul(&text, false, 1);
    crate::logfile::log_hex8::log_hex8(result as u32, true, 1);
    false
}
