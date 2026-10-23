//! Lost buffers restored, and the mixer's primary buffer closed (IDA, 2026-10-15):
//! `audio::restore_buffer`, `replay_looping`, `restore_lost`, `close_primary_mixer`. Their
//! calls go through a `Part` (the place, eax, esi -> the carry), so that the tests can
//! stand them in; in the game they are the Rust versions.

use crate::audio::voices::{read, write};
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

pub const WINDOW_ACTIVE_RVA: usize = rva(0x518374); // g_windowActive
pub const GET_STATUS: u32 = 0x24;
pub const RESTORE: u32 = 0x50;
pub type Part<'a> = &'a mut dyn FnMut(u32, u32, u32) -> Result<bool, Refused>;

/// The Rust versions, as the game has them.
pub fn game_part(place: u32, eax: u32, esi: u32) -> Result<bool, Refused> {
    use crate::audio::voices::*;
    let lost = game_addr(BUFFER_LOST_RVA) as u32;
    let primary = game_addr(PRIMARY_RVA) as u32;
    unsafe {
        Ok(match place {
            0 => restore_buffer(eax, lost)?.carry,
            1 => !crate::audio::fill_buffer::fill_buffer(
                voice(esi),
                game_addr(crate::audio::fill_buffer::LOCKED_PARTS_RVA) as u32,
                lost,
            )?,
            2 => {
                crate::audio::thread_parts::play_primary(primary, lost);
                false
            }
            3 => {
                replay_looping(game_addr(PLAYING_RVA) as u32)?;
                false
            }
            4 => crate::audio::buffer_play::play_buffer_in_game(voice(esi))?.1,
            5 => {
                windows_sys::Win32::System::Threading::Sleep(1000);
                false
            }
            6 => {
                crate::audio::clear_channels::clear_channels(World::game().channels);
                false
            }
            7 => {
                crate::audio::sound_device::release_primary(
                    primary,
                    game_addr(crate::audio::sound_device::VOLUME_KEPT_RVA) as u32,
                )?;
                false
            }
            8 => !crate::audio::primary::make_primary(&crate::audio::primary::PrimaryVars::game())?,
            9 => {
                let rate = crate::framework::game_read(crate::audio::primary::MIX_RATE_SETTING_RVA);
                !crate::audio::primary::set_primary_format(primary, eax, rate)?
            }
            10 => close_primary_mixer(
                game_addr(SOUND_MODE_RVA) as u32,
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(crate::audio::sound_device::WINDOW_RVA) as u32,
            )?,
            _ => !crate::audio::mixer_set_up::set_up_primary_mixer(
                &crate::audio::mixer_set_up::MixerVars::game(),
                &mut game_part,
            )?,
        })
    }
}

/// What `restore_buffer` leaves: the carry, and eax and edx where it changes them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Restored {
    pub carry: bool,
    pub eax: Option<u32>,
    pub edx: Option<u32>,
}

/// `audio::restore_buffer`: the buffer whose pointer is at `at`, if any: `GetStatus`
/// (+0x24, into edx, which `Restore` then leaves as it likes); lost: `Restore` (+0x50, into eax), whose failure sets the flag at
/// `lost` and the carry. Refuses A3D (0x40CB0A).
pub unsafe fn restore_buffer(at: u32, lost: u32) -> Result<Restored, Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    let mut left = Restored {
        carry: false,
        eax: None,
        edx: None,
    };
    unsafe {
        let b = read(at);
        if b == 0 {
            return Ok(left);
        }
        let mut status = 0u32;
        crate::audio::com::call1(b, GET_STATUS, &mut status as *mut u32 as u32);
        left.edx = Some(status);
        // Converted: `rc2k_core::audio::buffers::needs_restore`.
        if !rc2k_core::audio::buffers::needs_restore(status) {
            return Ok(left);
        }
        let r = crate::audio::com::call(b, RESTORE);
        left.eax = Some(r as u32);
        // `Restore` may change edx as it likes (as any COM call): not the status any more.
        left.edx = None;
        if r != 0 {
            write(lost, 1);
            left.carry = true;
        }
    }
    Ok(left)
}

/// `audio::replay_looping`: each playing voice (the list at `playing`, linked at +0x48)
/// whose +0x3C is 2 and that has a buffer (+8) played again (`play_buffer`, place 4).
pub unsafe fn replay_looping(playing: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::sounding_to_replay` (the list is walked before
    // any is played, so a list without its end plays none).
    unsafe {
        let lists = rc2k_core::audio::voices::Lists {
            playing: read(playing),
            ..Default::default()
        };
        let mut pool = crate::audio::voices::GamePool::new(&crate::audio::voices::World::game());
        for v in rc2k_core::audio::voices::sounding_to_replay(&mut pool, &lists)? {
            crate::audio::restore::game_part(4, 0, v)?;
        }
    }
    Ok(())
}

/// Where `restore_lost`'s things are.
#[derive(Clone, Copy)]
pub struct RestoreVars {
    pub lost: u32,
    pub primary: u32,
    pub voice_count: u32,
    pub voices: u32,
    pub window_active: u32,
}

impl RestoreVars {
    pub fn game() -> Self {
        use crate::audio::voices::*;
        let a = |r| game_addr(r) as u32;
        Self {
            lost: a(BUFFER_LOST_RVA),
            primary: a(PRIMARY_RVA),
            voice_count: a(VOICE_COUNT_RVA),
            voices: a(VOICES_RVA),
            window_active: a(WINDOW_ACTIVE_RVA),
        }
    }
}

/// `audio::restore_lost`: a second's wait; with the window active: the flag cleared, the
/// primary buffer and each voice's buffer restored (and the voice's refilled), the primary
/// played, the looping voices replayed. A failure ends it. Refuses A3D (0x40CB0A).
pub unsafe fn restore_lost(v: &RestoreVars) -> Result<(), Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CB0A)").into());
    }
    unsafe {
        crate::audio::restore::game_part(5, 0, 0)?;
        if read(v.window_active) == 0 {
            return Ok(());
        }
        write(v.lost, 0);
        if crate::audio::restore::game_part(0, v.primary, 0)? {
            return Ok(());
        }
        for n in 0..read(v.voice_count) {
            let at = v.voices + n * std::mem::size_of::<crate::audio::voices::Voice>() as u32;
            if crate::audio::restore::game_part(0, at + 8, at)?
                || crate::audio::restore::game_part(1, 0, at)?
            {
                return Ok(());
            }
        }
        crate::audio::restore::game_part(2, 0, 0)?;
        crate::audio::restore::game_part(3, 0, 0)?;
    }
    Ok(())
}

/// `audio::close_primary_mixer`: the channels cleared; in the mixer's mode with a
/// DirectSound: `release_primary`, cooperative level 3, `make_primary`. The carry it
/// leaves: `make_primary`'s, when that ran.
pub unsafe fn close_primary_mixer(
    mode: u32,
    direct_sound: u32,
    window: u32,
) -> Result<bool, Refused> {
    unsafe {
        crate::audio::restore::game_part(6, 0, 0)?;
        // Converted: `rc2k_core::audio::buffers::remakes_primary`.
        if !rc2k_core::audio::buffers::remakes_primary(read(mode), || read(direct_sound)) {
            return Ok(false);
        }
        crate::audio::restore::game_part(7, 0, 0)?;
        let ds = read(direct_sound);
        let level: unsafe extern "system" fn(u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(ds, crate::audio::sound_device::SET_COOPERATIVE_LEVEL),
        );
        level(ds, read(window), 3);
        crate::audio::restore::game_part(8, 0, 0)
    }
}

// ---- for the self-tests ----
