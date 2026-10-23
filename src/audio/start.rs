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
//! Shadow mode runs the original alone. Self-test: a redirected copy whose calls go to
//! stand-ins noting what each reads, 20 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

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
        let a = |r: usize| game_addr(rva(r)) as u32;
        Self {
            use_a3d: a(0x506D60),
            has_a3d2: a(0x506D5C),
            mode: a(0x4F82BC),
            a3d_ready: a(0x506C64),
            splash_kept: a(0x506C60),
            caps: a(0x506AA8),
            has_3d: a(0x506B46),
            voices: a(0x4F834C),
            mix_rate_setting: a(0x4F82D4),
            voice_manager_on: a(0x506C54),
            has_eax: a(0x506C70),
            menu: a(0x4F82C4),
            race: a(0x4F82C8),
        }
    }
}

/// A call (its place, the registers the original has there) -> its carry.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// False for the carry (no race mode).
pub unsafe fn start(entry: &Regs, v: &StartVars, part: Part) -> Result<bool, Refused> {
    use rc2k_core::audio::device::{
        A3D2_MODE, CAPS_EMULATED, Fact, MENU_MODE, modes_kept, probed_race_mode,
    };
    unsafe {
        let mut c = *entry;
        let mut call = |place: u32, c: &Regs| -> Result<bool, Refused> {
            let mut r = *c;
            part(place, &mut r)
        };
        c.eax = 0;
        call(0, &c)?;
        c.eax = read(v.use_a3d);
        write(v.has_a3d2, c.eax);
        call(1, &c)?;
        c.ebx = MENU_MODE;
        write(v.mode, c.ebx | 0x190_0000);
        c.eax = 0;
        let mut to_log = false;
        if call(2, &c)? {
            if read(v.has_a3d2) == 1 {
                c.ebx = A3D2_MODE;
            } else {
                c.eax = 0;
                write(v.menu, 0);
                write(v.race, 0);
                to_log = true;
            }
        } else {
            c.eax = read(v.a3d_ready);
            write(v.splash_kept, c.eax);
            let flags = |v: &StartVars| read(v.caps + 4);
            if !call(3, &c)? && flags(v) & CAPS_EMULATED == 0 {
                call(4, &c)?;
                c.eax = (read(v.mode) & 0x10_0000 != 0) as u32;
                (v.has_3d as *mut u32).write_unaligned(c.eax);
                call(5, &c)?;
                c.esi = v.voices;
                std::ptr::write_bytes(c.esi as *mut u8, 0, 0x70);
                write(c.esi + 0x28, 0x121);
                c.eax = read(v.mix_rate_setting);
                write(c.esi + 0x60, c.eax);
                write(c.esi, 1);
                write(c.esi + 0x20, 0x4000);
                for place in 6..12 {
                    call(place, &c)?;
                }
                // Converted: `rc2k_core::audio::device::probed_race_mode`.
                c.ebx = probed_race_mode(|fact| match fact {
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
                write(v.race, c.ebx);
                c.ebx = MENU_MODE;
                write(v.menu, c.ebx);
                write(v.race, c.ebx);
            }
            call(12, &c)?;
        }
        for (place, eax) in [
            (13, game_addr(USING_MODES_TEXT) as u32),
            (14, read(v.menu)),
            (15, read(v.race)),
        ] {
            call(place, &Regs { eax, ..c })?;
        }
        call(16, &c)?;
        write(v.mode, 0);
        Ok(read(v.race) >= 1)
    }
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    unsafe {
        Ok(match place {
            0 => !crate::audio::a3d::open_a3d2(
                &crate::audio::a3d::A3dVars::game(),
                &mut crate::audio::a3d::game_create,
            )?,
            1 => {
                crate::audio::a3d::close_a3d2(
                    game_addr(crate::audio::a3d::A3D2_INTERFACE_RVA) as u32,
                    game_addr(crate::audio::voices::A3D_RVA) as u32,
                )?;
                false
            }
            2 => !sd::open_sound(
                r.eax,
                &sd::OpenVars::game(),
                &mut sd::game_part,
                &mut sd::game_direct_sound_create,
            )?,
            3 => !sd::read_caps(
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(sd::CAPS_RVA) as u32,
            )?,
            4 => !crate::audio::primary::make_primary(&crate::audio::primary::PrimaryVars::game())?,
            5 => crate::audio::mixer_set_up::open_way(crate::framework::game_read(
                crate::audio::voices::SOUND_MODE_RVA,
            ))?,
            6 => !crate::audio::make_buffer::make_buffer(
                r.esi,
                r.ebx,
                &crate::audio::make_buffer::Vars::game(),
                &mut crate::audio::make_buffer::game_part,
            )?,
            7 => {
                crate::audio::release_eax::release_eax(crate::audio::release_eax::game_vars());
                false
            }
            8 => {
                crate::audio::release_voice_manager::release_voice_manager(game_addr(
                    crate::audio::voices::VOICE_MANAGER_RVA,
                )
                    as u32);
                false
            }
            9 => {
                crate::audio::release_voice::release_voice_in_game(r.esi);
                false
            }
            10 => crate::audio::mixer_set_up::close_way(crate::framework::game_read(
                crate::audio::voices::SOUND_MODE_RVA,
            ))?,
            11 => {
                sd::release_primary(
                    game_addr(crate::audio::voices::PRIMARY_RVA) as u32,
                    game_addr(sd::VOLUME_KEPT_RVA) as u32,
                )?;
                false
            }
            12 => {
                crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
                false
            }
            13 => sd::game_part(0, r.eax)?,
            14 => sd::game_part(2, r.eax)?,
            15 => sd::game_part(3, r.eax)?,
            _ => {
                sd::close_sound(
                    game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                    game_addr(sd::WINDOW_RVA) as u32,
                )?;
                false
            }
        })
    }
}
