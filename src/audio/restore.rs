//! Lost buffers restored, and the mixer's primary buffer closed (IDA, 2026-10-15):
//! `audio::restore_buffer`, `replay_looping`, `restore_lost`, `close_primary_mixer`. Their
//! calls go through a `Part` (the place, eax, esi -> the carry), so that the tests can
//! stand them in; in the game they are the Rust versions.

use crate::audio::voices::{read, write};
use crate::errors::{BadInput, Refused};
use crate::session::FLAGS;
use crate::store::addr32 as a;
use crate::store::game_addr;

pub const GET_STATUS: u32 = 0x24;
pub const RESTORE: u32 = 0x50;
fn lost() -> u32 {
    game_addr(crate::audio::voices::BUFFER_LOST_RVA) as u32
}

fn primary() -> u32 {
    game_addr(crate::audio::voices::PRIMARY_RVA) as u32
}

/// `restore_buffer` on the game's lost flag: true when it failed (the carry).
fn restore_failed(at: u32) -> Result<bool, Refused> {
    Ok(unsafe { restore_buffer(at, lost()) }?.carry)
}

/// The voice at `at`'s buffer refilled: true when it failed.
fn refill_failed(at: u32) -> Result<bool, Refused> {
    let parts = game_addr(crate::audio::fill_buffer::LOCKED_PARTS_RVA) as u32;
    Ok(!unsafe {
        crate::audio::fill_buffer::fill_buffer(crate::audio::voices::voice(at), parts, lost())
    }?)
}

/// The channels cleared.
pub fn clear_channels() {
    unsafe {
        crate::audio::clear_channels::clear_channels(crate::audio::voices::World::game().channels)
    };
}

/// `audio::release_primary` on the game's buffer.
pub fn release_primary() -> Result<(), Refused> {
    unsafe {
        crate::audio::sound_device::release_primary(
            primary(),
            game_addr(crate::audio::sound_device::VOLUME_KEPT_RVA) as u32,
        )
    }
}

/// `audio::make_primary` on the game's variables: true when it failed (the carry).
pub fn make_primary_failed() -> Result<bool, Refused> {
    Ok(!unsafe {
        crate::audio::primary::make_primary(&crate::audio::primary::PrimaryVars::game())
    }?)
}

/// `audio::close_primary_mixer` on the game's variables: the carry it leaves.
pub fn close_primary_mixer_in_game() -> Result<bool, Refused> {
    use crate::audio::voices::*;
    unsafe {
        close_primary_mixer(
            game_addr(SOUND_MODE_RVA) as u32,
            game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            crate::session::MACHINE.window.addr() as u32,
        )
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
/// whose +0x3C is 2 and that has a buffer (+8) played again (`play_buffer`).
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
            crate::audio::buffer_play::play_buffer_in_game(crate::audio::voices::voice(v))?;
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
        Self {
            lost: a(BUFFER_LOST_RVA),
            primary: a(PRIMARY_RVA),
            voice_count: a(VOICE_COUNT_RVA),
            voices: a(VOICES_RVA),
            window_active: FLAGS.window_active.addr() as u32,
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
        windows_sys::Win32::System::Threading::Sleep(1000);
        if read(v.window_active) == 0 {
            return Ok(());
        }
        write(v.lost, 0);
        if restore_failed(v.primary)? {
            return Ok(());
        }
        for n in 0..read(v.voice_count) {
            let at = v.voices + n * std::mem::size_of::<crate::audio::voices::Voice>() as u32;
            if restore_failed(at + 8)? || refill_failed(at)? {
                return Ok(());
            }
        }
        crate::audio::thread_parts::play_primary(primary(), lost());
        replay_looping(game_addr(crate::audio::voices::PLAYING_RVA) as u32)?;
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
        clear_channels();
        // Converted: `rc2k_core::audio::buffers::remakes_primary`.
        if !rc2k_core::audio::buffers::remakes_primary(read(mode), || read(direct_sound)) {
            return Ok(false);
        }
        release_primary()?;
        let ds = read(direct_sound);
        let level: unsafe extern "system" fn(u32, u32, u32) -> i32 = std::mem::transmute(
            crate::audio::com::method(ds, crate::audio::sound_device::SET_COOPERATIVE_LEVEL),
        );
        level(ds, read(window), 3);
        make_primary_failed()
    }
}

// ---- for the self-tests ----
