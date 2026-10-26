//! What `audio::set_buffer_volume`, `set_buffer_pan`, `set_buffer_frequency` and
//! `set_buffer_place` share: a voice's settings to its DirectSound buffer (one COM call
//! each), and their self-test on voices with test objects.

use crate::audio::com::{self, TestObjects, take_noted};
use crate::audio::voices::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, shadow};
use crate::screens::harness::CallOriginal;

pub const SET_VOLUME: u32 = 0x3C;
pub const SET_PAN: u32 = 0x40;
pub const SET_FREQUENCY: u32 = 0x44;
pub const SET_POSITION: u32 = 0x4C;

pub fn a3d() -> bool {
    unsafe { game_read::<u32>(A3D_RVA) == 1 }
}

/// `g_volumeTable`'s entry `n`, for the core.
pub fn volumes(n: u32) -> u32 {
    unsafe { game_read(VOLUMES_RVA + n as usize * 4) }
}

fn buffer(v: &Voice) -> Result<u32, Refused> {
    if v.buffer == 0 {
        return Err(BadInput("a voice without its buffer").into());
    }
    Ok(v.buffer)
}

/// `SetVolume` with `g_volumeTable[volume * volume_factor >> 6]` (signed). Returns eax.
pub unsafe fn set_buffer_volume(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEFE`)").into());
    }
    // Converted: `rc2k_core::audio::levels::buffer_volume`.
    let level = rc2k_core::audio::levels::buffer_volume(v.volume, v.volume_factor, volumes)?;
    let b = buffer(v)?;
    Ok(unsafe { com::call1(b, SET_VOLUME, level) })
}

/// `SetPan`, nothing in 3D: a pan below 0 is `g_volumeTable[pan + 0x40]`, else
/// `-g_volumeTable[0x40 - pan]`. Returns eax, or `None` where the original leaves eax.
pub unsafe fn set_buffer_pan(v: &Voice) -> Result<Option<i32>, Refused> {
    if unsafe { game_read::<u32>(SOUND_MODE_RVA) } & MODE_3D != 0 {
        return Ok(None);
    }
    if a3d() {
        return Err(BadInput("A3D (`nullsub_45`)").into());
    }
    // Converted: `rc2k_core::audio::levels::pan`.
    let value = rc2k_core::audio::levels::pan(v.pan, volumes)?;
    let b = buffer(v)?;
    Ok(Some(unsafe { com::call1(b, SET_PAN, value) }))
}

/// `SetFrequency` with the frequency shifted right by the format's halvings.
pub unsafe fn set_buffer_frequency(v: &Voice) -> Result<i32, Refused> {
    if a3d() {
        return Err(BadInput("A3D (`sub_40CEC5`)").into());
    }
    let b = buffer(v)?;
    // Converted: `rc2k_core::audio::levels::buffer_frequency`.
    let frequency = rc2k_core::audio::levels::buffer_frequency(v.frequency, v.format);
    Ok(unsafe { com::call1(b, SET_FREQUENCY, frequency) })
}

/// The 3D buffer's `SetPosition` with the place, deferred (1) while the voice sounds
/// (state 2), else at once (0).
pub unsafe fn set_buffer_place(v: &Voice) -> Result<i32, Refused> {
    if v.buffer_3d == 0 {
        return Err(BadInput("a voice without its 3D buffer").into());
    }
    // Converted: `rc2k_core::audio::buffers::place_deferral`.
    let deferred = rc2k_core::audio::buffers::place_deferral(v);
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(com::method(v.buffer_3d, SET_POSITION));
        Ok(f(v.buffer_3d, v.place[0], v.place[1], v.place[2], deferred))
    }
}

/// The whole `dispatch`: in shadow mode the original alone (the call is DirectSound's);
/// else the Rust version, and eax its result. ecx and edx are left as they were: the COM
/// method leaves them as it likes, and no caller reads them.
pub fn dispatch_setting(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    rust: unsafe fn(&Voice) -> Result<Option<i32>, Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { rust(voice(r.esi)) } {
        Ok(Some(eax)) => r.eax = eax as u32,
        Ok(None) => {}
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// The self-test: the original and the Rust version on voices of the test's whose buffers
/// note their calls, for each of `voices`; both noted calls and eax compared.
pub fn check_setting(
    t: &mut SelfTest,
    call_original: CallOriginal,
    rust: unsafe fn(&Voice) -> Result<Option<i32>, Refused>,
    voices: &[Voice],
) {
    if a3d() {
        t.check(true, String::new);
        return;
    }
    let objects = TestObjects::new(
        2,
        &[
            (SET_VOLUME, 1),
            (SET_PAN, 1),
            (SET_FREQUENCY, 1),
            (SET_POSITION, 4),
        ],
    );
    for (n, &given) in voices.iter().enumerate() {
        let mut v = given;
        v.buffer = objects.get(0);
        v.buffer_3d = objects.get(1);
        let before = Regs {
            esi: &v as *const Voice as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = take_noted();
        let mine = unsafe { rust(&v) };
        let noted = take_noted();
        let eax_ok = match mine {
            Ok(Some(eax)) => ro.eax == eax as u32,
            Ok(None) => ro.eax == before.eax,
            Err(_) => false,
        };
        t.check(eax_ok && theirs == noted, || {
            format!(
                "voice {n} (volume {:#x} x {:#x}, pan {}, frequency {}, format {:#x}): original {theirs:x?} eax {:#x}; rust {mine:?} {noted:x?}",
                v.volume, v.volume_factor, v.pan, v.frequency, v.format, ro.eax
            )
        });
        let _ = EAX;
    }
}
