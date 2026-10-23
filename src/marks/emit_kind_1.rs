//! `marks::emit_kind_1` (0x4CCCD7, a chunk of `marks::emit_wheel_particles` entered by
//! `jz` from it for the wheel kind 1): `marks::emit_particle_21` when `dword_93A1D6` is
//! not 0, else (after `pusha`) the body at 0x4CCF71, `emit_particle_1`: one ring entry of
//! kind 1.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! `emit_particle_1`: `g_marksWheelFlags` the wheel's +0x20; the wheel's +4 (a float here)
//! grows by |+0x18| + |+0x10| + |+0x18 - +0xC|; when that rounded (`fist`) is 0x18 or
//! more it is kept as its remainder by 0x18 (as a float) and an entry may be made: the
//! strength (`dword_9356C0`, a float: |+0x10| by `flt_765168` plus |+0xC - +0x18|, by
//! `flt_7651D0`) must not be under `flt_765164` (compared unrounded), and is held to at
//! most 1 (`flt_76516C`); the life (`dword_93567C`) it by `flt_7651F0` plus `flt_7651E8`,
//! rounded; the fade (`flt_935678`) it by `dword_76519C`. The entry: its place the
//! wheel's (+0x3C ..) plus a random side times `flt_93543A` .., less `flt_7651DC` in y at
//! +0x28; its motion the car matrix's first row by +0x10, its third by +0x18 and
//! `flt_93543A` .. by +0xC, each by a factor (`flt_93A39E` ..), moved by
//! `marks::jitter_point` (spread `flt_93A3C2`), y plus `flt_93A3C6`; a random spin and
//! turn; the flags' low 6 bits; the life and fade (+0x78 fade over life, +0x74 fade);
//! the car and its +0x88; +0x70 0x10000 and +0x60 it by `flt_93A3CE`; +0x64 0.
//! Refuses (before anything) a null wheel, a null object where read, a null car or
//! matrix when an entry is made, and `emit_particle_21`'s. Self-test: a ring, wheel and
//! car of the test's (`marks::emit_particle_6::TestEmitter`), both bodies, several
//! speeds and fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emit_particle_21::emit_particle_21;
use crate::marks::emitters::*;
use crate::marks::jitter_point::jitter_point;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, state_entry};

const CHOICE_RVA: usize = rva(0x93A1D6); // g_marksKind1Choice: emit_particle_21 when not 0
const STRENGTH_RVA: usize = rva(0x9356C0); // dword_9356C0
const LIFE_RVA: usize = rva(0x93567C); // dword_93567C
const FADE_RVA: usize = rva(0x935678); // flt_935678
const SIDE_RVA: usize = rva(0x765138); // dword_765138
const SIDE_RANDOM_RVA: usize = rva(0x93A39A);
const SPIN_RANDOM_RVA: usize = rva(0x93A3CA);
/// flt_765168, flt_7651D0, flt_765164: the strength's factors and its least.
const STRENGTH_K: [usize; 3] = [rva(0x765168), rva(0x7651D0), rva(0x765164)];
/// flt_7651F0, flt_7651E8: the life's factor and base.
const LIFE_K: [usize; 2] = [rva(0x7651F0), rva(0x7651E8)];
const FADE_K_RVA: usize = rva(0x76519C); // dword_76519C
/// flt_93543A, flt_93543E, flt_935442: the side's factors (the caller's).
const SIDES_K: [usize; 3] = [rva(0x93543A), rva(0x93543E), rva(0x935442)];
const DROP_RVA: usize = rva(0x7651DC); // flt_7651DC
/// Per axis: the factors of the first row by +0x10, the third by +0x18, the side by +0xC.
const MOTION_K: [[usize; 3]; 3] = [
    [rva(0x93A39E), rva(0x93A3A2), rva(0x93A3A6)],
    [rva(0x93A3AA), rva(0x93A3AE), rva(0x93A3B2)],
    [rva(0x93A3B6), rva(0x93A3BA), rva(0x93A3BE)],
];
const SPREAD_RVA: usize = rva(0x93A3C2);
const LIFT_RVA: usize = rva(0x93A3C6);
const SIZE_K_RVA: usize = rva(0x93A3CE);
const SIZE: u32 = 0x1_0000;
const STEP: i32 = 0x18;

/// The first part of `emit_particle_1`, worked out without writing: the wheel's +4 grown
/// (as `fist` and as a float), and, when an entry may be made, the strength (the float
/// kept, before the hold to 1).
struct Pace {
    grown: (i32, u32),
    strength: Option<(u32, bool)>,
}

