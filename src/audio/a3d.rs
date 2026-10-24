//! Aureal's A3D (IDA, 2026-10-15): the two attempts to use it, its close, and the two
//! functions that keep A3D's splash settings in the registry. A3D has not existed for a
//! long time: the Rust versions make the same attempt (`CoCreateInstance`), go the
//! original's way when it fails, and refuse when it works.

use crate::audio::store::Slot;
use crate::audio::voices::{read, write};
use crate::errors::{BadInput, Refused};
use crate::store::addr32 as a;
use crate::store::game_addr;

pub const NO_A3D_SETTING_RVA: Slot = Slot::at(0x506C5C); // g_noA3dSetting: 1 no A3D, 2 no A3D2
pub const A3D2_INTERFACE_RVA: Slot = Slot::at(0x506D58); // g_a3d2Interface
pub const SPLASH_KEPT_RVA: Slot = Slot::at(0x506C60); // g_a3dSplashKept (1: the splash pair acts)
const A3D_CLSID_RVA: Slot = Slot::at(0x506CD8);
const A3D_IID_RVA: Slot = Slot::at(0x506A28);
const A3D2_CLSID_RVA: Slot = Slot::at(0x506D6C);
const A3D2_IID_RVA: Slot = Slot::at(0x506D8C);
/// `CoCreateInstance`'s class context: in process.
const IN_PROCESS: u32 = 1;

/// `CoCreateInstance` (class, interface, where the object goes) -> its result.
pub type Create<'a> = &'a mut dyn FnMut(u32, u32, u32) -> i32;

pub fn game_create(class: u32, interface: u32, out: u32) -> i32 {
    unsafe {
        windows_sys::Win32::System::Com::CoCreateInstance(
            class as *const _,
            std::ptr::null_mut(),
            IN_PROCESS,
            interface as *const _,
            out as *mut *mut core::ffi::c_void,
        )
    }
}

unsafe fn release(at: u32) {
    unsafe {
        let o = read(at);
        if o != 0 {
            crate::audio::com::call(o, crate::audio::com::RELEASE);
            write(at, 0);
        }
    }
}

const THERE: &str = "A3D is there (CoCreateInstance worked): not ported";

/// Where the attempts' things are.
#[derive(Clone, Copy)]
pub struct A3dVars {
    pub setting: u32,
    pub ready: u32,
    pub direct_sound: u32,
    pub object: u32,
    pub use_a3d: u32,
}

impl A3dVars {
    pub fn game() -> Self {
        Self {
            setting: a(NO_A3D_SETTING_RVA),
            ready: a(crate::audio::sound_device::A3D_READY_RVA),
            direct_sound: a(crate::audio::make_buffer::DIRECT_SOUND_RVA),
            object: a(crate::audio::sound_device::A3D_OBJECT_RVA),
            use_a3d: a(crate::audio::voices::A3D_RVA),
        }
    }
}

/// `audio::open_a3d`: A3D as the DirectSound. False for the carry (not used).
pub unsafe fn open_a3d(v: &A3dVars, create: Create) -> Result<bool, Refused> {
    unsafe {
        if read(v.setting) & 1 != 0 {
            release(v.object);
            release(v.direct_sound);
            return Ok(false);
        }
        write(v.ready, 0);
        if create(
            game_addr(A3D_CLSID_RVA) as u32,
            game_addr(A3D_IID_RVA) as u32,
            v.direct_sound,
        ) != 0
        {
            write(v.direct_sound, 0);
            return Ok(false);
        }
        release(v.direct_sound);
    }
    Err(BadInput(THERE).into())
}

/// `audio::open_a3d2`: A3D's second interface. False for the carry (not used).
pub unsafe fn open_a3d2(v: &A3dVars, create: Create) -> Result<bool, Refused> {
    unsafe {
        if read(v.setting) & 2 != 0 {
            release(v.object);
            write(v.object, 0);
            return Ok(false);
        }
        write(v.use_a3d, 0);
        if create(
            game_addr(A3D2_CLSID_RVA) as u32,
            game_addr(A3D2_IID_RVA) as u32,
            v.object,
        ) != 0
        {
            write(v.object, 0);
            return Ok(false);
        }
        release(v.object);
    }
    Err(BadInput(THERE).into())
}

/// `audio::close_a3d2`: its interface released, `release_a3d` (place 9), A3D off.
pub unsafe fn close_a3d2(interface: u32, use_a3d: u32) -> Result<(), Refused> {
    unsafe {
        release(interface);
        crate::audio::sound_device::release_a3d_object();
        write(use_a3d, 0);
    }
    Ok(())
}

/// `audio::read_splash` and `write_splash`: nothing unless A3D was found at the start
/// (`g_a3dSplashKept` 1); then they read and turn off, or write back, A3D's
/// "SplashScreen" and "SplashAudio" under HKLM\SOFTWARE\Aureal\A3D: refused.
pub unsafe fn splash(kept: u32) -> Result<(), Refused> {
    if unsafe { read(kept) } == 1 {
        return Err(BadInput("A3D's splash settings in the registry (not ported)").into());
    }
    Ok(())
}
