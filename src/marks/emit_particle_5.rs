//! `marks::emit_particle_5` (0x4CC15D, a chunk of `marks::emit_wheel_particles` entered by
//! `jz` from it and from 0x4CA20B, left by `popa; retn`): one ring entry of kind 5 from a
//! wheel whose +0x18 is not 0.
//!
//! In:  edi the wheel record, esi the car object
//! Out: every register kept
//!
//! The strength `flt_93555E`: |the wheel's +0x18|, then less (bit 0 of `dword_935416`
//! clear, `dword_935432` -1) or plus (set, 1) its +0x10. The entry: its place the wheel's
//! (+0x3C ..) less the car matrix's (+0x30 ..) plus a random side (`dword_93554A` /
//! `dword_93554E`: the matrix's first and third rows by a random's low 14 bits, the first
//! less 0x2000); its motion the matrix's first row by `flt_93A23E` .. and the sign, by the
//! strength (the y plus `flt_93A246`), moved by `marks::jitter_point` (spread
//! `flt_93A24E`), plus the third row by `flt_93A252` .. and the wheel's +0x18; +0x54
//! `flt_93A25E` times the sign; +0x58 the wheel's flags' low 6 bits; +0x60 the integer
//! ((`jitter_point`'s ebx >> 2) & 0xFFF) + 0x7800 (section 16); +0x74 0xFF000000; a life
//! of 5000; +0x68 the car. Refuses (before anything) a null wheel, object, car or matrix
//! where read. The body is shared with kind 0xB's (`JitterSet`, `emit_particle_5_with`,
//! `test_with`). Self-test: a ring, wheel and car of the test's
//! (`marks::emit_particle_6::TestEmitter`), several strengths, signs and fillings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, game_read, game_write, rva};
use crate::marks::emit_particle_6::{TEST_ENTRIES, TestEmitter};
use crate::marks::emitters::*;
use crate::marks::jitter_point::jitter_point;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at as at_rva, check_state, restore, state_entry};

const STRENGTH_RVA: usize = rva(0x93555E); // g_marksStrength
const SIGN_RVA: usize = rva(0x935432); // g_marksSign: -1 or 1
const SIDE_BIT_RVA: usize = rva(0x935416); // dword_935416: bit 0 the side
/// One emitter's variables (kind 5's and kind 0xB's code is the same but for these).
pub struct JitterSet {
    pub kind: u32,
    /// Whether the rounded |speed| must be over 1 (kind 0xB) or the speed not 0 (kind 5).
    pub over_one: bool,
    pub place_random: usize,
    /// The motion's factors (x, y, z); `lift` the y's.
    pub motion_k: [usize; 3],
    pub lift: usize,
    pub spread: usize,
    /// The third row's factors (z, y, x).
    pub roll_k: [usize; 3],
    pub spin: usize,
    pub life: u32,
}

/// Kind 5's (`dword_93A23A`, `flt_93A23E` .. `flt_93A25E`).
pub const FIFTH: JitterSet = JitterSet {
    kind: 5,
    over_one: false,
    place_random: rva(0x93A23A),
    motion_k: [rva(0x93A23E), rva(0x93A242), rva(0x93A24A)],
    lift: rva(0x93A246),
    spread: rva(0x93A24E),
    roll_k: [rva(0x93A252), rva(0x93A256), rva(0x93A25A)],
    spin: rva(0x93A25E),
    life: 0x1388,
};

