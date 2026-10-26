//! Aureal's A3D (IDA, 2026-10-15): the two attempts to use it, its close, and the two
//! functions that keep A3D's splash settings in the registry. A3D has not existed for a
//! long time: the Rust versions make the same attempt (`CoCreateInstance`), go the
//! original's way when it fails, and refuse when it works.

use crate::audio::store::AUDIO;
use crate::errors::{BadInput, Refused};
use crate::session::Dword;

/// `CoCreateInstance`'s class context: in process.
const IN_PROCESS: u32 = 1;

/// `CoCreateInstance` (class, interface, where the object goes) -> its result.
fn game_create(class: u32, interface: u32, out: u32) -> i32 {
    let r = unsafe {
        windows_sys::Win32::System::Com::CoCreateInstance(
            crate::pointers::host(class) as *const _,
            std::ptr::null_mut(),
            IN_PROCESS,
            crate::pointers::host(interface) as *const _,
            crate::pointers::host(out) as *mut *mut core::ffi::c_void,
        )
    };
    if r >= 0 {
        // The system's object: its table pointer and the table's first methods (as far as
        // `Release`, all the game calls) made regions for the program's life.
        let object: u32 = crate::pointers::read_or_end(out, "the A3D object's place");
        if object != 0 {
            crate::pointers::register(object, 4);
            let table: u32 = crate::pointers::read_or_end(object, "the A3D object");
            crate::pointers::register(table, crate::audio::com::RELEASE + 4);
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
    if game_create(
        AUDIO.a3d_clsid.addr() as u32,
        AUDIO.a3d_iid.addr() as u32,
        AUDIO.direct_sound.addr() as u32,
    ) != 0
    {
        AUDIO.direct_sound.set(0);
        return Ok(false);
    }
    release(&AUDIO.direct_sound);
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
    if game_create(
        AUDIO.a3d2_clsid.addr() as u32,
        AUDIO.a3d2_iid.addr() as u32,
        AUDIO.a3d_object.addr() as u32,
    ) != 0
    {
        AUDIO.a3d_object.set(0);
        return Ok(false);
    }
    release(&AUDIO.a3d_object);
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
