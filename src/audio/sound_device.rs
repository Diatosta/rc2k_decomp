//! DirectSound's device and primary buffer, given back and asked after (IDA, 2026-10-15):
//! `audio::release_primary`, `read_caps`, `release_a3d`, `close_sound`. Their logging and
//! the calls they make go through a `Part` (the place and eax), so that the tests can stand
//! them in; in the game they are the Rust versions.

use crate::audio::store::Slot;
use crate::audio::voices::{read, write};
use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::session::MACHINE;
use crate::store::game_addr;

pub const VOLUME_KEPT_RVA: Slot = Slot::at(0x4F82DC); // g_primaryVolumeKept (1: none)
pub const CAPS_RVA: Slot = Slot::at(0x506AA8); // g_soundCaps (DSCAPS, 0x60 bytes)
pub const A3D_OBJECT_RVA: Slot = Slot::at(0x506C68); // g_a3dObject
pub const A3D_READY_RVA: Slot = Slot::at(0x506C64); // g_a3dReady
const CAPS_TEXT: usize = rva(0x4DA645); // "AUD: sound caps: flgs/pri/min/max:\t"
const MAX_HW_TEXT: usize = rva(0x4DA669); // " maxhw: all/static/stream (free)\t"
const MAX_3D_TEXT: usize = rva(0x4DA68B); // " max3d: all/static/stream (free)\t"
const CLOSE_TEXT: usize = rva(0x4DA634); // "AUD: close sound"
pub const GET_CAPS: u32 = 0x10;
pub const SET_COOPERATIVE_LEVEL: u32 = 0x18;

/// A text to the log at the audio's level (2), with a line's end or not.
pub fn log_text(text: u32, end: bool) {
    unsafe { crate::logfile::log_write::log_write(text as *const u8, end, 2) };
}

/// A number to the log at level 2 as eight hex digits, with a line's end or not.
pub fn log_hex(value: u32, end: bool) {
    crate::logfile::log_hex8::log_hex8(value, end, 2);
}

/// A number to the log at level 2 in decimal, with a line's end or not.
pub fn log_number(value: u32, end: bool) {
    crate::logfile::log_int::log_int(value as i32, end, 2);
}

/// A text to the log at level 1 with a line's end.
pub fn log_text_1(text: u32) {
    unsafe { crate::logfile::log_write::log_write(text as *const u8, true, 1) };
}

fn primary() -> u32 {
    game_addr(crate::audio::voices::PRIMARY_RVA) as u32
}

fn listener() -> u32 {
    game_addr(crate::audio::buffer_play::LISTENER_3D_RVA) as u32
}

/// The primary buffer stopped.
pub fn stop_primary() {
    unsafe { crate::audio::thread_parts::stop_primary(primary()) };
}

/// The 3D listener released.
pub fn release_listener() {
    unsafe { crate::audio::thread_parts::release_sound_listener(listener()) };
}

/// The A3D object released.
pub fn release_a3d_object() {
    unsafe {
        release_a3d(
            game_addr(A3D_OBJECT_RVA) as u32,
            game_addr(A3D_READY_RVA) as u32,
        )
    };
}

/// The primary buffer's format set for `mode` (at the mix rate setting): false when it
/// failed (the original's carry set).
pub fn set_primary_format(mode: u32) -> Result<bool, Refused> {
    let rate = unsafe { crate::store::game_read(crate::audio::primary::MIX_RATE_SETTING_RVA) };
    unsafe { crate::audio::primary::set_primary_format(primary(), mode, rate) }
}

/// The 3D listener opened: false when it failed (the carry).
pub fn open_listener() -> Result<bool, Refused> {
    use crate::audio::primary::*;
    unsafe {
        open_listener(
            primary(),
            listener(),
            MACHINE.has_fpu.addr() as u32,
            game_addr(NO_LISTENER_RVA) as u32,
        )
    }
}

/// DirectSound closed (the game's object and window).
pub fn close_sound_in_game() -> Result<(), Refused> {
    unsafe {
        close_sound(
            game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            MACHINE.window.addr() as u32,
        )
    }
}