/// Why the Rust version refuses these registers, if it does, worked out without writing.
pub unsafe fn refusal(set: &JitterSet, wheel: u32, object: u32) -> Option<&'static str> {
    unsafe {
        if wheel == 0 {
            return Some("a null wheel");
        }
        if !emits(set, wheel) {
            return None;
        }
        if object == 0 {
            return Some("a null object");
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

/// Whether the wheel's +0x18 passes the set's test.
unsafe fn emits(set: &JitterSet, wheel: u32) -> bool {
    let speed = unsafe { at(wheel + 0x18) } & 0x7FFF_FFFF;
    if set.over_one {
        x87::to_int(speed) > 1
    } else {
        x87::loaded(speed) != 0
    }
}

/// The Rust version (see the header).
pub unsafe fn emit_particle_5(wheel: u32, object: u32) -> Result<(), Refused> {
    unsafe { emit_particle_5_with(&FIFTH, wheel, object) }
}

/// The shared body, on `set`'s variables.
pub unsafe fn emit_particle_5_with(
    set: &JitterSet,
    wheel: u32,
    object: u32,
) -> Result<(), Refused> {
    unsafe {
        if let Some(why) = refusal(set, wheel, object) {
            return Err(BadInput(why).into());
        }
        let strength = x87::loaded(at(wheel + 0x18)) & 0x7FFF_FFFF;
        game_write(STRENGTH_RVA, strength);
        if !emits(set, wheel) || next_entry().is_none() {
            return Ok(());
        }
        let e = take_entry(at(object + 0x1C) & 0xFF_FFFF, set.kind);
        let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        let (sign, strength) = if game_read::<u32>(SIDE_BIT_RVA) & 1 == 0 {
            (-1i32, x87::sub(strength, at(wheel + 0x10)))
        } else {
            (1, x87::add(strength, at(wheel + 0x10)))
        };
        game_write(SIGN_RVA, sign);
        game_write(STRENGTH_RVA, strength);
        let car: u32 = game_read(OBJECT_RVA);
        let m = at(car + 0x24);
        let r = next_c5(set.place_random);
        let (a, b) = (((r & 0x3FFF) as i32) - 0x2000, (r & 0x3FFF) as i32);
        game_write(SIDES_RVA, [a, b]);
        let place: [u32; 3] = std::array::from_fn(|k| {
            let k = 4 * k as u32;
            x87::pair_sum_less(
                a,
                at(m + k),
                b,
                at(m + 0x20 + k),
                at(wheel + 0x3C + k),
                at(m + 0x30 + k),
            )
        });
        put(0x18, place[2]);
        put(0x28, place[1]);
        put(0x14, place[1]);
        put(0x10, place[0]);
        let motion: [[u8; 10]; 3] = std::array::from_fn(|k| {
            let lift = (k == 1).then(|| game_read::<u32>(set.lift));
            x87::times_int_plus_times_80(
                at(m + 4 * k as u32),
                game_read(set.motion_k[k]),
                sign,
                lift,
                strength,
            )
        });
        let (moved, last) = jitter_point([
            extended(game_read(set.spread)),
            motion[2],
            motion[1],
            motion[0],
        ]);
        let speed = at(wheel + 0x18);
        for (n, o) in [0x34u32, 0x30, 0x2C].into_iter().enumerate() {
            let row = at(m + 0x28 - 4 * n as u32);
            put(
                o,
                x87::product_plus(row, game_read(set.roll_k[n]), speed, rounded(moved[n])),
            );
        }
        put(
            0x54,
            x87::float_times_int_to_int(game_read(set.spin), sign) as u32,
        );
        put(0x58, at(wheel + 0x20) & 0x3F);
        put(0x60, ((last >> 2) & 0xFFF) + 0x7800);
        put(0x74, 0xFF00_0000);
        put(0x84, set.life);
        put(0x68, car);
    }
    Ok(())
}

/// What it may write: the globals and, when one is to be taken, the next entry.
pub fn regions(set: &JitterSet) -> Vec<Region> {
    let mut r: Vec<Region> = [
        STRENGTH_RVA,
        SIGN_RVA,
        set.place_random,
        SEED_RVA,
        RING_CURSOR_RVA,
        RING_COUNT_RVA,
    ]
    .map(at_rva)
    .to_vec();
    r.push((crate::framework::game_addr(SIDES_RVA) as u32, 8));
    if let Some(e) = unsafe { next_entry() } {
        r.push((e, ENTRY_LEN));
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { emit_particle_5(r.edi, r.esi) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The test of either emitter.
pub fn test_with(
    t: &mut SelfTest,
    set: &JitterSet,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::service::harness::Rust,
) {
    let mut test = TestEmitter::new(&[STRENGTH_RVA, SIGN_RVA, SIDE_BIT_RVA, set.place_random]);
    let regions = test.regions();
    let mut made = 0;
    let mut case = 0u32;
    // (the wheel's +0x18, +0x10, the side bit, taken)
    for (speed, push, side, taken) in [
        (3.0f32, 0.5f32, 0u32, 0u32),
        (-10.0, 4.0, 1, 5),
        (0.0, 1.0, 0, 0),
        (-0.0, 1.0, 1, 0),
        (1.4, 1.0, 1, 0),
        (-1.6, 1.0, 0, 0),
        (2.5e-3, -7.0, 3, 0x10),
        (1e6, 1e6, 2, 0),
        (5.0, 2.0, 1, MOST_TAKEN),
    ] {
        case += 1;
        let ptrs = (
            test.wheel.as_mut_ptr(),
            test.object.as_mut_ptr(),
            test.matrix.as_mut_ptr(),
        );
        let seed = case.wrapping_mul(0x2545_F491);
        test.prepare(taken);
        check_state(
            t,
            &format!("speed {speed}, push {push}, side {side}, taken {taken:#x}"),
            call_original,
            Regs {
                edi: ptrs.0 as u32,
                esi: ptrs.1 as u32,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &vec![0xAA; regions.iter().map(|r| r.1).sum()]);
                let base = test.ring.as_ptr() as u32;
                for n in 0..TEST_ENTRIES {
                    let e = (base + n * ENTRY_LEN as u32) as *mut u32;
                    e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * ENTRY_LEN as u32);
                }
                game_write(RING_CURSOR_RVA, base + 3 * ENTRY_LEN as u32);
                game_write(RING_COUNT_RVA, taken);
                game_write(OBJECT_RVA, ptrs.1 as u32);
                game_write(SEED_RVA, seed);
                game_write(set.place_random, seed ^ 0x55);
                game_write(SIDE_BIT_RVA, side);
                let w = ptrs.0;
                for n in 0..0x20 {
                    *w.add(n) = (n as f32 * 0.75 - 3.0).to_bits();
                }
                *w.add(4) = push.to_bits();
                *w.add(6) = speed.to_bits();
                *w.add(8) = 0x1234_56C7;
                let o = ptrs.1;
                for n in 0..0x30 {
                    *o.add(n) = (n as f32 * 1.25 + 0.5).to_bits();
                }
                *o.add(0x1C / 4) = 0x1234_5678;
                *o.add(0x24 / 4) = ptrs.2 as u32;
                for n in 0..0x10 {
                    *ptrs.2.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
                }
            },
            &regions,
            rust,
        );
        if unsafe { game_read::<u32>(RING_COUNT_RVA) } != taken {
            made += 1;
        }
    }
    t.check(made > 0, || "no case made an entry".into());
    drop(test);
}
