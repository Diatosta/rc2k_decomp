//! DirectSound's device and primary buffer, given back and asked after (IDA, 2026-10-15):
//! `audio::release_primary`, `read_caps`, `release_a3d`, `close_sound`. Their logging and
//! the calls they make go through a `Part` (the place and eax), so that the tests can stand
//! them in; in the game they are the Rust versions.

use crate::audio::voices::{read, write};
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};

pub const VOLUME_KEPT_RVA: usize = rva(0x4F82DC); // g_primaryVolumeKept (1: none)
pub const CAPS_RVA: usize = rva(0x506AA8); // g_soundCaps (DSCAPS, 0x60 bytes)
pub const A3D_OBJECT_RVA: usize = rva(0x506C68); // g_a3dObject
pub const A3D_READY_RVA: usize = rva(0x506C64); // g_a3dReady
pub const WINDOW_RVA: usize = rva(0x5184A4); // hWnd
const CAPS_TEXT: usize = rva(0x4DA645); // "AUD: sound caps: flgs/pri/min/max:\t"
const MAX_HW_TEXT: usize = rva(0x4DA669); // " maxhw: all/static/stream (free)\t"
const MAX_3D_TEXT: usize = rva(0x4DA68B); // " max3d: all/static/stream (free)\t"
const CLOSE_TEXT: usize = rva(0x4DA634); // "AUD: close sound"
pub const GET_CAPS: u32 = 0x10;
pub const SET_COOPERATIVE_LEVEL: u32 = 0x18;

/// The places of a `Part`: the log entries the functions call (by address), then the three
/// calls.
pub const CALLEES: [usize; 18] = [
    0x401432, // 0 log_write_l2
    0x401439, // 1 log_write_l2_nl
    0x40162D, // 2 log_hex8_l2
    0x401634, // 3 log_hex8_l2_nl
    0x40147D, // 4 log_int_l2
    0x401484, // 5 log_int_l2_nl
    0x40142B, // 6 log_write_l1_nl
    0x40A7A9, // 7 audio::stop_primary
    0x40C655, // 8 audio::release_sound_listener
    0x40C8A2, // 9 audio::release_a3d
    0x40A677, // 10 audio::set_primary_format (eax the mode)
    0,        // 11 (`audio::volume_index` is called as it is)
    0x40A740, // 12 audio::set_master_volume (eax)
    0x40C5F9, // 13 audio::open_listener
    0x40CA17, // 14 audio::open_a3d2 (eax the GUID)
    0x40C76D, // 15 audio::open_a3d (eax the GUID)
    0x409EA5, // 16 audio::close_sound
    0x409E1D, // 17 audio::open_sound (eax the GUID)
];
/// (place, eax) -> the carry it leaves (only 10 and 13 have one).
pub type Part<'a> = &'a mut dyn FnMut(u32, u32) -> Result<bool, Refused>;

/// The Rust versions, as the game has them.
pub fn game_part(place: u32, eax: u32) -> Result<bool, Refused> {
    use crate::audio::voices::PRIMARY_RVA;
    use crate::logfile::{log_hex8::log_hex8, log_int::log_int, log_write::log_write};
    let primary = game_addr(PRIMARY_RVA) as u32;
    let listener = game_addr(crate::audio::buffer_play::LISTENER_3D_RVA) as u32;
    unsafe {
        match place {
            0 | 1 => log_write(eax as *const u8, place == 1, 2),
            2 | 3 => log_hex8(eax, place == 3, 2),
            4 | 5 => log_int(eax as i32, place == 5, 2),
            6 => log_write(eax as *const u8, true, 1),
            7 => crate::audio::thread_parts::stop_primary(primary),
            8 => crate::audio::thread_parts::release_sound_listener(listener),
            9 => release_a3d(
                game_addr(A3D_OBJECT_RVA) as u32,
                game_addr(A3D_READY_RVA) as u32,
            ),
            10 => {
                let rate = crate::framework::game_read(crate::audio::primary::MIX_RATE_SETTING_RVA);
                return crate::audio::primary::set_primary_format(primary, eax, rate).map(|ok| !ok);
            }
            12 => crate::audio::set_master_volume::set_master_volume_in_game(eax),
            14 | 15 => {
                use crate::audio::a3d::*;
                let v = A3dVars::game();
                let ok = if place == 14 {
                    open_a3d2(&v, &mut game_create)?
                } else {
                    open_a3d(&v, &mut game_create)?
                };
                return Ok(!ok);
            }
            16 => close_sound(
                game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
                game_addr(WINDOW_RVA) as u32,
                &mut game_part,
            )?,
            17 => {
                return open_sound(
                    eax,
                    &OpenVars::game(),
                    &mut game_part,
                    &mut game_direct_sound_create,
                )
                .map(|ok| !ok);
            }
            _ => {
                use crate::audio::primary::*;
                return open_listener(
                    primary,
                    listener,
                    game_addr(HAS_FPU_RVA) as u32,
                    game_addr(NO_LISTENER_RVA) as u32,
                    &mut game_part,
                )
                .map(|ok| !ok);
            }
        }
    }
    Ok(false)
}