/// DirectSound opened (GUID `guid`) with the game's variables: false when it failed.
pub fn open_sound_in_game(guid: u32) -> Result<bool, Refused> {
    unsafe { open_sound(guid, &OpenVars::game(), &mut game_direct_sound_create) }
}

/// `audio::release_primary`: unless A3D or no primary buffer: stopped, its kept volume put
/// back (unless 1, then 1), the listener released, the buffer released and forgotten.
pub unsafe fn release_primary(primary: u32, kept: u32) -> Result<(), Refused> {
    unsafe {
        let p = read(primary);
        if crate::audio::buffer_settings::a3d() || p == 0 {
            return Ok(());
        }
        stop_primary();
        // Converted: `rc2k_core::audio::device::kept_volume`.
        use rc2k_core::audio::device::{NO_KEPT_VOLUME, kept_volume};
        if let Some(volume) = kept_volume(read(kept)) {
            crate::audio::com::call1(
                read(primary),
                crate::audio::buffer_settings::SET_VOLUME,
                volume,
            );
            write(kept, NO_KEPT_VOLUME);
        }
        release_listener();
        crate::audio::com::call(read(primary), crate::audio::com::RELEASE);
        write(primary, 0);
    }
    Ok(())
}

/// `audio::read_caps`: `GetCaps` into `caps` (zeroed, its size 0x60), logged. False for
/// the carry (it failed; nothing logged).
pub unsafe fn read_caps(direct_sound: u32, caps: u32) -> Result<bool, Refused> {
    unsafe {
        let ds = read(direct_sound);
        if ds == 0 {
            return Err(BadInput("no DirectSound to ask (the original calls through 0)").into());
        }
        std::ptr::write_bytes(caps as *mut u8, 0, 0x60);
        write(caps, rc2k_core::audio::device::CAPS_SIZE);
        if crate::audio::com::call1(ds, GET_CAPS, caps) != 0 {
            return Ok(false);
        }
        // Converted: `rc2k_core::audio::device::CAPS_LOG`.
        use rc2k_core::audio::device::{CAPS_LOG, CapsLog, CapsText};
        for entry in CAPS_LOG {
            let field = |n: u32| read(caps + n);
            match entry {
                CapsLog::Text(text) => {
                    let at = match text {
                        CapsText::Caps => CAPS_TEXT,
                        CapsText::MaxHardware => MAX_HW_TEXT,
                        CapsText::Max3d => MAX_3D_TEXT,
                    };
                    log_text(game_addr(at) as u32, false)
                }
                CapsLog::Hex(n) => log_hex(field(n), false),
                CapsLog::Int(n) => log_number(field(n), false),
                CapsLog::IntLine(n) => log_number(field(n), true),
            };
        }
        Ok(true)
    }
}

/// `audio::release_a3d`: the A3D object released and forgotten; not ready.
pub unsafe fn release_a3d(object: u32, ready: u32) {
    unsafe {
        let o = read(object);
        if o != 0 {
            crate::audio::com::call(o, crate::audio::com::RELEASE);
            write(object, 0);
        }
        write(ready, 0);
    }
}

/// `audio::close_sound`: DirectSound's cooperative level 1 (normal), released, forgotten,
/// then `release_a3d`. Refuses A3D (0x40CAD9).
pub unsafe fn close_sound(direct_sound: u32, window: u32) -> Result<(), Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CAD9)").into());
    }
    unsafe {
        let ds = read(direct_sound);
        if ds != 0 {
            log_text(game_addr(CLOSE_TEXT) as u32, true);
            let level: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, SET_COOPERATIVE_LEVEL));
            level(ds, read(window), 1);
            crate::audio::com::call(read(direct_sound), crate::audio::com::RELEASE);
            write(direct_sound, 0);
        }
        release_a3d_object();
    }
    Ok(())
}

pub const OPENED_MODE_RVA: Slot = Slot::at(0x4F82C0); // g_openedSoundMode
const OPEN_TEXT: usize = rva(0x4DA60F); // "AUD: open sound"
const OPEN_FAILED_TEXT: usize = rva(0x4DA61F); // "AUD: open failed rc="

