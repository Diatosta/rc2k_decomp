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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::emitters::*;
use crate::marks::setup::SEED_RVA;
use crate::math::x87;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at as at_rva, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::emit_particle_6", rva(0x4CDC74), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.edi), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// A ring, a wheel, a car object (with its matrix) of a test's; the game's variables
/// they replace kept and put back on drop.
pub struct TestEmitter {
    pub ring: Vec<u8>,
    pub wheel: Vec<u32>,
    pub object: Vec<u32>,
    pub matrix: Vec<u32>,
    saved: Vec<u8>,
    inputs: Vec<Region>,
}

pub const TEST_ENTRIES: u32 = 0x20;

impl TestEmitter {
    pub fn new(extra: &[usize]) -> Self {
        let mut inputs: Vec<Region> = [
            OBJECT_RVA,
            TAG_RVA,
            FLAGS_RVA,
            RATE_RVA,
            SEED_RVA,
            RING_CURSOR_RVA,
            RING_COUNT_RVA,
        ]
        .map(at_rva)
        .to_vec();
        inputs.push((crate::framework::game_addr(SIDES_RVA) as u32, 8));
        inputs.extend(extra.iter().map(|&a| at_rva(a)));
        let saved = capture(&inputs);
        Self {
            ring: vec![0; (TEST_ENTRIES as usize) * ENTRY_LEN],
            wheel: vec![0; 0x20],
            object: vec![0; 0x30],
            matrix: vec![0; 0x10],
            saved,
            inputs,
        }
    }

    /// The ring linked (+0 the next taken), the cursor at entry 3, `count` taken.
    pub fn prepare(&mut self, count: u32) {
        let base = self.ring.as_mut_ptr() as u32;
        self.ring.fill(0xAA);
        for n in 0..TEST_ENTRIES {
            let e = (base + n * ENTRY_LEN as u32) as *mut u32;
            let next = base + ((n + 1) % TEST_ENTRIES) * ENTRY_LEN as u32;
            unsafe { e.write_unaligned(next) };
        }
        unsafe {
            game_write(RING_CURSOR_RVA, base + 3 * ENTRY_LEN as u32);
            game_write(RING_COUNT_RVA, count);
            self.object[0x24 / 4] = self.matrix.as_ptr() as u32;
            game_write(OBJECT_RVA, self.object.as_ptr() as u32);
        }
    }

    pub fn regions(&self) -> Vec<Region> {
        let mut r = self.inputs.clone();
        r.push((self.ring.as_ptr() as u32, self.ring.len()));
        r.push((self.wheel.as_ptr() as u32, 4 * self.wheel.len()));
        r
    }
}

impl Drop for TestEmitter {
    fn drop(&mut self) {
        restore(&self.inputs, &self.saved);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut test = TestEmitter::new(&[SIDE_RANDOM_RVA, SPIN_RANDOM_RVA]);
    let regions = test.regions();
    let mut case = 0u32;
    let mut made = 0;
    // (flags, speeds +0xC, +0x10, +0x18, the object's +0x90, the wheel's +4, taken)
    for (flags, speeds, size, count, taken) in [
        (0x40u32, [3.0f32, 0.5, -2.0], 0x2000u32, 0x5000u32, 0u32),
        (0xC0, [-10.0, 1.0, 4.0], 0x1800, 0x7000, 5),
        (0x80, [0.0, 0.0, 0.0], 0x4000, 0x9000, 0),
        (0x3F, [1.0, 1.0, 1.0], 0x4000, 0x9000, 0),
        (0xBF, [2.0, -0.0, 0.0], 0x0800, 0x9000, 0),
        (0x40, [5.0, 2.0, 1.0], 0x2000, 0x10, 0),
        (0xC1, [5.0, 2.0, 1.0], 0x2000, 0x5000, MOST_TAKEN),
        (0x81, [0.25, 7.0, -3.0], 0x7FFF_F000, 0x7FFF_FFFF, 1),
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
            &mut t,
            &format!(
                "flags {flags:#x}, speeds {speeds:?}, size {size:#x}, count {count:#x}, taken {taken:#x}"
            ),
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
                game_write(SIDE_RANDOM_RVA, seed ^ 0x55);
                game_write(SPIN_RANDOM_RVA, seed ^ 0xAA);
                let w = ptrs.0;
                for n in 0..0x20 {
                    *w.add(n) = (n as f32 * 0.75 - 3.0).to_bits();
                }
                *w.add(1) = count;
                *w.add(3) = speeds[0].to_bits();
                *w.add(4) = speeds[1].to_bits();
                *w.add(6) = speeds[2].to_bits();
                *w.add(8) = flags;
                let o = ptrs.1;
                for n in 0..0x30 {
                    *o.add(n) = (n as f32 * 1.25 + 0.5).to_bits();
                }
                *o.add(0x1C / 4) = 0x1234_5678;
                *o.add(0x24 / 4) = ptrs.2 as u32;
                *o.add(0x88 / 4) = 0x77;
                *o.add(0x90 / 4) = size;
                for n in 0..0x10 {
                    *ptrs.2.add(n) = ((n as f32 - 5.0) * 0.3).to_bits();
                }
            },
            &regions,
            &rust,
        );
        if unsafe { game_read::<u32>(RING_COUNT_RVA) } != taken {
            made += 1;
        }
    }
    // The cases must reach the entry's making at least once.
    t.check(made > 0, || {
        "no case made an entry (the rate gate never passed)".into()
    });
    // The refusal: flags of 0x100 or more when an entry is made.
    test.prepare(0);
    let refused = unsafe {
        let w = test.wheel.as_mut_ptr();
        *w.add(8) = 0x1C0;
        *w.add(1) = 0x5000;
        *w.add(3) = 3.0f32.to_bits();
        *w.add(4) = 1.0f32.to_bits();
        test.object[0x90 / 4] = 0x2000;
        let object = test.object.as_ptr() as u32;
        let made = gate(w as u32, object, 0xC, 0x18, RATE_K).passes;
        (made, emit_particle_6(w as u32, object).is_err())
    };
    t.check(refused.1 == refused.0, || {
        format!(
            "flags 0x1C0, an entry made {}: refused {}",
            refused.0, refused.1
        )
    });
    drop(test);
    t.finish()
}
