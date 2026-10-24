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

use crate::audio::store::Slot;
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::rva;
use crate::store::addr32 as a;
use crate::store::game_addr;

const USING_MODES_TEXT: usize = rva(0x4DA6AD); // "AUD: Using modes"

#[derive(Clone, Copy)]
pub struct StartVars {
    pub use_a3d: u32,
    pub has_a3d2: u32,
    pub mode: u32,
    pub a3d_ready: u32,
    pub splash_kept: u32,
    /// `g_soundCaps`: flags at +4, free hardware buffers at +0x14, free 3D ones at +0x3C.
    pub caps: u32,
    pub has_3d: u32,
    pub voices: u32,
    pub mix_rate_setting: u32,
    pub voice_manager_on: u32,
    pub has_eax: u32,
    pub menu: u32,
    pub race: u32,
}

impl StartVars {
    pub fn game() -> Self {
        Self {
            use_a3d: a(Slot::at(0x506D60)),
            has_a3d2: a(Slot::at(0x506D5C)),
            mode: a(Slot::at(0x4F82BC)),
            a3d_ready: a(Slot::at(0x506C64)),
            splash_kept: a(Slot::at(0x506C60)),
            caps: a(Slot::at(0x506AA8)),
            has_3d: a(Slot::at(0x506B46)),
            voices: a(Slot::at(0x4F834C)),
            mix_rate_setting: a(Slot::at(0x4F82D4)),
            voice_manager_on: a(Slot::at(0x506C54)),
            has_eax: a(Slot::at(0x506C70)),
            menu: a(Slot::at(0x4F82C4)),
            race: a(Slot::at(0x4F82C8)),
        }
    }
}

/// False for the carry (no race mode).
pub unsafe fn start(v: &StartVars) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    use crate::audio::voices::{A3D_RVA, PRIMARY_RVA, SOUND_MODE_RVA, VOICE_MANAGER_RVA};
    use crate::store::game_read;
    use rc2k_core::audio::device::{
        A3D2_MODE, CAPS_EMULATED, Fact, MENU_MODE, modes_kept, probed_race_mode,
    };
    let direct_sound = || game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32;
    let release_primary = || unsafe {
        sd::release_primary(
            game_addr(PRIMARY_RVA) as u32,
            game_addr(sd::VOLUME_KEPT_RVA) as u32,
        )
    };
    unsafe {
        crate::audio::a3d::open_a3d2(
            &crate::audio::a3d::A3dVars::game(),
            &mut crate::audio::a3d::game_create,
        )?;
        write(v.has_a3d2, read(v.use_a3d));
        crate::audio::a3d::close_a3d2(
            game_addr(crate::audio::a3d::A3D2_INTERFACE_RVA) as u32,
            game_addr(A3D_RVA) as u32,
        )?;
        // The original's ebx: the race mode found.
        let mut found = MENU_MODE;
        write(v.mode, MENU_MODE | 0x190_0000);
        let mut to_log = false;
        if !sd::open_sound_in_game(0)? {
            if read(v.has_a3d2) == 1 {
                found = A3D2_MODE;
            } else {
                write(v.menu, 0);
                write(v.race, 0);
                to_log = true;
            }
        } else {
            write(v.splash_kept, read(v.a3d_ready));
            let flags = |v: &StartVars| read(v.caps + 4);
            if sd::read_caps(direct_sound(), game_addr(sd::CAPS_RVA) as u32)?
                && flags(v) & CAPS_EMULATED == 0
            {
                crate::audio::primary::make_primary(&crate::audio::primary::PrimaryVars::game())?;
                let has_3d = (read(v.mode) & 0x10_0000 != 0) as u32;
                (v.has_3d as *mut u32).write_unaligned(has_3d);
                crate::audio::mixer_set_up::open_way(game_read(SOUND_MODE_RVA))?;
                let voice = v.voices;
                std::ptr::write_bytes(voice as *mut u8, 0, 0x70);
                write(voice + 0x28, 0x121);
                write(voice + 0x60, read(v.mix_rate_setting));
                write(voice, 1);
                write(voice + 0x20, 0x4000);
                crate::audio::make_buffer::make_buffer(
                    voice,
                    MENU_MODE,
                    &crate::audio::make_buffer::Vars::game(),
                )?;
                crate::audio::release_eax::release_eax(crate::audio::release_eax::game_vars());
                crate::audio::release_voice_manager::release_voice_manager(game_addr(
                    VOICE_MANAGER_RVA,
                )
                    as u32);
                crate::audio::release_voice::release_voice_in_game(voice);
                crate::audio::mixer_set_up::close_way(game_read(SOUND_MODE_RVA))?;
                release_primary()?;
                // Converted: `rc2k_core::audio::device::probed_race_mode`.
                found = probed_race_mode(|fact| match fact {
                    Fact::CapsFlags => flags(v),
                    Fact::FreeHardware => read(v.caps + 0x14),
                    Fact::Has3d => (v.has_3d as *const u32).read_unaligned(),
                    Fact::HasA3d2 => read(v.has_a3d2),
                    Fact::VoiceManagerOn => read(v.voice_manager_on),
                    Fact::Free3d => read(v.caps + 0x3C),
                    Fact::HasEax => read(v.has_eax),
                });
            }
        }
        if !to_log {
            if !modes_kept(read(v.race), || read(v.menu)) {
                write(v.race, found);
                write(v.menu, MENU_MODE);
                write(v.race, MENU_MODE);
            }
            crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
        }
        sd::log_text(game_addr(USING_MODES_TEXT) as u32, false);
        sd::log_hex(read(v.menu), false);
        sd::log_hex(read(v.race), true);
        sd::close_sound(direct_sound(), crate::session::MACHINE.window.addr() as u32)?;
        write(v.mode, 0);
        Ok(read(v.race) >= 1)
    }
}