/// `DirectSoundCreate` (the device's GUID, where the object goes) -> its result.
pub type CreateSound<'a> = &'a mut dyn FnMut(u32, u32) -> i32;

/// `DirectSoundCreate`: in the standalone program DirectSound on SDL (`platform_sound`).
pub fn game_direct_sound_create(guid: u32, out: u32) -> i32 {
    let _ = guid;
    crate::platform_sound::direct_sound_create(out)
}

/// Where `open_sound`'s things are.
#[derive(Clone, Copy)]
pub struct OpenVars {
    pub mode: u32,
    pub direct_sound: u32,
    pub window: u32,
}

impl OpenVars {
    pub fn game() -> Self {
        Self {
            mode: game_addr(crate::audio::voices::SOUND_MODE_RVA) as u32,
            direct_sound: game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            window: MACHINE.window.addr() as u32,
        }
    }
}

/// `open_sound`'s parts in the game's memory: the mode read and written where the original
/// does, the logs, the platform's DirectSound.
struct Open<'a> {
    v: &'a OpenVars,
    create: CreateSound<'a>,
}

impl rc2k_core::audio::device::Opening for Open<'_> {
    type Error = Refused;
    fn mode(&mut self) -> u32 {
        unsafe { read(self.v.mode) }
    }
    fn set_mode(&mut self, mode: u32) {
        unsafe { write(self.v.mode, mode) };
    }
    fn log_open(&mut self, guid: u32) -> Result<(), Refused> {
        log_text(game_addr(OPEN_TEXT) as u32, false);
        log_hex(guid, true);
        Ok(())
    }
    fn log_failed(&mut self, code: u32) -> Result<(), Refused> {
        log_text(game_addr(OPEN_FAILED_TEXT) as u32, false);
        log_hex(code, true);
        Ok(())
    }
    // The carry: true when it failed.
    fn open_a3d2(&mut self, _: u32) -> Result<bool, Refused> {
        use crate::audio::a3d::*;
        Ok(!unsafe { open_a3d2(&A3dVars::game(), &mut game_create) }?)
    }
    fn open_a3d(&mut self, _: u32) -> Result<bool, Refused> {
        use crate::audio::a3d::*;
        Ok(!unsafe { open_a3d(&A3dVars::game(), &mut game_create) }?)
    }
    fn create(&mut self, guid: u32) -> i32 {
        (self.create)(guid, self.v.direct_sound)
    }
    fn cooperate(&mut self) -> i32 {
        unsafe {
            let ds = read(self.v.direct_sound);
            let level: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, SET_COOPERATIVE_LEVEL));
            level(ds, read(self.v.window), 3)
        }
    }
}

/// `audio::open_sound`: "AUD: open sound" and the GUID logged; mode bit 0x2000:
/// `open_a3d2`, which done is all; failing, the mode's 0x3000 bits flipped. Then only
/// with bit 0x1000: `open_a3d`, failing which `DirectSoundCreate`; cooperative level 3.
/// A failure (or no bit 0x1000) logs "AUD: open failed rc=" and a code (the GUID, or the
/// call's result) and gives false (the carry).
pub unsafe fn open_sound(guid: u32, v: &OpenVars, create: CreateSound) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::open_sound`.
    rc2k_core::audio::device::open_sound(guid, &mut Open { v, create })
}

/// `audio::reopen_sound`: when the mode's 0xF000 bits differ from those DirectSound was
/// opened with: `close_sound` and `open_sound` (GUID 0). Returns the carry (that
/// of `open_sound`, when it ran).
pub unsafe fn reopen_sound(mode: u32, opened: u32) -> Result<bool, Refused> {
    unsafe {
        // Converted: `rc2k_core::audio::device::needs_reopen`.
        if !rc2k_core::audio::device::needs_reopen(read(mode), read(opened)) {
            return Ok(false);
        }
        close_sound_in_game()?;
        Ok(!open_sound_in_game(0)?)
    }
}
