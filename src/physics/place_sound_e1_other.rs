//! `physics::place_sound_e1_other` (0x45CE32): when the car numbered `dword_6D3CD4` or the current car is
//! listed, `physics::sound_band_3` (ebx); a band not 0 goes into `dword_6D1AD4` and to the
//! current car's sound object with +0x238 0xE1, +0x23C the band, the place at +0x240 ..
//! and +0x10 1 (see `physics/car_sounds.rs`). A reading: a sound of another car met.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45F7B1` (its tail).
//! Refuses (before anything) a car number past 7, a null sound object when it is
//! written (the object is looked up first, but only written after the band).
//! Self-test: `physics/car_sounds.rs` (`test_sound`): the original and the Rust version on
//! the game's variables (saved and put back) with a test object, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::car_sounds::{g, listed, object, place, put, test_sound};
use crate::physics::sound_band_3::TWIN;
use crate::physics::sound_bands::band_value;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::place_sound_e1_other",
    rva(0x45CE32),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn place_sound_e1_other() -> Result<Left, Refused> {
    if !listed(g(0x6D3CD4))? && !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let scratch = g(0x6D1AEC);
    let b = unsafe { band_value(TWIN) };
    if b == 0 {
        return Ok(Left::nothing());
    }
    // The band left `dword_6D1AEC` written: put back if the object refuses.
    let o = match object() {
        Ok(o) => o,
        Err(e) => {
            unsafe { crate::framework::game_write(rva(0x6D1AEC), scratch) };
            return Err(e);
        }
    };
    unsafe {
        crate::framework::game_write(rva(0x6D1AD4), b);
        put(o + 0x238, 0xE1);
        put(o + 0x23C, b);
        place(o, 0x240, None);
        put(o + 0x10, 1);
    }
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_e1_other() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = match object() {
        Ok(o) => vec![
            (o + 0x10, 4),
            (o + 0x238, 0x28),
            (game_addr(rva(0x6D1AEC)) as u32, 4),
            (game_addr(rva(0x6D1AD4)) as u32, 4),
        ],
        Err(_) => vec![],
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    test_sound(
        &HOOK,
        call_original,
        &rust,
        &|seed: u32| unsafe { crate::framework::game_write(rva(0x6D3CD4), (seed * 3) % 8) },
        &[],
    )
}
