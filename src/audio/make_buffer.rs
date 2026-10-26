//! `audio::make_buffer` (0x40B5CF): a voice's DirectSound buffer made.
//!
//! In:  esi = the voice
//! Out: carry = not made; every register preserved
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
//! `audio::set_eax_room` (the openings read through ebx, which is the caller's, the
//! frequency after the create path, or the 3D buffer's place after the voice manager's
//! call, as the original leaves it; `open_eax` refuses one that is no pointer). Callers:
//! command 3 and `audio::start`.
//! Refuses A3D (the chunk at 0x40CC88), and what the settings refuse, checked first.
//! Shadow mode runs the original alone. Self-test: a redirected copy on a DirectSound of
//! the test's (`audio/test_sound.rs`) whose 3D calls and logging note their registers, 48
//! cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::buffer_settings::{a3d, set_buffer_frequency, set_buffer_pan, set_buffer_volume};
use crate::audio::describe_wave_format::{WAVE_FORMAT_RVA, describe_wave_format};
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};

pub const DIRECT_SOUND_RVA: usize = rva(0x506A88); // g_directSound
pub const DESCRIPTION_RVA: usize = rva(0x506B1C); // g_bufferDescription
const IID_3D_RVA: usize = rva(0x506A58); // g_iidSound3dBuffer
const CREATE_FAILED_RVA: usize = rva(0x4DA6E4); // "AUD: 1-DSBCreate failed"
const QUERY_FAILED_RVA: usize = rva(0x4DA6FC); // "AUD: 1-DSBQuery3D failed"
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
    pub format: u32,
}

/// Where its variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub mode: u32,
    pub direct_sound: u32,
    pub description: u32,
    pub commit: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            mode: game_addr(SOUND_MODE_RVA) as u32,
            direct_sound: game_addr(DIRECT_SOUND_RVA) as u32,
            description: game_addr(DESCRIPTION_RVA) as u32,
            commit: game_addr(COMMIT_RVA) as u32,
        }
    }
}

/// The parts it calls by the game's addresses: 0 the place, 1 the minimum distance,
/// 2 `sub_40C6B5`, 3 `sub_40D27F` (true: carry), 4 `sub_40D26B`; 5 a text to the log,
/// 6 a number to the log with a line's end.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<bool, Refused>;

/// True for the carry clear. `ebx`: the caller's, which the original leaves in ebx (or
/// replaces, as below) when it calls the EAX parts: they read through it.
pub unsafe fn make_buffer(at: u32, ebx: u32, vars: &Vars, part: Part) -> Result<bool, Refused> {
    unsafe {
        if a3d() {
            return Err(BadInput("A3D (the chunk at 0x40CC88)").into());
        }
        let mode = read(vars.mode);
        if mode & MODE_OWN_BUFFERS == 0 {
            return Ok(true);
        }
        let ds = read(vars.direct_sound);
        if ds == 0 {
            return Ok(false);
        }
        let v = voice(at);
        // What the settings would refuse, before anything is made.
        let level = (v.volume as i32).wrapping_mul(v.volume_factor as i32) >> 6;
        if !(0..=0x40).contains(&level) || !(-0x40..=0x40).contains(&v.pan) {
            return Err(BadInput("a volume or pan past the volume table").into());
        }
        // ebx as the original has it at each call: the caller's, the frequency after the
        // create path (`mov ebx, [esi+60h]`), the 3D buffer's place after `sub_40C6B5`'s.
        let mut regs = Regs {
            esi: at,
            ebx,
            ..Regs::default()
        };
        let mut made = false;
        if v.source != 0 {
            let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, 0x14));
            if f(ds, voice(v.source).buffer, &mut v.buffer as *mut u32 as u32) == 0 {
                made = true;
            } else {
                v.source = 0;
            }
        }
        if !made {
            regs.ebx = v.frequency;
            describe_wave_format(v.format, v.frequency)?;
            // Converted: `rc2k_core::audio::voices::buffer_flags`.
            let flags = rc2k_core::audio::voices::buffer_flags(mode, v.flags);
            (vars.description as *mut BufferDescription).write_unaligned(BufferDescription {
                size: 0x14,
                flags,
                bytes: v.length,
                reserved: 0,
                format: game_addr(WAVE_FORMAT_RVA) as u32,
            });
            let f: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, 0x0C));
            let r = f(ds, vars.description, &mut v.buffer as *mut u32 as u32, 0);
            if r != 0 {
                return log_failure(part, CREATE_FAILED_RVA, r);
            }
        }
        set_buffer_volume(v)?;
        set_buffer_pan(v)?;
        set_buffer_frequency(v)?;
        if mode & MODE_3D != 0 {
            let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(v.buffer, 0));
            let r = f(
                v.buffer,
                game_addr(IID_3D_RVA) as u32,
                &mut v.buffer_3d as *mut u32 as u32,
            );
            if r != 0 {
                return log_failure(part, QUERY_FAILED_RVA, r);
            }
            part(0, &mut regs)?;
            part(1, &mut regs)?;
            write(vars.commit, 1);
            if mode & MODE_HARDWARE_3D != 0 {
                regs.ebx = &mut v.buffer_3d as *mut u32 as u32;
                let mut r = regs;
                part(2, &mut r)?;
            }
            if mode & MODE_EAX != 0 {
                let mut r = regs;
                if !part(3, &mut r)? {
                    let mut r = regs;
                    part(4, &mut r)?;
                }
            }
        }
        Ok(true)
    }
}

fn log_failure(part: Part, text: usize, result: i32) -> Result<bool, Refused> {
    let mut r = Regs {
        eax: game_addr(text) as u32,
        ..Regs::default()
    };
    part(5, &mut r)?;
    let mut r = Regs {
        eax: result as u32,
        ..Regs::default()
    };
    part(6, &mut r)?;
    Ok(false)
}

/// The game's parts: the Rust versions (the carry of 3 returned).
pub fn game_part(which: u32, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        match which {
            0 => {
                crate::audio::buffer_settings::set_buffer_place(voice(r.esi))?;
            }
            1 => {
                let v = &*(r.esi as *const Voice);
                r.eax = crate::audio::set_buffer_min_distance::set_buffer_min_distance(v)? as u32;
            }
            2 => {
                crate::audio::open_voice_manager::open_voice_manager(r.ebx)?;
            }
            3 => return Ok(!crate::audio::open_eax::open_eax(r.ebx)?),
            4 => r.eax = crate::audio::set_eax_room::in_game()?,
            5 => crate::logfile::log_write::log_write(r.eax as *const u8, false, 1),
            _ => crate::logfile::log_hex8::log_hex8(r.eax, true, 1),
        }
    }
    Ok(false)
}
