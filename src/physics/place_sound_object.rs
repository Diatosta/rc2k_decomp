//! `physics::place_sound_object` (0x45CD60): when the current car is listed: a sound by the kind of the
//! object `dword_6D321C` (`g_objectKinds`, its material record's byte +0x24 & 0xF, then
//! `dword_6D1A90[]`) in eax, `physics::sound_band_2` (ebx); a band not 0 goes to the car's
//! sound object with +0x238 the sound, +0x23C the band, the place offset by
//! `dword_6D3B4C` .. at +0x240 .. and +0x10 1 (see `physics/car_sounds.rs`). A reading:
//! the sound of a roadside object hit.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45E9E0`.
//! Refuses (before anything) a car number past 7, a null sound object when it is
//! written, an object past 0x100 or a kind past the 0x4B0 records.
//! Self-test: `physics/car_sounds.rs` (`test_sound`): the original and the Rust version on
//! the game's variables (saved and put back) with a test object, under three precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::car_sounds::{g, listed, object, place, put, test_sound};
use crate::physics::sound_band_2::TWIN;
use crate::physics::sound_bands::band_value;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::place_sound_object",
    rva(0x45CD60),
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
pub unsafe fn place_sound_object() -> Result<Left, Refused> {
    if !listed(g(0x71B3F8))? {
        return Ok(Left::nothing());
    }
    let thing = g(0x6D321C);
    if thing >= 0x100 {
        return Err(BadInput("an object past 0x100").into());
    }
    let kind = g(0x5CD27C + 4 * thing as usize);
    if kind >= 0x4B0 {
        return Err(BadInput("a kind past the 0x4B0 records").into());
    }
    let byte = unsafe { *(game_addr(rva(0x97DF54 + 0x80 * kind as usize)) as *const u8) } as u32;
    let sound = g(0x6D1A90 + 4 * (byte & 0xF) as usize);
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
        put(o + 0x238, sound);
        put(o + 0x23C, b);
        place(o, 0x240, Some([g(0x6D3B4C), g(0x6D3B50), g(0x6D3B54)]));
        put(o + 0x10, 1);
    }
    Ok(Left::nothing())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { place_sound_object() }
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
    let kinds = (game_addr(rva(0x5CD27C)) as u32, 0x40);
    test_sound(
        &HOOK,
        call_original,
        &rust,
        &|seed: u32| unsafe {
            crate::framework::game_write(rva(0x6D321C), seed % 16);
            for k in 0..16usize {
                crate::framework::game_write(rva(0x5CD27C + 4 * k), (k as u32 * 7) % 0x40);
            }
            for k in 0..3 {
                crate::framework::game_write(
                    rva(0x6D3B4C + 4 * k),
                    (seed + k as u32).wrapping_mul(0x0101_0101),
                );
            }
        },
        &[kinds],
    )
}
