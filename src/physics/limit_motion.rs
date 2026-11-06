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
use crate::math::x87_float::compare;
use crate::math::x87_float::{ld, st};
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::race::owner::Race;
use crate::scene::matrix3::rounded;
use crate::scene::owner::scene;
use crate::session::Cell32;
use std::cmp::Ordering;

/// The value at `v` held to ±(the limit at `l`), as the original: `fld l; fld v;
/// fcompp`, the limit negated in place for the second compare and put back.
fn hold(v: &impl Cell32, l: &impl Cell32) {
    let limit = ld(l.bits());
    if compare(ld(v.bits()), limit) & 0x100 == 0 {
        v.set_bits(st(limit));
    }
    l.set_bits(st(-ld(l.bits())));
    let negated = ld(l.bits());
    if compare(ld(v.bits()), negated) & 0x100 != 0 {
        v.set_bits(st(negated));
    }
    l.set_bits(st(-ld(l.bits())));
}

/// The Rust version (the exact one, for `physics::car_physics_step` and
/// `physics::find_clear_place`).
pub fn limit_motion(race: &Race) {
    let drawing = &scene().drawing;
    let mut bad = false;
    for k in 0..3usize {
        let ecx = race.shared.car.velocity[k].bits() & 0x7F80_0000;
        if ecx == 0x7F80_0000 {
            bad = true;
            break;
        }
    }
    if bad {
        crate::logfile::log_write::log_nul(&c"*** INFINITY", false, 1);
        crate::logfile::log_write::log_nul(&c"[car0.newxspd] =", false, 1);
        for k in 0..3usize {
            log_float(
                f32::from_bits(race.shared.car.velocity[k].bits()),
                5,
                k == 2,
                1,
            );
        }

        for k in 0..3usize {
            race.shared.car.velocity[k].set_bits(0);
            race.shared.car.local_velocity[k].set_bits(0);
            race.shared.car.spin[k].set_bits(0);
        }
    }
    for k in 0..3usize {
        hold(
            &race.shared.car.spin[k],
            &race.shared.physics_tables.spin_limit,
        );
    }
    for k in [0usize, 2, 1] {
        hold(
            &race.shared.car.velocity[k],
            &race.shared.physics_tables.velocity_limit,
        );
    }
    let length = rounded(vector_length(&race.shared.car.velocity));
    drawing.scene.motion_length.set_bits(length);
    let top = race.shared.physics_tables.top_speed.bits();
    let past = matches!(
        f32::from_bits(top).partial_cmp(&f32::from_bits(length)),
        Some(Ordering::Less)
    ) || f32::from_bits(top).is_nan()
        || f32::from_bits(length).is_nan();
    if past {
        let factor = crate::math::x87_run::div_80(top, length);
        scale_vector(&race.shared.car.velocity, factor);
        scale_vector(&race.shared.car.local_velocity, factor);
        drawing.scene.scratch.set(rounded(factor));
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
            plain: |a| {
                hold(
                    &crate::scene::x87_mem::HostCell(a[0]),
                    &crate::scene::x87_mem::HostCell(a[1]),
                );
            },
        }];
        sparse(&blocks, 6000, 201);
    }
}
