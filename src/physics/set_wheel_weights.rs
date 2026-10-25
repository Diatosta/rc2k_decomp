//! `physics::set_wheel_weights` (0x43F3E7): the four wheels' load forces (the records at
//! 0x614770 + 0x20 k, `physics::clear_forces`'): a share `flt_6D3070` held to 1 ..
//! `flt_5C5D48` (through `dword_62400C`), ((it - 1) `flt_5C5DA4` + 1) gravity
//! (`flt_613FC8`) `flt_614170`, negated, times `flt_5C5DA4` again into each record's +4,
//! turned into the car's frame (`physics::turn_by_inverse`); then each record's second
//! vector (+0xC ..) set to (-1 / 1, 0, ±1) by its corner and turned the same way. A
//! reading: the weight on each wheel, the corners' places.
//!
//! In:  nothing
//! Out: esi 0x6147DC (the last turned); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B723), every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::framework::{game_addr, rva};
use crate::math::x87;
use crate::math::x87_run::{compare_full, weight_force};
use crate::physics::tables_store::PHYSICS;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;

/// The share, the scratch, the weight, the records (0x80 bytes).
pub struct Places {
    pub share: u32,
    pub scratch: u32,
    pub weight: u32,
    pub records: u32,
}

impl Places {
    fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            share: a(0x6D3070),
            scratch: (DRAW.scratch.addr() as u32),
            weight: (PHYSICS.weight.addr() as u32),
            records: ((PHYSICS.forces.as_ptr() as usize) as u32),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `p`. The registers the original leaves are read by none of its callers (esi).
pub unsafe fn weights_at(p: &Places) {
    unsafe {
        let one: u32 = NUMBERS.one.bits();
        let high: u32 = NUMBERS.five.bits();
        let k: u32 = NUMBERS.quarter.bits();
        put(p.scratch, x87::loaded(at(p.share)));
        if compare_full(at(p.scratch), one) & 0x100 != 0 {
            put(p.scratch, x87::loaded(one));
        }
        if compare_full(at(p.scratch), high) & 0x100 == 0 {
            put(p.scratch, x87::loaded(high));
        }
        let f = weight_force(at(p.scratch), one, k, PHYSICS.gravity.bits(), at(p.weight));
        for c in 0..4 {
            put(p.records + 0x20 * c + 4, f);
        }
        for c in 0..4 {
            turn_by_inverse(p.records + 0x20 * c);
        }
        let plus = x87::loaded(1.0f32.to_bits());
        let minus = x87::negated(plus);
        for o in [0x2C, 0x6C, 0x14, 0x34] {
            put(p.records + o, plus);
        }
        for o in [0x0C, 0x4C, 0x54, 0x74] {
            put(p.records + o, minus);
        }
        for c in 0..4 {
            turn_by_inverse(p.records + 0x20 * c + 0xC);
        }
    }
}

/// The Rust version.
pub unsafe fn set_wheel_weights() {
    unsafe { weights_at(&Places::game()) }
}
