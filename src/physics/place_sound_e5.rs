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
//! the game's variables (saved and put back) with a test object, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::car_sounds::{g, listed, object, place, put, test_sound};
use crate::physics::sound_band_4::TWIN;
use crate::physics::sound_bands::band_value;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
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
pub unsafe fn place_sound_e5() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let o = object()?;
    let b = unsafe { band_value(TWIN) };
    unsafe {
        put(o + 0x24C, 0xE5);
        put(o + 0x250, b);
        place(o, 0x254, None);
        put(o + 0x10, 1);
    }
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_e5() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = match object() {
        Ok(o) => vec![
            (o + 0x10, 4),
            (o + 0x238, 0x28),
            (game_addr(rva(0x6D1AEC)) as u32, 4),
        ],
        Err(_) => vec![],
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    test_sound(&HOOK, call_original, &rust, &|_| {}, &[])
}
