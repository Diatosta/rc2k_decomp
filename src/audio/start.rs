//! Probes the sound device at the audio thread's start and chooses the two sound modes: A3D2
//! tried, DirectSound opened, its caps read; unless that fails or the caps say emulated, the
//! primary made, a probe voice made through `make_buffer` and released, and the 3D / EAX / voice
//! manager abilities noted for the race mode. Not opened: A3D2's race mode with A3D2, else both
//! modes 0. Unless both modes are kept, the race mode is the probe's and at once the menu's (the
//! probe thrown away: a game slip kept). Then `audio::choose_modes`, "AUD: Using
//! modes" logged and the device closed.

use crate::audio::own::AudioThread;
use crate::errors::Refused;

const USING_MODES_TEXT: &std::ffi::CStr = c"AUD: Using modes";

/// False when there is no race mode.
pub fn start(
    audio: &AudioThread,
    system: &'static crate::system::owner::System,
) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    use rc2k_core::audio::device::{
        A3D2_MODE, CAPS_EMULATED, Fact, MENU_MODE, modes_kept, probed_race_mode,
    };
    crate::audio::a3d::open_a3d2(&audio.state)?;
    audio.state.has_a3d2.set(audio.state.use_a3d.get());
    crate::audio::a3d::close_a3d2(&audio.state);
    // The race mode found.
    let mut found = MENU_MODE;
    audio.state.sound_mode.set(MENU_MODE | 0x190_0000);
    let mut to_log = false;
    if !sd::open_sound_in_game(&audio.state, system, 0)? {
        if audio.state.has_a3d2.get() == 1 {
            found = A3D2_MODE;
        } else {
            audio.state.menu_sound_mode.set(0);
            audio.state.race_sound_mode.set(0);
            to_log = true;
        }
    } else {
        audio.state.a3d_splash_kept.set(audio.state.a3d_ready.get());
        let flags = || audio.state.sound_caps.get_u32(4);
        if sd::read_caps(&audio.state)? && flags() & CAPS_EMULATED == 0 {
            crate::audio::primary::make_primary(audio, system)?;
            let has_3d = (audio.state.sound_mode.get() & 0x10_0000 != 0) as u32;
            audio.state.has_3d.set(has_3d);
            crate::audio::mixer_set_up::open_way(audio, system, audio.state.sound_mode.get())?;
            // A probe voice in the pool's first record: zeroed, then its format, the mixing rate
            // setting, flags 1 and 0x4000 bytes.
            let probe = &audio.state.voices[0];
            probe.update(|v| {
                *v = Default::default();
                v.format = 0x121;
                v.frequency = audio.state.mix_rate_setting.get();
                v.flags = 1;
                v.length = 0x4000;
            });
            let voice = crate::audio::voices::voice_id(0);
            crate::audio::make_buffer::make_buffer(
                &audio.state,
                voice,
                crate::audio::property_sets::Holder::Value,
            )?;
            crate::audio::release_eax::release_eax(&audio.state);
            crate::audio::release_voice_manager::release_voice_manager(&audio.state);
            crate::audio::release_voice::release_voice_in_game(&audio.state, voice);
            crate::audio::mixer_set_up::close_way(audio, system, audio.state.sound_mode.get())?;
            sd::release_primary(&audio.state);
            found = probed_race_mode(|fact| match fact {
                Fact::CapsFlags => flags(),
                Fact::FreeHardware => audio.state.sound_caps.get_u32(0x14),
                Fact::Has3d => audio.state.has_3d.get(),
                Fact::HasA3d2 => audio.state.has_a3d2.get(),
                Fact::VoiceManagerOn => audio.state.voice_manager_on.get(),
                Fact::Free3d => audio.state.sound_caps.get_u32(0x3C),
                Fact::HasEax => audio.state.has_eax.get(),
            });
        }
    }
    if !to_log {
        if !modes_kept(audio.state.race_sound_mode.get(), || {
            audio.state.menu_sound_mode.get()
        }) {
            audio.state.race_sound_mode.set(found);
            audio.state.menu_sound_mode.set(MENU_MODE);
            audio.state.race_sound_mode.set(MENU_MODE);
        }
        crate::audio::choose_modes::in_game(audio, &mut crate::audio::choose_modes::real_log)?;
    }
    sd::log_text(USING_MODES_TEXT, false);
    sd::log_hex(audio.state.menu_sound_mode.get(), false);
    sd::log_hex(audio.state.race_sound_mode.get(), true);
    sd::close_sound(&audio.state, system)?;
    audio.state.sound_mode.set(0);
    Ok(audio.state.race_sound_mode.get() >= 1)
}
