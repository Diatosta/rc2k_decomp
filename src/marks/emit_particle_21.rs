//! `marks::emit_particle_21` (0x4CCA58, a chunk of `marks::emit_wheel_particles` entered
//! by `jnz` from 0x4CCCDE when `dword_93A1D6` is not 0, left by `popa; retn`): one ring
//! entry of kind 0x15 for the wheel kind 1, when the rate gate passes.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The rate (`dword_9356B0`): (1 - 1 / (the wheel's +0xC by `flt_93A312`, `flt_93A316`,
//! plus `flt_93A31A`)) by `flt_93A31E`; then `marks/emitters.rs`' gate. The entry: its
//! place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the car matrix's first
//! row, 0 or 1 times its third), less `flt_93A326` in y at +0x28; its motion per axis the
//! matrix's first row by the wheel's +0x10, its third by +0x18 and by +0xC, each by a
//! factor (`flt_93A32A` ..); a random spin and turn (+0x54, +0x50); +0x58 0x34; a life of
//! 1000 (+0x78 `flt_7651A4` over it); +0x74 `dword_76519C`; the car (+0x68) and its +0x88,
//! +0x90 plus the rate; +0x60 that sum by `flt_93A352` (after a first value by the marks'
//! random, which still steps). Refuses (before anything) a null wheel, a null object, and a
//! null car or matrix when an entry is made.

use crate::errors::{BadInput, Refused};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::marks::store::{Slot, game_read, game_write};
use crate::math::x87;
use crate::scene::matrix3::rounded;

const RATE_K: [Slot; 4] = [
    Slot::at(0x93A312),
    Slot::at(0x93A316),
    Slot::at(0x93A31A),
    Slot::at(0x93A31E),
];
const SIDE_RANDOM_RVA: Slot = Slot::at(0x93A322);
const LIFT_RVA: Slot = Slot::at(0x93A326);
/// Per axis (x, y, z): the factors of the first row by +0x10, the third by +0x18 and by
/// +0xC (`flt_93A32A` .. `flt_93A34A`).
const MOTION_K: [[Slot; 3]; 3] = [
    [Slot::at(0x93A32A), Slot::at(0x93A32E), Slot::at(0x93A332)],
    [Slot::at(0x93A336), Slot::at(0x93A33A), Slot::at(0x93A33E)],
    [Slot::at(0x93A342), Slot::at(0x93A346), Slot::at(0x93A34A)],
];
const SPIN_RANDOM_RVA: Slot = Slot::at(0x93A34E);
const LIFE_SCALE_RVA: Slot = Slot::at(0x7651A4); // flt_7651A4
const FADE_RVA: Slot = Slot::at(0x76519C); // dword_76519C
const SIZE_K_RVA: Slot = Slot::at(0x93A352);
const LOOK: u32 = 0x34;
const LIFE: u32 = 0x3E8;

/// The rate, from the wheel's +0xC.
unsafe fn rate(wheel: u32) -> i32 {
    unsafe {
        let [k1, k2, k3, k4] = RATE_K.map(|r| game_read::<u32>(r));
        x87::speed_rate_to_int(at(wheel + 0xC), k1, k2, k3, game_read(ONE_RVA), k4)
    }
}

/// Why the Rust version refuses these registers, if it does, worked out without writing.
unsafe fn refusal(wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        if object == 0 {
            return Some("a null object");
        }
        if !gate_with(wheel, object, rate(wheel)).passes {
            return None;
        }
        // The ring full: nothing more is read.
        next_entry()?;
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null car object or matrix");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_21(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(wheel, object) {
            return Err(BadInput(why).into());
        }
        let g = gate_with(wheel, object, rate(wheel));
        game_write(RATE_RVA, g.rate);
        if let Some(count) = g.count {
            ((wheel + 4) as *mut u32).write_unaligned(count);
        }
        if !g.passes || next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, 0x15);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let r = next_c5(SIDE_RANDOM_RVA);
        let (a, b) = ((r & 1) as i32 - 1, (r & 1) as i32);
        game_write(SIDES_RVA, [a, b]);
        let place: [[u8; 10]; 3] = std::array::from_fn(|k| {
            let k = 4 * k as u32;
            x87::pair_sum_80(a, at(m + k), b, at(m + 0x20 + k), at(wheel + 0x3C + k))
        });
        put(0x18, rounded(place[2]));
        put(0x14, rounded(place[1]));
        put(0x28, x87::ext_less(place[1], game_read(LIFT_RVA)));
        put(0x10, rounded(place[0]));
        let motion: [u32; 3] = std::array::from_fn(|k| {
            let (first, third) = (at(m + 4 * k as u32), at(m + 0x20 + 4 * k as u32));
            let f = MOTION_K[k].map(|r| game_read::<u32>(r));
            x87::three_products([
                [first, at(wheel + 0x10), f[0]],
                [third, at(wheel + 0x18), f[1]],
                [third, at(wheel + 0xC), f[2]],
            ])
        });
        put(0x34, motion[2]);
        put(0x30, motion[1]);
        put(0x2C, motion[0]);
        let r = next_c5(SPIN_RANDOM_RVA);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, LOOK);
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        put(0x60, x87::from_int(((s & 0x3FFF) + 0x4000) as i32));
        put(0x84, LIFE);
        put(0x78, x87::over_int(game_read(LIFE_SCALE_RVA), LIFE as i32));
        put(0x74, game_read(FADE_RVA));
        put(0x68, car);
        let size = at(car + 0x90).wrapping_add(g.rate as u32);
        put(0x70, size);
        put(0x6C, at(car + 0x88));
        put(0x60, x87::int_times(size as i32, game_read(SIZE_K_RVA)));
    }
    Ok(())
}
