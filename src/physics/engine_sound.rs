//! `physics::engine_sound` (0x43DC72): the work car's sound figures. +0xF08
//! (`unk_71AEE0`) = speed x `flt_5C5D00`, and that (x `flt_5C5E08` with bit 8 of +0x220)
//! `fistp` into +0xAB0; +0x294 = the motion's length (`fsqrt` of z² + x² + y², as the
//! original adds them) x `flt_5C5D00`; the revs +0x1E4 eased toward the engine speed (at
//! least `flt_6140F8`, `fistp`, through `dword_62400C`) by 50/256 of the gap (`imul`,
//! `sar`), held to 2000 ..= 12000 (by the sign); then `physics::car_sound_state` and
//! `physics::gear_change_sound`.
//!
//! The x87 run is the arithmetic of the original's instructions over the game's variables
//! (from IDA), in plain Rust over `X` (`math/x87_float.rs`); the instructions are
//! `asm_reference` in the tests.
//!
//! In:  nothing
//! Out: eax the gap's step, edi the sound object (`gear_change_sound`); no flags read
//!
//! Caller: `sub_43B635` (0x43B905), every physics step. Refuses (before anything) what the
//! two calls refuse.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::math::x87_run::ext_to_int;
use crate::physics::car_sound_state::car_sound_state;
use crate::physics::car_store::CAR;
use crate::physics::gear_change_sound::gear_change_sound;
use crate::physics::tables_store::PHYSICS;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::extended;
use rc2k_core::physics::engine::ease_revs;
use rc2k_core::physics::x87::hold_low;

/// 0x43DC72 .. 0x43DCC6: the speed figures (the original's `test` / `jz` is the `if`).
fn run(course: &Course) {
    run_at(course);
}

/// The run over the work car's figures:
/// +0x31AEE0 the speed x `flt_5C5D00`; +0x31AA88 that (x `flt_5C5E08` with bit 8 of
/// +0x31A1F8) as a whole number; +0x31A26C the length of the motion (+0x31A170 ..) x
/// `flt_5C5D00`, the squares added as the original adds them (z, then x and y).
fn run_at(course: &Course) {
    CAR.engine_speed_scaled.set_bits(st(
        ld(CAR.local_velocity[2].bits()) * ld(course.numbers.two_and_a_quarter.bits())
    ));
    let speed = ld(CAR.engine_speed_scaled.bits());
    let flags = CAR.option_bits.get();
    let scaled = if flags & 0x100 != 0 {
        speed * ld(course.numbers.kilometres_in_mile.bits())
    } else {
        speed
    };
    CAR.engine_sound_speed_int.set_bits(to_int(scaled) as u32);
    let (z, x, y) = (
        ld(CAR.velocity[2].bits()),
        ld(CAR.velocity[0].bits()),
        ld(CAR.velocity[1].bits()),
    );
    let sum = z * z + (x * x + y * y);
    CAR.sound_speed
        .set_bits(st(sum.sqrt() * ld(course.numbers.two_and_a_quarter.bits())));
}

/// The Rust version.
// Converted: `rc2k_core::physics::engine::ease_revs` and `x87::hold_low` (the speed figures
// and the rounding stay x87).
pub fn engine_sound(race: &Race) -> Result<(), Refused> {
    if RACE_VIEW.players.get() == 1 && RACE_VIEW.view_block(0) == 0 {
        return Err(crate::errors::BadInput("a null view block").into());
    }
    run(&race.course);
    DRAW.scratch.set(x87::loaded(CAR.engine_speed.bits()));
    let floor = PHYSICS.idle_engine_speed.bits();
    let held = hold_low(f32::from_bits(DRAW.scratch.get()), f32::from_bits(floor));
    DRAW.scratch.set(held.to_bits());
    DRAW.scratch
        .set(ext_to_int(extended(DRAW.scratch.get())) as u32);
    let (revs, _) = ease_revs(CAR.engine_sound_revs.get(), DRAW.scratch.get());
    CAR.engine_sound_revs.set(revs);
    { car_sound_state(race) }?;
    gear_change_sound()
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    reference!(ref_0(rc)
        "fld dword ptr [{rc} + 0x31A184]",
        "fmul dword ptr [{rc} + 0x1C5D00]",
        "fstp dword ptr [{rc} + 0x31AEE0]",
        "fld dword ptr [{rc} + 0x31AEE0]",
        "test dword ptr [{rc} + 0x31A1F8], 0x100",
        "jz 2f",
        "fmul dword ptr [{rc} + 0x1C5E08]",
        "2:",
        "fistp dword ptr [{rc} + 0x31AA88]",
        "fld dword ptr [{rc} + 0x31A178]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{rc} + 0x31A170]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{rc} + 0x31A174]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fmul dword ptr [{rc} + 0x1C5D00]",
        "fstp dword ptr [{rc} + 0x31A26C]"
    );

    #[test]
    fn run_matches_the_assembly() {
        let blocks = vec![Block {
            name: "engine_sound run",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x31A184, In::F),
                    (0x1C5D00, In::F),
                    (0x1C5E08, In::F),
                    (0x31A1F8, In::I),
                    (0x31A178, In::F),
                    (0x31A170, In::F),
                    (0x31A174, In::F),
                ],
                writes: &[(0x31AEE0, Out::F), (0x31AA88, Out::I), (0x31A26C, Out::F)],
            }],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0]) },
            plain: |_| run_at(&crate::race::owner::race().course),
        }];
        sparse(&blocks, 4000, 200);
    }
}
