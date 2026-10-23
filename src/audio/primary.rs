//! The primary buffer made (IDA, 2026-10-15): `audio::make_primary` and its parts
//! `volume_index`, `set_primary_format`, `open_listener`. Logging and calls go through
//! `sound_device`'s named calls.

use crate::audio::voices::{read, write};
use crate::errors::Refused;
use crate::framework::{game_addr, game_read, rva};

pub const MIX_RATE_SETTING_RVA: usize = rva(0x4F82D4); // g_mixRateSetting
pub const HAS_FPU_RVA: usize = rva(0x5183EC); // g_hasFpu (set by `system::test_processor` from `cpuid`)
pub const NO_LISTENER_RVA: usize = rva(0x506B42); // an unaligned dword: 1 = no listener
pub const IID_LISTENER_RVA: usize = rva(0x506A48); // g_iidListener3d
pub const MASTER_SHOWN_RVA: usize = rva(0x755684); // g_masterVolumeShown
const OPEN_TEXT: usize = rva(0x4DA715); // "AUD: open ds3d"
const FAILED_TEXT: usize = rva(0x4DA724); // "AUD: open ds3d failed rc="
pub const SET_FORMAT: u32 = 0x38;
pub const GET_VOLUME: u32 = 0x18;
const MODE_3D: u32 = crate::audio::voices::MODE_3D;

/// `audio::volume_index`: the first index of `g_volumeTable` whose volume is at least
/// `volume` (signed), or 0x40.
pub fn volume_index(volume: i32) -> u32 {
    // Converted: `rc2k_core::audio::samples::volume_index`.
    let table: [i32; 0x40] = unsafe { game_read(crate::audio::voices::VOLUMES_RVA) };
    rc2k_core::audio::samples::volume_index(volume, &table)
}

/// `audio::set_primary_format`: `describe_wave_format` (the mode, `rate`), `SetFormat`
/// (+0x38). False for the carry (no primary buffer, or it failed).
pub unsafe fn set_primary_format(primary: u32, mode: u32, rate: u32) -> Result<bool, Refused> {
    unsafe {
        let p = read(primary);
        if p == 0 {
            return Ok(false);
        }
        crate::audio::describe_wave_format::describe_wave_format(mode, rate)?;
        let format = game_addr(crate::audio::describe_wave_format::WAVE_FORMAT_RVA) as u32;
        Ok(crate::audio::com::call1(p, SET_FORMAT, format) == 0)
    }
}

/// `audio::open_listener`: the 3D listener from the primary buffer (`QueryInterface`
/// with `g_iidListener3d` into `listener`) when `g_hasFpu` is 1 and the dword at
/// 0x506B42 is not; else, or failing, logged and `release_sound_listener`. False for the
/// carry.
pub unsafe fn open_listener(
    primary: u32,
    listener: u32,
    setting: u32,
    no_listener: u32,
) -> Result<bool, Refused> {
    unsafe {
        crate::audio::sound_device::log_text(game_addr(OPEN_TEXT) as u32, true);
        // Converted: `rc2k_core::audio::device::listener_refused`.
        let code =
            match rc2k_core::audio::device::listener_refused(read(setting), || read(no_listener)) {
                Some(code) => code,
                None => {
                    let p = read(primary);
                    let query: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                        std::mem::transmute(crate::audio::com::method(p, 0));
                    let r = query(p, game_addr(IID_LISTENER_RVA) as u32, listener);
                    if r == 0 {
                        return Ok(true);
                    }
                    r as u32
                }
            };
        crate::audio::sound_device::log_text(game_addr(FAILED_TEXT) as u32, false);
        crate::audio::sound_device::log_hex(code, true);
        crate::audio::sound_device::release_listener();
        Ok(false)
    }
}

/// Where `make_primary`'s things are.
#[derive(Clone, Copy)]
pub struct PrimaryVars {
    pub direct_sound: u32,
    pub primary: u32,
    pub kept: u32,
    pub mode: u32,
    pub master: u32,
    pub shown: u32,
}

impl PrimaryVars {
    pub fn game() -> Self {
        use crate::audio::voices::*;
        let a = |r| game_addr(r) as u32;
        Self {
            direct_sound: a(crate::audio::make_buffer::DIRECT_SOUND_RVA),
            primary: a(PRIMARY_RVA),
            kept: a(crate::audio::sound_device::VOLUME_KEPT_RVA),
            mode: a(SOUND_MODE_RVA),
            master: a(MASTER_VOLUME_RVA),
            shown: a(MASTER_SHOWN_RVA),
        }
    }
}

/// `audio::make_primary`: nothing with A3D; no DirectSound or a failed create: false (the
/// carry). `g_bufferDescription` zeroed, size 0x14, flags 0x81 (+0x10 in 3D);
/// `CreateSoundBuffer` into the primary; `GetVolume` (+0x18) into the kept volume;
/// `set_primary_format` (place 10, eax the mode; its carry not looked at); a master volume
/// past 0x40 becomes `volume_index` of the kept one; it is shown and set
/// (`set_master_volume`, place 12, eax); in 3D `open_listener` (place 13), whose failure
/// takes 3D out of the mode.
pub unsafe fn make_primary(v: &PrimaryVars) -> Result<bool, Refused> {
    use crate::audio::make_buffer::{BufferDescription, DESCRIPTION_RVA};
    unsafe {
        if crate::audio::buffer_settings::a3d() {
            return Ok(true);
        }
        let ds = read(v.direct_sound);
        if ds == 0 {
            return Ok(false);
        }
        // Converted: `rc2k_core::audio::device` (`primary_flags`, `master_volume`,
        // `without_3d`).
        use rc2k_core::audio::device::{master_volume, primary_flags, without_3d};
        let description = game_addr(DESCRIPTION_RVA) as *mut BufferDescription;
        description.write_unaligned(BufferDescription {
            size: 0x14,
            flags: primary_flags(read(v.mode)),
            ..BufferDescription::default()
        });
        let create: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(ds, 0x0C));
        if create(ds, description as u32, v.primary, 0) != 0 {
            return Ok(false);
        }
        crate::audio::com::call1(read(v.primary), GET_VOLUME, v.kept);
        crate::audio::sound_device::set_primary_format(read(v.mode))?;
        let (volume, stored) = master_volume(read(v.master), || volume_index(read(v.kept) as i32));
        if stored {
            write(v.master, volume);
        }
        write(v.shown, volume);
        crate::audio::set_master_volume::set_master_volume_in_game(volume);
        if read(v.mode) & MODE_3D != 0 && !crate::audio::sound_device::open_listener()? {
            write(v.mode, without_3d(read(v.mode)));
        }
        Ok(true)
    }
}
