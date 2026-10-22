//! `audio::close_all` (0x409DD5): everything of the sound closed (the audio thread's end).
//!
//! In:  ebx, ecx, edx = handed to `audio::stop_streams` (edx its action)
//! Out: every register preserved
//!
//! `audio::stop_streams`, `audio::free_voices`, `audio::close_way`,
//! `audio::release_primary`, `audio::close_sound`, then `g_soundMode` 0. One caller, the
//! audio thread. Shadow mode runs the original alone. Self-test: a redirected copy whose
//! five calls go to stand-ins (the first noting ebx, ecx, edx), 2 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr};

/// Its calls in order, each with the registers it gets.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub fn close_all(r: &Regs, mode: u32, part: Part) -> Result<(), Refused> {
    for place in 0..5 {
        let mut c = *r;
        part(place, &mut c)?;
    }
    unsafe { write(mode, 0) };
    let _ = read;
    Ok(())
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::audio::sound_device::*;
    use crate::audio::voices::*;
    unsafe {
        match place {
            0 => crate::audio::stop_streams::stop_streams(
                r,
                &mut crate::audio::stop_streams::game_command,
            )?,
            1 => crate::audio::free_voices::free_voices(
                &World::game(),
                &crate::audio::free_voices::game_vars(),
                &mut crate::audio::free_voices::game_part,
            )?,
            2 => {
                crate::audio::mixer_set_up::close_way(
                    crate::framework::game_read(SOUND_MODE_RVA),
                    &mut crate::audio::restore::game_part,
                )?;
            }
            3 => release_primary(
                game_addr(PRIMARY_RVA) as u32,
                game_addr(VOLUME_KEPT_RVA) as u32,
                &mut crate::audio::sound_device::game_part,
            )?,
            _ => close_sound(
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(WINDOW_RVA) as u32,
                &mut crate::audio::sound_device::game_part,
            )?,
        }
    }
    Ok(())
}
