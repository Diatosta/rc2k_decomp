//! `marks::emit_particle_6` (0x4CDC74): one ring entry of kind 6 from a wheel, when its
//! flags (+0x20, kept in `dword_939BAE`) have 0xC0 and the rate gate passes
//! (`marks/emitters.rs`, the wheel's +0xC less +0x18 by `flt_93A41A` ..).
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The entry: its place the wheel's (+0x3C ..) plus a random side (-1 or 0 times the
//! object's matrix's first row, 0 or 1 times its third) less 0x93A42E in y at +0x28; its
//! motion three sums of products of the wheel's speeds; a random spin and turn (+0x54,
//! +0x50), the flags' low 6 bits (+0x58), a size by the marks' random (+0x60, then
//! replaced), a life from `dword_939B76` by the flags' bits 6 and 7 (+0x84; +0x78
//! `flt_7651A4` over it), +0x74 `dword_76519C`, the object (+0x68) and its +0x88, +0x90.
//! Callers: `marks::emit_wheel_particles`' kinds 2 .. 4, 6 .. 9, 0xC (after `sub_4CD21A`
//! or `sub_4CD747`).
//! Refuses (before anything) a null wheel, object or matrix where read, and flags of
//! 0x100 or more when an entry is made (the life read past `dword_939B76`'s four).
//! Self-test: a ring, wheel and object of the test's; the original and the Rust version
//! at several speeds, flags and fillings; put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, state_entry};

const RATE_K: [usize; 4] = [rva(0x93A41A), rva(0x93A41E), rva(0x93A422), rva(0x93A426)];
const SIDE_RANDOM_RVA: usize = rva(0x93A42A);
const LIFT_RVA: usize = rva(0x93A42E);
/// The motion: per axis (x, y, z) the three (factor, wheel speed, scale) products.
const MOTION: [[(usize, u32, usize); 3]; 3] = [
    [
        (0x93545E, 0x10, 0x93A432),
        (0x93543A, 0x18, 0x93A436),
        (0x93543A, 0x0C, 0x93A43A),
    ],
    [
        (0x935462, 0x10, 0x93A43E),
        (0x93543E, 0x18, 0x93A442),
        (0x93543E, 0x0C, 0x93A446),
    ],
    [
        (0x935466, 0x10, 0x93A44A),
        (0x935442, 0x18, 0x93A44E),
        (0x935442, 0x0C, 0x93A452),
    ],
];
const SPIN_RANDOM_RVA: usize = rva(0x93A456);
const LIVES_RVA: usize = rva(0x939B76); // g_marksParticleLives: by the flags' bits 6, 7
const LIFE_SCALE_RVA: usize = rva(0x7651A4); // flt_7651A4
const FADE_RVA: usize = rva(0x76519C); // dword_76519C
const SIZE_K: [usize; 2] = [rva(0x93A45A), rva(0x93A45E)];

/// Why the Rust version refuses these registers, if it does, worked out without writing.
unsafe fn refusal(wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        let flags = at(wheel + 0x20);
        if flags & 0xC0 == 0 {
            return None;
        }
        if object == 0 {
            return Some("a null object");
        }
        if !gate(wheel, object, 0xC, 0x18, RATE_K).passes || next_entry().is_none() {
            return None;
        }
        let car: u32 = game_read(OBJECT_RVA);
        if car == 0 || at(car + 0x24) == 0 {
            return Some("a null car object or matrix");
        }
        if flags >> 6 > 3 {
            return Some("wheel flags of 0x100 or more: the life read past dword_939B76");
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_6(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(wheel, object) {
            return Err(BadInput(why).into());
        }
        let flags = at(wheel + 0x20);
        game_write(FLAGS_RVA, flags);
        if flags & 0xC0 == 0 {
            return Ok(());
        }
        let g = gate(wheel, object, 0xC, 0x18, RATE_K);
        game_write(RATE_RVA, g.rate);
        if let Some(count) = g.count {
            ((wheel + 4) as *mut u32).write_unaligned(count);
        }
        if !g.passes || next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, 6);
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
        let motion = MOTION.map(|axis| {
            x87::three_products(
                axis.map(|(f, s, k)| [game_read(rva(f)), at(wheel + s), game_read(rva(k))]),
            )
        });
        put(0x34, motion[2]);
        put(0x30, motion[1]);
        put(0x2C, motion[0]);
        let r = next_c5(SPIN_RANDOM_RVA);
        put(0x54, (r & 0x3F).wrapping_sub(0x20));
        put(0x50, r.rotate_right(6) & 0xFFF);
        put(0x58, flags & 0x3F);
        let s = game_read::<u32>(SEED_RVA)
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        game_write(SEED_RVA, s);
        put(0x60, x87::from_int(((s & 0x3FFF) + 0x4000) as i32));
        let life: u32 = game_read(LIVES_RVA + 4 * (flags >> 6) as usize);
        put(0x84, life);
        put(0x78, x87::over_int(game_read(LIFE_SCALE_RVA), life as i32));
        put(0x74, game_read(FADE_RVA));
        put(0x68, car);
        let size = at(car + 0x90).wrapping_add(g.rate as u32);
        put(0x70, size);
        put(0x6C, at(car + 0x88));
        put(
            0x60,
            x87::int_times_plus(size as i32, game_read(SIZE_K[0]), game_read(SIZE_K[1])),
        );
    }
    Ok(())
}

/// What it may write: the globals and, when one is to be taken, the next entry.
pub fn regions(wheel: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [
        FLAGS_RVA,
        RATE_RVA,
        SIDE_RANDOM_RVA,
        SPIN_RANDOM_RVA,
        SEED_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec();
    r.push((crate::framework::game_addr(SIDES_RVA) as u32, 8));
    if wheel != 0 {
        r.push((wheel + 4, 4));
    }
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_6(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
