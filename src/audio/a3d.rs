//! Aureal's A3D: the two attempts to use it, its close, and the two
//! functions that keep A3D's splash settings in the registry. A3D has not existed for a
//! long time: the attempt (`CoCreateInstance` of A3D's classes) fails as on any machine
//! without A3D's driver (there is no COM here: the class is never there), and the
//! code goes the original's way for that.

use crate::audio::store::AudioState;
use crate::errors::{BadInput, Refused};
use crate::session::Dword;

/// `REGDB_E_CLASSNOTREG`: what `CoCreateInstance` says of a class no driver registered.
const CLASS_NOT_REGISTERED: i32 = 0x8004_0154u32 as i32;

/// `CoCreateInstance` (class, interface) -> its result: A3D's classes are never there.
fn game_create<C: ?Sized, I: ?Sized>(_class: &C, _interface: &I) -> i32 {
    CLASS_NOT_REGISTERED
}

/// The object a cell holds released (when there is one) and forgotten.
fn release(cell: &Dword) {
    let o = cell.get();
    if o != 0 {
        crate::platform_sound::release(o);
        cell.set(0);
    }
}

const THERE: &str = "A3D is there (CoCreateInstance worked): not ported";

/// `audio::open_a3d`: A3D as the DirectSound. False for the carry (not used).
pub fn open_a3d(state: &AudioState) -> Result<bool, Refused> {
    if state.no_a3d_setting.get() & 1 != 0 {
        release(&state.a3d_object);
        release(&state.direct_sound);
        return Ok(false);
    }
    state.a3d_ready.set(0);
    state.direct_sound.set(0);
    if game_create(&state.a3d_clsid, &state.a3d_iid) != 0 {
        return Ok(false);
    }
    Err(BadInput(THERE).into())
}

/// `audio::open_a3d2`: A3D's second interface. False for the carry (not used).
pub fn open_a3d2(state: &AudioState) -> Result<bool, Refused> {
    if state.no_a3d_setting.get() & 2 != 0 {
        release(&state.a3d_object);
        state.a3d_object.set(0);
        return Ok(false);
    }
    state.use_a3d.set(0);
    state.a3d_object.set(0);
    if game_create(&state.a3d2_clsid, &state.a3d2_iid) != 0 {
        return Ok(false);
    }
    Err(BadInput(THERE).into())
}

/// `audio::close_a3d2`: its interface released, `release_a3d` (place 9), A3D off.
pub fn close_a3d2(state: &AudioState) {
    release(&state.a3d2_interface);
    crate::audio::sound_device::release_a3d_object(state);
    state.use_a3d.set(0);
}

/// `audio::read_splash` and `write_splash`: nothing unless A3D was found at the start
/// (`g_a3dSplashKept` 1); then they read and turn off, or write back, A3D's
/// "SplashScreen" and "SplashAudio" under HKLM\SOFTWARE\Aureal\A3D: refused.
pub fn splash(state: &AudioState) -> Result<(), Refused> {
    if state.a3d_splash_kept.get() == 1 {
        return Err(BadInput("A3D's splash settings in the registry (not ported)").into());
    }
    Ok(())
}
