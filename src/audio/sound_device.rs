//! DirectSound's device and primary buffer, given back and asked after (IDA, 2026-10-15):
//! `audio::release_primary`, `read_caps`, `release_a3d`, `close_sound`. Their logging and
//! the calls they make go through a `Part` (the place and eax), so that the tests can stand
//! them in; in the game they are the Rust versions.

use crate::audio::a3d::{open_a3d, open_a3d2};
use crate::errors::{BadInput, Refused};

const CAPS_TEXT: &std::ffi::CStr = c"AUD: sound caps: flgs/pri/min/max:\t"; // "AUD: sound caps: flgs/pri/min/max:\t"
const MAX_HW_TEXT: &std::ffi::CStr = c" maxhw: all/static/stream (free)\t"; // " maxhw: all/static/stream (free)\t"
const MAX_3D_TEXT: &std::ffi::CStr = c" max3d: all/static/stream (free)\t"; // " max3d: all/static/stream (free)\t"
const CLOSE_TEXT: &std::ffi::CStr = c"AUD: close sound"; // "AUD: close sound"

/// A text to the log at the audio's level (2), with a line's end or not.
pub fn log_text(text: &std::ffi::CStr, end: bool) {
    crate::logfile::write_line::log_text(text.to_bytes(), end, 2);
}

/// A number to the log at level 2 as eight hex digits, with a line's end or not.
pub fn log_hex(value: u32, end: bool) {
    crate::logfile::log_hex8::log_hex8(value, end, 2);
}

/// A number to the log at level 2 in decimal, with a line's end or not.
pub fn log_number(value: u32, end: bool) {
    crate::logfile::log_int::log_int(value as i32, end, 2);
}

/// The primary buffer stopped.
pub fn stop_primary() {
    crate::audio::thread_parts::stop_primary();
}

/// The 3D listener released.
pub fn release_listener() {
    crate::audio::thread_parts::release_sound_listener();
}

/// The A3D object released.
pub fn release_a3d_object() {
    release_a3d();
}

/// The primary buffer's format set for `mode` (at the mix rate setting): false when it
/// failed (the original's carry set).
pub fn set_primary_format(mode: u32) -> Result<bool, Refused> {
    let rate = crate::audio::owner::audio().state.mix_rate_setting.get();
    crate::audio::primary::set_primary_format(mode, rate)
}

/// The 3D listener opened: false when it failed (the carry).
pub fn open_listener() -> Result<bool, Refused> {
    crate::audio::primary::open_listener(
        crate::system::owner::system().session.machine.has_fpu.get(),
    )
}

/// DirectSound closed (the game's object and window).
pub fn close_sound_in_game() -> Result<(), Refused> {
    close_sound()
}

/// DirectSound opened (GUID `guid`) with the game's variables: false when it failed.
pub fn open_sound_in_game(guid: u32) -> Result<bool, Refused> {
    open_sound(guid, &mut game_direct_sound_create)
}

/// `audio::release_primary`: unless A3D or no primary buffer: stopped, its kept volume put
/// back (unless 1, then 1), the listener released, the buffer released and forgotten.
pub fn release_primary() -> Result<(), Refused> {
    {
        let p = crate::audio::owner::audio().state.primary_buffer.get();
        if crate::audio::buffer_settings::a3d() || p == 0 {
            return Ok(());
        }
        stop_primary();
        // Converted: `rc2k_core::audio::device::kept_volume`.
        use rc2k_core::audio::device::{NO_KEPT_VOLUME, kept_volume};
        if let Some(volume) =
            kept_volume(crate::audio::owner::audio().state.primary_volume_kept.get())
        {
            crate::platform_sound::set_volume(
                crate::audio::owner::audio().state.primary_buffer.get(),
                volume as i32,
            );
            crate::audio::owner::audio()
                .state
                .primary_volume_kept
                .set(NO_KEPT_VOLUME);
        }
        release_listener();
        crate::platform_sound::release(crate::audio::owner::audio().state.primary_buffer.get());
        crate::audio::owner::audio().state.primary_buffer.set(0);
    }
    Ok(())
}

