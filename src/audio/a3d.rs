//! Aureal's A3D (IDA, 2026-10-15): the two attempts to use it, its close, and the two
//! functions that keep A3D's splash settings in the registry. A3D has not existed for a
//! long time: the Rust versions make the same attempt (`CoCreateInstance`), go the
//! original's way when it fails, and refuse when it works.

use crate::audio::store::AUDIO;
use crate::errors::{BadInput, Refused};
use crate::session::Dword;

/// `CoCreateInstance`'s class context: in process.
const IN_PROCESS: u32 = 1;

/// `CoCreateInstance` (class, interface) -> its result. The object never reaches the game:
/// when there is one A3D is there, which the callers refuse, so it is released here at
/// once (a system object's address need not fit the game's 32 bits).
fn game_create(class: u32, interface: u32) -> i32 {
    let mut object: *mut core::ffi::c_void = std::ptr::null_mut();
    let r = unsafe {
        windows_sys::Win32::System::Com::CoCreateInstance(
            crate::pointers::host(class) as *const _,
            std::ptr::null_mut(),
            IN_PROCESS,
            crate::pointers::host(interface) as *const _,
            &mut object,
        )
    };
    if r >= 0 && !object.is_null() {
        // SAFETY: a COM object the system made: its table's third method is `Release`.
        unsafe {
            let table = *(object as *const *const usize);
            let release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32 =
                std::mem::transmute(*table.add(2));
            release(object);
        }
    }
    r
}

/// The object a cell holds released (when there is one) and forgotten.
fn release(cell: &Dword) {
    let o = cell.get();
    if o != 0 {
        unsafe { crate::audio::com::call(o, crate::audio::com::RELEASE) };
        cell.set(0);
    }
}

const THERE: &str = "A3D is there (CoCreateInstance worked): not ported";

/// `audio::open_a3d`: A3D as the DirectSound. False for the carry (not used).
pub fn open_a3d() -> Result<bool, Refused> {
    if AUDIO.no_a3d_setting.get() & 1 != 0 {
        release(&AUDIO.a3d_object);
        release(&AUDIO.direct_sound);
        return Ok(false);
    }
    AUDIO.a3d_ready.set(0);
    AUDIO.direct_sound.set(0);
    if game_create(AUDIO.a3d_clsid.addr(), AUDIO.a3d_iid.addr()) != 0 {
        return Ok(false);
    }
    Err(BadInput(THERE).into())
}

/// `audio::open_a3d2`: A3D's second interface. False for the carry (not used).
pub fn open_a3d2() -> Result<bool, Refused> {
    if AUDIO.no_a3d_setting.get() & 2 != 0 {
        release(&AUDIO.a3d_object);
        AUDIO.a3d_object.set(0);
        return Ok(false);
    }
    AUDIO.use_a3d.set(0);
    AUDIO.a3d_object.set(0);
    if game_create(AUDIO.a3d2_clsid.addr(), AUDIO.a3d2_iid.addr()) != 0 {
        return Ok(false);
    }
    Err(BadInput(THERE).into())
}

/// `audio::close_a3d2`: its interface released, `release_a3d` (place 9), A3D off.
pub fn close_a3d2() -> Result<(), Refused> {
    release(&AUDIO.a3d2_interface);
    crate::audio::sound_device::release_a3d_object();
    AUDIO.use_a3d.set(0);
    Ok(())
}

/// `audio::read_splash` and `write_splash`: nothing unless A3D was found at the start
/// (`g_a3dSplashKept` 1); then they read and turn off, or write back, A3D's
/// "SplashScreen" and "SplashAudio" under HKLM\SOFTWARE\Aureal\A3D: refused.
pub fn splash() -> Result<(), Refused> {
    if AUDIO.a3d_splash_kept.get() == 1 {
        return Err(BadInput("A3D's splash settings in the registry (not ported)").into());
    }
    Ok(())
}