unsafe fn pace(wheel: u32) -> Pace {
    unsafe {
        let grown = x87::abs_sum_to_int_kept(
            at(wheel + 0x18),
            at(wheel + 0x10),
            at(wheel + 0xC),
            at(wheel + 4),
        );
        if grown.0 < STEP {
            return Pace {
                grown,
                strength: None,
            };
        }
        let [k1, k2, least] = STRENGTH_K.map(|r| game_read::<u32>(r));
        let (kept, status) = x87::abs_blend_compare(
            at(wheel + 0x10),
            k1,
            at(wheel + 0xC),
            at(wheel + 0x18),
            k2,
            least,
        );
        let under = status & 0x700 == 0x100;
        Pace {
            grown,
            strength: Some((kept, under)),
        }
    }
}

/// Why `emit_particle_1` refuses, if it does, worked out without writing.
unsafe fn refusal(wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        match pace(wheel).strength {
            Some((_, false)) => {}
            _ => return None,
        }
        if object == 0 {
            return Some("a null object");
        }
        next_entry()?;
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null car object or matrix");
        }
        None
    }
}

/// The body at 0x4CCF71 (see the header).
pub unsafe fn emit_particle_1(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(wheel, object) {
            return Err(BadInput(why).into());
        }
        let flags = at(wheel + 0x20);
        game_write(FLAGS_RVA, flags);
        let p = pace(wheel);
        game_write(STRENGTH_RVA, p.grown.0);
        ((wheel + 4) as *mut u32).write_unaligned(p.grown.1);
        let Some((kept, under)) = p.strength else {
            return Ok(());
        };
        let rest = p.grown.0 % STEP;
        game_write(STRENGTH_RVA, rest);
        ((wheel + 4) as *mut u32).write_unaligned(x87::from_int(rest));
        game_write(STRENGTH_RVA, kept);
        if under {
            return Ok(());
        }
        let one: u32 = game_read(ONE_RVA);
        if x87::compare_status(kept, one) != 0x100 {
            game_write(STRENGTH_RVA, one);
        }
        let strength: u32 = game_read(STRENGTH_RVA);
        let life = x87::times_plus_to_int(strength, game_read(LIFE_K[0]), game_read(LIFE_K[1]));
        game_write(LIFE_RVA, life);
        let fade = x87::mul(strength, game_read(FADE_K_RVA));
        game_write(FADE_RVA, fade);
        if next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, 1);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let side = (next_c5(SIDE_RANDOM_RVA) & 0x3FFF) as i32 - 0x2000;
        game_write(SIDE_RVA, side);
        let place: [[u8; 10]; 3] = std::array::from_fn(|k| {
            x87::plus_int_times_80(at(wheel + 0x3C + 4 * k as u32), game_read(SIDES_K[k]), side)
        });
        put(0x10, rounded(place[0]));
        put(0x14, rounded(place[1]));
        put(0x28, x87::ext_less(place[1], game_read(DROP_RVA)));
        put(0x18, rounded(place[2]));
        let motion: [[u8; 10]; 3] = std::array::from_fn(|k| {
            let k4 = 4 * k as u32;
            let f = MOTION_K[k].map(|r| game_read::<u32>(r));
            x87::three_products_80([
                [at(m + k4), at(wheel + 0x10), f[0]],
                [at(m + 0x20 + k4), at(wheel + 0x18), f[1]],
                [game_read(SIDES_K[k]), at(wheel + 0xC), f[2]],
            ])
        });
        let (moved, _) = jitter_point([
            extended(game_read(SPREAD_RVA)),
            motion[2],
            motion[1],
            motion[0],
        ]);
        put(0x34, rounded(moved[0]));
        put(0x30, x87::add(rounded(moved[1]), game_read(LIFT_RVA)));
        put(0x2C, rounded(moved[2]));
        let r = next_c5(SPIN_RANDOM_RVA);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, flags & 0x3F);
        put(0x84, life as u32);
        put(0x78, x87::over_int(fade, life));
        put(0x74, fade);
        put(0x68, car);
        put(0x70, SIZE);
        put(0x6C, at(car + 0x88));
        put(0x60, x87::int_times(SIZE as i32, game_read(SIZE_K_RVA)));
        put(0x64, 0);
    }
    Ok(())
}

/// The Rust version (see the header).
pub unsafe fn emit_kind_1(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(CHOICE_RVA) != 0 {
            emit_particle_21(wheel, object)
        } else {
            emit_particle_1(wheel, object)
        }
    }
}

/// What it may write.
pub fn regions(wheel: u32) -> Vec<Region> {
    if unsafe { game_read::<u32>(CHOICE_RVA) } != 0 {
        return crate::marks::emit_particle_21::regions(wheel);
    }
    let mut r: Vec<Region> = [
        FLAGS_RVA,
        STRENGTH_RVA,
        LIFE_RVA,
        FADE_RVA,
        SIDE_RVA,
        SIDE_RANDOM_RVA,
        SPIN_RANDOM_RVA,
        SEED_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec();
    if wheel != 0 {
        r.push((wheel + 4, 4));
    }
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_kind_1(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
