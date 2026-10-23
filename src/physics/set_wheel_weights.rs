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
//! Self-test: a copy of the original on test records (its eight calls to
//! `turn_by_inverse` kept), shares below, in and above the range under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::math::x87_run::{compare_full, weight_force};
use crate::physics::turn_by_inverse::turn_by_inverse;

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
            scratch: a(0x62400C),
            weight: a(0x614170),
            records: a(0x614770),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `p`; `r` as the original leaves it.
pub unsafe fn weights_at(p: &Places, r: &mut Regs) {
    unsafe {
        let one: u32 = game_read(rva(0x5C5D38));
        let high: u32 = game_read(rva(0x5C5D48));
        let k: u32 = game_read(rva(0x5C5DA4));
        put(p.scratch, x87::loaded(at(p.share)));
        if compare_full(at(p.scratch), one) & 0x100 != 0 {
            put(p.scratch, x87::loaded(one));
        }
        if compare_full(at(p.scratch), high) & 0x100 == 0 {
            put(p.scratch, x87::loaded(high));
        }
        let f = weight_force(
            at(p.scratch),
            one,
            k,
            game_read(rva(0x613FC8)),
            at(p.weight),
        );
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
            r.esi = p.records + 0x20 * c + 0xC;
            turn_by_inverse(r.esi);
        }
    }
}

/// The Rust version; `r` as the original leaves it.
pub unsafe fn set_wheel_weights(r: &mut Regs) {
    unsafe { weights_at(&Places::game(), r) }
}