/// `audio::read_caps`: `GetCaps` into `g_soundCaps` (zeroed, its size 0x60), logged. False
/// for the carry (it failed; nothing logged).
pub fn read_caps() -> Result<bool, Refused> {
    let caps = &crate::audio::owner::audio().state.sound_caps;
    {
        let ds = crate::audio::owner::audio().state.direct_sound.get();
        if ds == 0 {
            return Err(BadInput("no DirectSound to ask (the original calls through 0)").into());
        }
        // `DSCAPS` zeroed, its size, then `GetCaps`'s dwords at +4 .. +0x10.
        for n in (0..0x60).step_by(4) {
            caps.set_u32(n, 0);
        }
        caps.set_u32(0, rc2k_core::audio::device::CAPS_SIZE);
        let got = crate::platform_sound::device_caps();
        for (n, value) in [
            (4, got.flags),
            (8, got.min_rate),
            (0xC, got.max_rate),
            (0x10, got.primary_buffers),
        ] {
            caps.set_u32(n, value);
        }
        // Converted: `rc2k_core::audio::device::CAPS_LOG`.
        use rc2k_core::audio::device::{CAPS_LOG, CapsLog, CapsText};
        for entry in CAPS_LOG {
            let field = |n: u32| caps.get_u32(n as usize);
            match entry {
                CapsLog::Text(text) => {
                    let at = match text {
                        CapsText::Caps => CAPS_TEXT,
                        CapsText::MaxHardware => MAX_HW_TEXT,
                        CapsText::Max3d => MAX_3D_TEXT,
                    };
                    log_text(at, false);
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
pub fn release_a3d() {
    let o = crate::audio::owner::audio().state.a3d_object.get();
    if o != 0 {
        crate::platform_sound::release(o);
        crate::audio::owner::audio().state.a3d_object.set(0);
    }
    crate::audio::owner::audio().state.a3d_ready.set(0);
}

/// `audio::close_sound`: DirectSound's cooperative level 1 (normal), released, forgotten,
/// then `release_a3d`. Refuses A3D (0x40CAD9).
pub fn close_sound() -> Result<(), Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CAD9)").into());
    }
    {
        let ds = crate::audio::owner::audio().state.direct_sound.get();
        if ds != 0 {
            log_text(CLOSE_TEXT, true);
            crate::platform_sound::set_cooperative_level(
                ds,
                crate::system::owner::system().session.machine.window.get(),
                1,
            );
            crate::platform_sound::release(crate::audio::owner::audio().state.direct_sound.get());
            crate::audio::owner::audio().state.direct_sound.set(0);
        }
    }
    release_a3d_object();
    Ok(())
}

const OPEN_TEXT: &std::ffi::CStr = c"AUD: open sound"; // "AUD: open sound"
const OPEN_FAILED_TEXT: &std::ffi::CStr = c"AUD: open failed rc="; // "AUD: open failed rc="

/// `DirectSoundCreate` (the device's GUID) -> the device, or the failure's result.
pub type CreateSound<'a> = &'a mut dyn FnMut(u32) -> Result<u32, i32>;

/// `DirectSoundCreate`: in the standalone program DirectSound on SDL (`platform_sound`),
/// whatever the GUID.
pub fn game_direct_sound_create(_guid: u32) -> Result<u32, i32> {
    crate::platform_sound::create_device()
}

/// `open_sound`'s parts in the game's memory: the mode read and written where the original
/// does, the logs, the platform's DirectSound.
struct Open<'a> {
    create: CreateSound<'a>,
}

impl rc2k_core::audio::device::Opening for Open<'_> {
    type Error = Refused;
    fn mode(&mut self) -> u32 {
        crate::audio::owner::audio().state.sound_mode.get()
    }
    fn set_mode(&mut self, mode: u32) {
        crate::audio::owner::audio().state.sound_mode.set(mode);
    }
    fn log_open(&mut self, guid: u32) -> Result<(), Refused> {
        log_text(OPEN_TEXT, false);
        log_hex(guid, true);
        Ok(())
    }
    fn log_failed(&mut self, code: u32) -> Result<(), Refused> {
        log_text(OPEN_FAILED_TEXT, false);
        log_hex(code, true);
        Ok(())
    }
    // The carry: true when it failed.
    fn open_a3d2(&mut self, _: u32) -> Result<bool, Refused> {
        Ok(!open_a3d2()?)
    }
    fn open_a3d(&mut self, _: u32) -> Result<bool, Refused> {
        Ok(!open_a3d()?)
    }
    fn create(&mut self, guid: u32) -> i32 {
        // A failed `DirectSoundCreate` leaves its out-variable as it was.
        match (self.create)(guid) {
            Ok(device) => {
                crate::audio::owner::audio().state.direct_sound.set(device);
                crate::platform_sound::OK
            }
            Err(r) => r,
        }
    }
    fn cooperate(&mut self) -> i32 {
        let ds = crate::audio::owner::audio().state.direct_sound.get();
        crate::platform_sound::set_cooperative_level(
            ds,
            crate::system::owner::system().session.machine.window.get(),
            3,
        )
    }
}

/// `audio::open_sound`: "AUD: open sound" and the GUID logged; mode bit 0x2000:
/// `open_a3d2`, which done is all; failing, the mode's 0x3000 bits flipped. Then only
/// with bit 0x1000: `open_a3d`, failing which `DirectSoundCreate`; cooperative level 3.
/// A failure (or no bit 0x1000) logs "AUD: open failed rc=" and a code (the GUID, or the
/// call's result) and gives false (the carry).
pub fn open_sound(guid: u32, create: CreateSound) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::open_sound`.
    rc2k_core::audio::device::open_sound(guid, &mut Open { create })
}

/// `audio::reopen_sound`: when the mode's 0xF000 bits differ from those DirectSound was
/// opened with: `close_sound` and `open_sound` (GUID 0). Returns the carry (that
/// of `open_sound`, when it ran).
pub fn reopen_sound() -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::needs_reopen`.
    if !rc2k_core::audio::device::needs_reopen(
        crate::audio::owner::audio().state.sound_mode.get(),
        crate::audio::owner::audio().state.opened_sound_mode.get(),
    ) {
        return Ok(false);
    }
    close_sound_in_game()?;
    Ok(!open_sound_in_game(0)?)
}