/// `audio::release_primary`: unless A3D or no primary buffer: stopped, its kept volume put
/// back (unless 1, then 1), the listener released, the buffer released and forgotten.
pub unsafe fn release_primary(primary: u32, kept: u32, part: Part) -> Result<(), Refused> {
    unsafe {
        let p = read(primary);
        if crate::audio::buffer_settings::a3d() || p == 0 {
            return Ok(());
        }
        part(7, 0)?;
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
        part(8, 0)?;
        crate::audio::com::call(read(primary), crate::audio::com::RELEASE);
        write(primary, 0);
    }
    Ok(())
}

/// `audio::read_caps`: `GetCaps` into `caps` (zeroed, its size 0x60), logged. False for
/// the carry (it failed; nothing logged).
pub unsafe fn read_caps(direct_sound: u32, caps: u32, part: Part) -> Result<bool, Refused> {
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
                    part(0, game_addr(at) as u32)?
                }
                CapsLog::Hex(n) => part(2, field(n))?,
                CapsLog::Int(n) => part(4, field(n))?,
                CapsLog::IntLine(n) => part(5, field(n))?,
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
pub unsafe fn close_sound(direct_sound: u32, window: u32, part: Part) -> Result<(), Refused> {
    if crate::audio::buffer_settings::a3d() {
        return Err(BadInput("A3D (0x40CAD9)").into());
    }
    unsafe {
        let ds = read(direct_sound);
        if ds != 0 {
            part(1, game_addr(CLOSE_TEXT) as u32)?;
            let level: unsafe extern "system" fn(u32, u32, u32) -> i32 =
                std::mem::transmute(crate::audio::com::method(ds, SET_COOPERATIVE_LEVEL));
            level(ds, read(window), 1);
            crate::audio::com::call(read(direct_sound), crate::audio::com::RELEASE);
            write(direct_sound, 0);
        }
        part(9, 0)?;
    }
    Ok(())
}

pub const OPENED_MODE_RVA: usize = rva(0x4F82C0); // g_openedSoundMode
const DIRECT_SOUND_CREATE: usize = rva(0x4D5EA2); // the import's thunk
const OPEN_TEXT: usize = rva(0x4DA60F); // "AUD: open sound"
const OPEN_FAILED_TEXT: usize = rva(0x4DA61F); // "AUD: open failed rc="

/// `DirectSoundCreate` (the device's GUID, where the object goes) -> its result.
pub type CreateSound<'a> = &'a mut dyn FnMut(u32, u32) -> i32;

/// The game's own `DirectSoundCreate` (the ASI loader's dsound.dll), through its thunk; in
/// the standalone program DirectSound on SDL (`platform_sound`).
pub fn game_direct_sound_create(guid: u32, out: u32) -> i32 {
    #[cfg(feature = "standalone")]
    if crate::framework::standalone() {
        let _ = guid;
        return crate::platform_sound::direct_sound_create(out);
    }
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(game_addr(DIRECT_SOUND_CREATE));
        f(guid, out, 0)
    }
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
            window: game_addr(WINDOW_RVA) as u32,
        }
    }
}

