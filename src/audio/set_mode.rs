//! `audio::set_mode` (0x40A426): command 5, the sound mode changed (menus <-> a race).
//!
//! In:  eax = the new mode; ebx, ecx = handed to `audio::stop_streams`, with action 0 (the
//!      original hands on edx: section 16, fixed 2026-10-20)
//! Out: carry = failed ("SAM: FAIL"); every register preserved
//!
//! "SAM:" and the mode logged; `audio::stop_streams`, `audio::free_voices`,
//! `audio::close_way`, `audio::release_primary`; `g_soundMode` the new mode; the channels:
//! 8 (0x10 with bit 0x400000), held to the free hardware buffers of the mode's kind (2D or
//! 3D: bit 0x10000 all, 0x20000 the other count of `g_soundCaps`) when there are any, and
//! at least 6, into `g_channelCount`; `audio::reopen_sound`, `audio::make_primary` (either
//! failing: `g_openedSoundMode` 0, "SAM: FAIL", the carry), `audio::open_way`, a second's
//! `system::sleep`, `audio::play_primary`; `g_openedSoundMode` the mode, "SAM: OK". Either
//! way `system::restore_window` last (its flags kept around it). Callers: the audio thread's
//! start (the menus' mode), `audio::take_commands` (command 5).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::audio::buffer_play::{Leaves, leaf_entry};
use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

const SAM_TEXT: usize = rva(0x4DA6CD); // "SAM:"
const OK_TEXT: usize = rva(0x4DA6D2); // "SAM: OK"
const FAIL_TEXT: usize = rva(0x4DA6DA); // "SAM: FAIL"

#[derive(Clone, Copy)]
pub struct ModeVars {
    pub mode: u32,
    /// `g_soundCaps` (free hardware buffers: +0x24 all, +0x28 static, +0x3C 3D all,
    /// +0x40 3D static).
    pub caps: u32,
    pub channel_count: u32,
    pub opened: u32,
}

impl ModeVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            mode: a(crate::audio::voices::SOUND_MODE_RVA),
            caps: a(crate::audio::sound_device::CAPS_RVA),
            channel_count: a(crate::audio::voices::CHANNEL_COUNT_RVA),
            opened: a(crate::audio::sound_device::OPENED_MODE_RVA),
        }
    }
}

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// True for no carry.
pub fn set_mode(entry: &Regs, v: &ModeVars, part: Part) -> Result<bool, Refused> {
    unsafe {
        let mode = entry.eax;
        let mut c = *entry;
        let mut call = |place: u32, c: &Regs| -> Result<bool, Refused> {
            let mut r = *c;
            part(place, &mut r)
        };
        call(
            0,
            &Regs {
                eax: game_addr(SAM_TEXT) as u32,
                ..c
            },
        )?;
        call(1, &c)?;
        // Fixed (section 16, 2026-10-20): the streams stopped with action 0; the original
        // hands on whatever edx holds (normally the thread's 1, a feed), so the music
        // outlived the DirectSound it played on.
        call(2, &Regs { edx: 0, ..c })?;
        for place in 3..6 {
            call(place, &c)?;
        }
        c.edx = mode;
        write(v.mode, mode);
        // Converted: `rc2k_core::audio::modes::channels`.
        let (channels, looked) = rc2k_core::audio::modes::channels(
            mode,
            [read(v.caps + 0x24), read(v.caps + 0x28)],
            [read(v.caps + 0x3C), read(v.caps + 0x40)],
        );
        c.ebx = looked;
        c.eax = channels;
        write(v.channel_count, c.eax);
        let ok = !call(6, &c)? && !call(7, &c)?;
        if ok {
            call(8, &c)?;
            call(9, &Regs { eax: 1000, ..c })?;
            call(10, &c)?;
            write(v.opened, read(v.mode));
            call(
                11,
                &Regs {
                    eax: game_addr(OK_TEXT) as u32,
                    ..c
                },
            )?;
        } else {
            write(v.opened, 0);
            call(
                11,
                &Regs {
                    eax: game_addr(FAIL_TEXT) as u32,
                    ..c
                },
            )?;
        }
        call(12, &c)?;
        Ok(ok)
    }
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<bool, Refused> {
    use crate::audio::sound_device as sd;
    use crate::audio::voices::*;
    let mode = || unsafe { crate::framework::game_read::<u32>(SOUND_MODE_RVA) };
    unsafe {
        Ok(match place {
            0 => sd::game_part(0, r.eax)?,
            1 => sd::game_part(3, r.eax)?,
            11 => sd::game_part(1, r.eax)?,
            2 => {
                crate::audio::stop_streams::stop_streams(
                    r,
                    &mut crate::audio::stop_streams::game_command,
                )?;
                false
            }
            3 => {
                crate::audio::free_voices::free_voices(
                    &World::game(),
                    &crate::audio::free_voices::game_vars(),
                    &mut crate::audio::free_voices::game_part,
                )?;
                false
            }
            4 => crate::audio::mixer_set_up::close_way(
                mode(),
                &mut crate::audio::restore::game_part,
            )?,
            5 => {
                sd::release_primary(
                    game_addr(PRIMARY_RVA) as u32,
                    game_addr(sd::VOLUME_KEPT_RVA) as u32,
                    &mut sd::game_part,
                )?;
                false
            }
            6 => sd::reopen_sound(
                game_addr(SOUND_MODE_RVA) as u32,
                game_addr(sd::OPENED_MODE_RVA) as u32,
                &mut sd::game_part,
            )?,
            7 => !crate::audio::primary::make_primary(
                &crate::audio::primary::PrimaryVars::game(),
                &mut sd::game_part,
            )?,
            8 => {
                crate::audio::mixer_set_up::open_way(mode(), &mut crate::audio::restore::game_part)?
            }
            9 => {
                crate::system::sleep::sleep(r.eax);
                false
            }
            10 => {
                crate::audio::thread_parts::play_primary(
                    game_addr(PRIMARY_RVA) as u32,
                    game_addr(BUFFER_LOST_RVA) as u32,
                );
                false
            }
            _ => {
                crate::system::restore_window::restore_window();
                false
            }
        })
    }
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = set_mode(r, &ModeVars::game(), &mut game_part)?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
        zero: None,
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    leaf_entry(r, rust)
}
