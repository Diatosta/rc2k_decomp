//! `physics::place_sound_e5` (0x45CF95): when the current car is listed, `physics::sound_band_4` (ebx)
//! goes to its sound object with +0x24C 0xE5, +0x250 the band, the place at +0x254 .. and
//! +0x10 1 (see `physics/car_sounds.rs`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45EDB5`.
//! Refuses (before anything) a car number past 7, a null sound object when it is
//! written.
//! Self-test: `physics/car_sounds.rs` (`test_sound`): the original and the Rust version on
//! the game's variables (saved and put back) with a test object, exact at double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::physics::car_sounds::{cue_plain, dispatch_placed, g, listed, object, test_sound};
use crate::physics::sound_band_4::TWIN;
use crate::physics::sound_bands::band_value_plain;
use crate::screens::harness::Left;
use rc2k_core::physics::sound_places::{SECOND, SOUND_E5};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::place_sound_e5", rva(0x45CF95), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::physics::sound_places`.
pub unsafe fn place_sound_e5() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let o = object()?;
    let b = unsafe { band_value_plain(TWIN) };
    unsafe { cue_plain(o, SECOND, SOUND_E5, b, None) };
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_e5() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_placed(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    test_sound(&HOOK, call_original, &rust, &|_| {}, &[])
}
