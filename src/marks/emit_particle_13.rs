//! `marks::emit_particle_13` (0x4CE05E): one ring entry of kind 0xD for a car each frame,
//! at its model's point (`flt_769312` .., 12 bytes by the car's +4).
//!
//! In:  esi the car object
//! Out: every register kept (`pusha` .. `popa`)
//!
//! The entry: its place the point (x negated; y less `flt_7651E4`; by `flt_7651D4`)
//! turned by `g_marksCar`'s matrix (placed and the place taken off again, on the x87
//! stack: `x87::turned_place_less`), less `flt_93A462` in y at +0x28; the strength
//! (`dword_9356C0`, a float) 1 - 1 / (the car's +8 by `flt_93A466` plus 1); its motion the
//! matrix's third row by the strength and `flt_93A46A` ..; then its place moved by the
//! motion times a random's low word by `flt_765214` times `g_marksFrameTicks`; a random
//! spin and turn (+0x54, +0x50); +0x58 0x3F; a life of 250 (+0x78 the strength by
//! `flt_7651A4` over it); +0x74 the strength by `dword_7651A0`; +0x68 `g_marksCar` and its
//! +0x88; +0x70 `flt_93A47E` by the strength plus `flt_93A482`, rounded, and +0x60 that by
//! `flt_93A486`. Caller: `sub_4C9AF6` (the cars' loop). Refuses (before anything) a null
//! object, a null `g_marksCar` or matrix, and a model (+4) past the table's 34 entries,
//! when an entry is made. Self-test: a ring and car of the test's
//! (`marks::emit_particle_6::TestEmitter`), several models, speeds, ticks and fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emitters::*;
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, state_entry};

const POINTS_RVA: usize = rva(0x769312); // flt_769312: 3 floats by the car's model
const MODELS: u32 = 34;
const SCALE_RVA: usize = rva(0x7651D4); // flt_7651D4
const DROP_RVA: usize = rva(0x7651E4); // flt_7651E4
const LIFT_RVA: usize = rva(0x93A462);
const STRENGTH_RVA: usize = rva(0x9356C0); // dword_9356C0
const STRENGTH_K_RVA: usize = rva(0x93A466);
/// flt_93A46A, flt_93A46E, flt_93A472: the motion's factors (x, y, z).
const MOTION_K: [usize; 3] = [rva(0x93A46A), rva(0x93A46E), rva(0x93A472)];
const DRIFT_RANDOM_RVA: usize = rva(0x93A476);
const DRIFT_K_RVA: usize = rva(0x765214); // flt_765214
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const SPIN_RANDOM_RVA: usize = rva(0x93A47A);
const LIFE_SCALE_RVA: usize = rva(0x7651A4); // flt_7651A4
const FADE_K_RVA: usize = rva(0x7651A0); // dword_7651A0
/// flt_93A47E, flt_93A482, flt_93A486: the size's factor, base and scale.
const SIZE_K: [usize; 3] = [rva(0x93A47E), rva(0x93A482), rva(0x93A486)];
const LOOK: u32 = 0x3F;
const LIFE: u32 = 0xFA;

/// Why the Rust version refuses, if it does, worked out without writing.
unsafe fn refusal(object: u32) -> Option<&'static str> {
    unsafe {
        if object == 0 {
            return Some("a null object");
        }
        // The ring full: nothing more is read.
        next_entry()?;
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null g_marksCar or matrix");
        }
        if at(object + 4) >= MODELS {
            return Some("a car model past flt_769312's 34 entries");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_13(object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(object) {
            return Err(BadInput(why).into());
        }
        if next_entry().is_none() {
            return Ok(());
        }
        let model = at(object + 4) as usize;
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, 0xD);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let t: [u32; 3] = game_read(POINTS_RVA + 12 * model);
        let [x, y, y_less, z] = x87::turned_place_less(
            t,
            game_read(SCALE_RVA),
            game_read(DROP_RVA),
            game_read(LIFT_RVA),
            m,
        );
        put(0x10, x);
        put(0x14, y);
        put(0x28, y_less);
        put(0x18, z);
        let one: u32 = game_read(ONE_RVA);
        let s = x87::one_less_inverse(at(object + 8), game_read(STRENGTH_K_RVA), one);
        game_write(STRENGTH_RVA, s);
        let motion: [u32; 3] = std::array::from_fn(|k| {
            x87::mul2(at(m + 0x20 + 4 * k as u32), s, game_read(MOTION_K[k]))
        });
        put(0x2C, motion[0]);
        put(0x30, motion[1]);
        put(0x34, motion[2]);
        let r = (next_c5(DRIFT_RANDOM_RVA) & 0xFFFF) as i32;
        let moved = x87::drifted(
            r,
            game_read(DRIFT_K_RVA),
            motion,
            game_read(TICKS_RVA),
            [x, y, z],
        );
        put(0x10, moved[0]);
        put(0x14, moved[1]);
        put(0x18, moved[2]);
        let r = next_c5(SPIN_RANDOM_RVA);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, LOOK);
        put(0x84, LIFE);
        put(
            0x78,
            x87::times_over_int(s, game_read(LIFE_SCALE_RVA), LIFE as i32),
        );
        put(0x74, x87::mul(s, game_read(FADE_K_RVA)));
        put(0x68, car);
        let size = x87::times_plus_to_int(game_read(SIZE_K[0]), s, game_read(SIZE_K[1]));
        put(0x70, size as u32);
        put(0x6C, at(car + 0x88));
        put(0x60, x87::int_times(size, game_read(SIZE_K[2])));
    }
    Ok(())
}

/// The globals it may write.
pub fn globals() -> Vec<Region> {
    [
        STRENGTH_RVA,
        DRIFT_RANDOM_RVA,
        SPIN_RANDOM_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec()
}

/// What it may write: the globals and, when one is to be taken, the next entry.
pub fn regions() -> Vec<Region> {
    let mut r = globals();
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_13(r.esi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