/// `open_sound`'s parts in the game's memory: the mode read and written where the original
/// does, the logs and the calls through `part`, the platform's DirectSound.
struct Open<'a> {
    v: &'a OpenVars,
    part: Part<'a>,
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
        (self.part)(0, game_addr(OPEN_TEXT) as u32)?;
        (self.part)(3, guid)?;
        Ok(())
    }
    fn log_failed(&mut self, code: u32) -> Result<(), Refused> {
        (self.part)(0, game_addr(OPEN_FAILED_TEXT) as u32)?;
        (self.part)(3, code)?;
        Ok(())
    }
    fn open_a3d2(&mut self, guid: u32) -> Result<bool, Refused> {
        (self.part)(14, guid)
    }
    fn open_a3d(&mut self, guid: u32) -> Result<bool, Refused> {
        (self.part)(15, guid)
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
/// `open_a3d2` (14), which done is all; failing, the mode's 0x3000 bits flipped. Then only
/// with bit 0x1000: `open_a3d` (15), failing which `DirectSoundCreate`; cooperative level 3.
/// A failure (or no bit 0x1000) logs "AUD: open failed rc=" and a code (the GUID, or the
/// call's result) and gives false (the carry).
pub unsafe fn open_sound(
    guid: u32,
    v: &OpenVars,
    part: Part,
    create: CreateSound,
) -> Result<bool, Refused> {
    // Converted: `rc2k_core::audio::device::open_sound`.
    rc2k_core::audio::device::open_sound(guid, &mut Open { v, part, create })
}

/// `audio::reopen_sound`: when the mode's 0xF000 bits differ from those DirectSound was
/// opened with: `close_sound` (16) and `open_sound` (17, eax 0). Returns the carry (that
/// of `open_sound`, when it ran).
pub unsafe fn reopen_sound(mode: u32, opened: u32, part: Part) -> Result<bool, Refused> {
    unsafe {
        // Converted: `rc2k_core::audio::device::needs_reopen`.
        if !rc2k_core::audio::device::needs_reopen(read(mode), read(opened)) {
            return Ok(false);
        }
        part(16, 0)?;
        part(17, 0)
    }
}

// ---- for the self-tests ----

use std::sync::Mutex;

/// The stand-ins' notes: (place, eax where it is read: the log calls, 10 and 12).
pub static NOTED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

/// What the listener stand-in (place 13) answers: true for the carry.
pub static LISTENER_FAILS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What the stand-ins at places 14 .. 17 answer (true: the carry).
pub static PART_CARRIES: Mutex<[bool; 4]> = Mutex::new([false; 4]);

/// `DirectSoundCreate` in a copy: hands out `TEST_DIRECT_SOUND` (when `CREATE_RESULT` is
/// 0) and gives `CREATE_RESULT`, its three arguments taken off the stack.
pub static TEST_DIRECT_SOUND: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
pub static CREATE_RESULT: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub fn test_direct_sound_create(guid: u32, out: u32) -> i32 {
    use std::sync::atomic::Ordering;
    NOTED.lock().unwrap().push((0x100, guid));
    let r = CREATE_RESULT.load(Ordering::SeqCst);
    if r == 0 {
        unsafe { write(out, TEST_DIRECT_SOUND.load(Ordering::SeqCst)) };
    }
    r
}

extern "system" fn create_from_copy(guid: u32, out: u32, _outer: u32) -> i32 {
    test_direct_sound_create(guid, out)
}

/// The address a copy's `DirectSoundCreate` call is pointed at.
pub fn create_stand_in() -> usize {
    create_from_copy as *const () as usize
}

pub fn handler(place: u32, r: &mut crate::framework::Regs) {
    let carry = test_part(place, r.eax).unwrap_or(false);
    r.set_carry(carry);
}

pub fn noted() -> Vec<(u32, u32)> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

/// The Rust side's part in a test: what the stand-ins note.
pub fn test_part(place: u32, eax: u32) -> Result<bool, Refused> {
    let eax = if place < 7 || place == 10 || place == 12 {
        eax
    } else {
        0
    };
    NOTED.lock().unwrap().push((place, eax));
    Ok(place == 13 && LISTENER_FAILS.load(std::sync::atomic::Ordering::SeqCst))
}

/// A copy whose calls in `CALLEES` go to stand-ins and the rest where they went. (Each
/// argument is one of the copy's facts, read off IDA.)
#[allow(clippy::too_many_arguments)]
pub fn copy(
    hook: &crate::framework::RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    jumps_out: &[usize],
    variables: &[(usize, u32)],
    references: usize,
) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out,
        },
        &|callee| {
            if callee == DIRECT_SOUND_CREATE {
                return Some(create_stand_in());
            }
            CALLEES
                .iter()
                .position(|&c| c != 0 && rva(c) == callee)
                .map(|n| crate::install::stand_ins::stand_in(n as u32))
        },
        variables,
        references,
    )
}
