//! Aureal's A3D: the two attempts to use it, its close, and the two functions that keep A3D's
//! splash settings in the registry. A3D's classes are never found, so they all do nothing.

use crate::audio::own::AudioOwn;
use crate::errors::{BadInput, Refused};
use crate::session::PlainDword;

/// What `CoCreateInstance` says of a class no driver registered.
const CLASS_NOT_REGISTERED: i32 = 0x8004_0154u32 as i32;

/// `CoCreateInstance` (class, interface) -> its result.
fn game_create<C: ?Sized, I: ?Sized>(_class: &C, _interface: &I) -> i32 {
    CLASS_NOT_REGISTERED
}

/// The object a cell holds released (when there is one) and forgotten.
fn release(cell: &PlainDword) {
    let o = cell.get();
    if o != 0 {
        crate::platform_sound::release(o);
        cell.set(0);
    }
}

const THERE: &str = "A3D is there (CoCreateInstance worked): not ported";

/// A3D as the DirectSound. Always false (not used).
pub fn open_a3d(state: &AudioOwn) -> Result<bool, Refused> {
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

/// A3D's second interface. Always false (not used).
pub fn open_a3d2(state: &AudioOwn) -> Result<bool, Refused> {
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

/// Releases the second interface and A3D.
pub fn close_a3d2(state: &AudioOwn) {
    release(&state.a3d2_interface);
    crate::audio::sound_device::release_a3d_object(state);
    state.use_a3d.set(0);
}

/// Nothing unless A3D was found at the start; then they would read and turn off, or write back,
/// A3D's splash settings in the registry: refused.
pub fn splash(state: &AudioOwn) -> Result<(), Refused> {
    if state.a3d_splash_kept.get() == 1 {
        return Err(BadInput("A3D's splash settings in the registry (not ported)").into());
    }
    Ok(())
}
