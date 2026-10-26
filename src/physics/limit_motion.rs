//! `physics::limit_motion` (0x43F7AC): the work car's motion held to its limits. A motion
//! part that is no number or infinite (its exponent all ones) logs "*** INFINITY",
//! "[car0.newxspd] =" and the three (5 decimals) and zeroes the motion (+0x198), its car
//! frame copy (+0x1A4) and the spin (+0x184). Then the spin's parts held to
//! ±`flt_613E70` and the motion's (x, z, y) to ±`flt_613E74` (the limit negated in place
//! for the second compare and put back); its length (`physics::vector_length`) into
//! `flt_623FBC`, and past `flt_614100` the motion and its copy scaled down to it
//! (`physics::scale_vector`, the factor then in `dword_62400C`).
//!
//! The x87 compares are the arithmetic of the original's instructions over the game's
//! variables (from IDA), in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! `asm_reference` in the tests.
//!
//! In:  nothing
//! Out: ecx the last part's exponent bits, esi +0x198 (+0x1A4 after a scaling); no
//!      flags read
//!
//! Callers: `sub_43B635` (twice), `sub_43BDD1`.

use crate::logfile::log_float::log_float;
use crate::logfile::log_write::log_write;
use crate::math::x87_float::compare;
use crate::physics::car_store::CAR;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::PHYSICS;
use crate::physics::vector_length::vector_length;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::rounded;
use crate::scene::x87_mem::{fld, fstp};
use std::cmp::Ordering;

/// The value at `v` held to ±(the limit at `l`), as the original: `fld l; fld v;
/// fcompp`, the limit negated in place for the second compare and put back.
fn hold(v: u32, l: u32) {
    hold_at(v as usize, l as usize);
}

/// `hold` over the dwords at `v` and `l`: `v` becomes the limit unless it is below it (or no
/// number); the limit is stored negated, `v` becomes that unless it is not below it, and the
/// limit is stored negated again.
fn hold_at(v: usize, l: usize) {
    let (v, l) = (crate::pointers::host_at(v), crate::pointers::host_at(l));
    let limit = fld(l, 0);
    if compare(fld(v, 0), limit) & 0x100 == 0 {
        fstp(v, 0, limit);
    }
    fstp(l, 0, -fld(l, 0));
    let negated = fld(l, 0);
    if compare(fld(v, 0), negated) & 0x100 != 0 {
        fstp(v, 0, negated);
    }
    fstp(l, 0, -fld(l, 0));
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::find_clear_place`).
pub unsafe fn limit_motion() {
    let mut bad = false;
    for k in 0..3usize {
        let ecx = CAR.velocity[k].bits() & 0x7F80_0000;
        if ecx == 0x7F80_0000 {
            bad = true;
            break;
        }
    }
    if bad {
        unsafe {
            log_write(c"*** INFINITY".as_ptr() as *const u8, false, 1);
            log_write(c"[car0.newxspd] =".as_ptr() as *const u8, false, 1);
            for k in 0..3usize {
                log_float(f32::from_bits(CAR.velocity[k].bits()), 5, k == 2, 1);
            }
        }
        for k in 0..3usize {
            CAR.velocity[k].set_bits(0);
            CAR.local_velocity[k].set_bits(0);
            CAR.spin[k].set_bits(0);
        }
    }
    for k in 0..3usize {
        hold(CAR.spin[k].addr(), PHYSICS.spin_limit.addr());
    }
    for k in [0usize, 2, 1] {
        hold(CAR.velocity[k].addr(), PHYSICS.velocity_limit.addr());
    }
    let length = rounded(vector_length(CAR.velocity[0].addr()));
    DRAW.motion_length.set_bits(length);
    let top = PHYSICS.top_speed.bits();
    let past = matches!(
        f32::from_bits(top).partial_cmp(&f32::from_bits(length)),
        Some(Ordering::Less)
    ) || f32::from_bits(top).is_nan()
        || f32::from_bits(length).is_nan();
    if past {
        let factor = crate::math::x87_run::div_80(top, length);
        scale_vector(CAR.velocity[0].addr(), factor);
        scale_vector(CAR.local_velocity[0].addr(), factor);
        DRAW.scratch.set(rounded(factor));
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference_eax, sparse};

    reference_eax!(ref_0(v, l)
        "fld dword ptr [{l}]",
        "fld dword ptr [{v}]",
        ".byte 0xDE, 0xD9",
        "fnstsw ax",
        "shr ah, 1",
        "jb 2f",
        "fld dword ptr [{l}]",
        "fstp dword ptr [{v}]",
        "2:",
        "fld dword ptr [{l}]",
        ".byte 0xD9, 0xE0",
        "fstp dword ptr [{l}]",
        "fld dword ptr [{l}]",
        "fld dword ptr [{v}]",
        ".byte 0xDE, 0xD9",
        "fnstsw ax",
        "shr ah, 1",
        "jnb 3f",
        "fld dword ptr [{l}]",
        "fstp dword ptr [{v}]",
        "3:",
        "fld dword ptr [{l}]",
        ".byte 0xD9, 0xE0",
        "fstp dword ptr [{l}]"
    );

    #[test]
    fn hold_matches_the_assembly() {
        let blocks = vec![Block {
            name: "limit_motion hold",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| hold_at(a[0], a[1]),
        }];
        sparse(&blocks, 6000, 201);
    }
}
