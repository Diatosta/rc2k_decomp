//! `audio::start` (0x40A01D): the sound device probed at the audio thread's start, and the
//! two sound modes chosen.
//!
//! In:  nothing
//! Out: carry = no race mode (no sound); every register preserved
//!
//! `audio::open_a3d2` (eax 0), `g_hasA3d2` = `g_useA3d`, `audio::close_a3d2`; the mode
//! 0x11221 | 0x1900000; `audio::open_sound` (eax 0). Opened: `g_a3dSplashKept` =
//! `g_a3dReady`; `audio::read_caps`; unless it fails or the caps say emulated (0x20):
//! `audio::make_primary`, `g_has3d` from the mode's 3D bit, `audio::open_way`, a probe voice
//! in `g_voices[0]` (zeroed; format 0x121, `g_mixRateSetting`, flags 1, 0x4000 bytes)
//! through `audio::make_buffer` (ebx 0x11221, esi the voice), `release_eax`,
//! `release_voice_manager`, `release_voice`, `audio::close_way`, `audio::release_primary`; with
//! caps flag 0x100, free hardware buffers and 3D: race bits 0x112221 for A3D2, else with
//! `g_voiceManagerOn` and at least 6 free 3D buffers 0x900000 (and 0x1000000 with `g_hasEax`). Not
//! opened: 0x112221 with A3D2, else both modes 0 and straight to the log. Then: unless
//! both modes are kept, the race mode is the probe's and at once the menu's 0x11221 (the
//! probe thrown away: section 16); `audio::choose_modes`; "AUD: Using modes" and both;
//! `audio::close_sound`; the mode 0. One caller, the audio thread.

use crate::audio::store::AUDIO;
use crate::errors::Refused;

const USING_MODES_TEXT: &std::ffi::CStr = c"AUD: Using modes"; // "AUD: Using modes"

/// False for the carry (no race mode).
pub fn start() -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    use rc2k_core::audio::device::{
        A3D2_MODE, CAPS_EMULATED, Fact, MENU_MODE, modes_kept, probed_race_mode,
    };
    crate::audio::a3d::open_a3d2()?;
    AUDIO.has_a3d2.set(AUDIO.use_a3d.get());
    crate::audio::a3d::close_a3d2()?;
    // The original's ebx: the race mode found.
    let mut found = MENU_MODE;
    AUDIO.sound_mode.set(MENU_MODE | 0x190_0000);
    let mut to_log = false;
    if !sd::open_sound_in_game(0)? {
        if AUDIO.has_a3d2.get() == 1 {
            found = A3D2_MODE;
        } else {
            AUDIO.menu_sound_mode.set(0);
            AUDIO.race_sound_mode.set(0);
            to_log = true;
        }
    } else {
        AUDIO.a3d_splash_kept.set(AUDIO.a3d_ready.get());
        let flags = || AUDIO.sound_caps.get_u32(4);
        if sd::read_caps()? && flags() & CAPS_EMULATED == 0 {
            crate::audio::primary::make_primary()?;
            let has_3d = (AUDIO.sound_mode.get() & 0x10_0000 != 0) as u32;
            AUDIO.has_3d.set(has_3d);
            crate::audio::mixer_set_up::open_way(AUDIO.sound_mode.get())?;
            // A probe voice in the pool's first record: zeroed, then its format, the mixing
            // rate setting, flags 1 and 0x4000 bytes.
            let probe = &AUDIO.voices[0];
            probe.update(|v| {
                *v = Default::default();
                v.format = 0x121;
                v.frequency = AUDIO.mix_rate_setting.get();
                v.flags = 1;
                v.length = 0x4000;
            });
            let voice = crate::audio::voices::voice_id(0);
            crate::audio::make_buffer::make_buffer(voice, MENU_MODE)?;
            crate::audio::release_eax::release_eax();
            crate::audio::release_voice_manager::release_voice_manager();
            crate::audio::release_voice::release_voice_in_game(voice);
            crate::audio::mixer_set_up::close_way(AUDIO.sound_mode.get())?;
            sd::release_primary()?;
            // Converted: `rc2k_core::audio::device::probed_race_mode`.
            found = probed_race_mode(|fact| match fact {
                Fact::CapsFlags => flags(),
                Fact::FreeHardware => AUDIO.sound_caps.get_u32(0x14),
                Fact::Has3d => AUDIO.has_3d.get(),
                Fact::HasA3d2 => AUDIO.has_a3d2.get(),
                Fact::VoiceManagerOn => AUDIO.voice_manager_on.get(),
                Fact::Free3d => AUDIO.sound_caps.get_u32(0x3C),
                Fact::HasEax => AUDIO.has_eax.get(),
            });
        }
    }
    if !to_log {
        if !modes_kept(AUDIO.race_sound_mode.get(), || AUDIO.menu_sound_mode.get()) {
            AUDIO.race_sound_mode.set(found);
            AUDIO.menu_sound_mode.set(MENU_MODE);
            AUDIO.race_sound_mode.set(MENU_MODE);
        }
        crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
    }
    sd::log_text(
        crate::pointers::game_address(USING_MODES_TEXT.as_ptr()),
        false,
    );
    sd::log_hex(AUDIO.menu_sound_mode.get(), false);
    sd::log_hex(AUDIO.race_sound_mode.get(), true);
    sd::close_sound()?;
    AUDIO.sound_mode.set(0);
    Ok(AUDIO.race_sound_mode.get() >= 1)
}
