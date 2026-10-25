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
use crate::math::x87_run::ext_to_int;
use crate::physics::car_sound_state::car_sound_state;
use crate::physics::car_store::CAR;
use crate::physics::gear_change_sound::gear_change_sound;
use crate::physics::tables_store::PHYSICS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::extended;
use crate::scene::x87_mem::{fistp, fld, fstp};
use rc2k_core::physics::engine::ease_revs;
use rc2k_core::physics::x87::hold_low;

fn g(x: impl crate::store::At) -> u32 {
    unsafe { crate::store::at_read(x) }
}

/// 0x43DC72 .. 0x43DCC6: the speed figures (the original's `test` / `jz` is the `if`).
unsafe fn run() {
    run_at(crate::race::store::asm_base() as usize);
}

/// The run over the race core's variables at `rc` (the dwords at their offsets from the base):
/// +0x31AEE0 the speed x `flt_5C5D00`; +0x31AA88 that (x `flt_5C5E08` with bit 8 of
/// +0x31A1F8) as a whole number; +0x31A26C the length of the motion (+0x31A170 ..) x
/// `flt_5C5D00`, the squares added as the original adds them (z, then x and y).
fn run_at(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        fstp(rc, 0x31AEE0, fld(rc, 0x31A184) * fld(rc, 0x1C5D00));
        let speed = fld(rc, 0x31AEE0);
        let scaled = if rc.add(0x31A1F8).cast::<u32>().read_unaligned() & 0x100 != 0 {
            speed * fld(rc, 0x1C5E08)
        } else {
            speed
        };
        fistp(rc, 0x31AA88, scaled);
        let (z, x, y) = (fld(rc, 0x31A178), fld(rc, 0x31A170), fld(rc, 0x31A174));
        let sum = z * z + (x * x + y * y);
        fstp(rc, 0x31A26C, sum.sqrt() * fld(rc, 0x1C5D00));
    }
}

/// The Rust version.
// Converted: `rc2k_core::physics::engine::ease_revs` and `x87::hold_low` (the speed figures
// and the rounding stay x87).
pub unsafe fn engine_sound() -> Result<(), Refused> {
    let o =
        crate::sound::store::RACE_SOUNDS.table[(RACE_WORK.current_car.get() & 7) as usize].get();
    if o == 0 {
        return Err(crate::errors::BadInput("a null sound object").into());
    }
    if RACE_VIEW.players.get() == 1 && RACE_VIEW.view_blocks[0].get() == 0 {
        return Err(crate::errors::BadInput("a null view block").into());
    }
    unsafe { run() };
    DRAW.scratch.set(x87::loaded(CAR.engine_speed.bits()));
    let floor = PHYSICS.idle_engine_speed.bits();
    let held = hold_low(f32::from_bits(DRAW.scratch.get()), f32::from_bits(floor));
    DRAW.scratch.set(held.to_bits());
    DRAW.scratch.set(ext_to_int(extended(g(0x62400C))) as u32);
    let (revs, _) = ease_revs(CAR.engine_sound_revs.get(), DRAW.scratch.get());
    CAR.engine_sound_revs.set(revs);
    unsafe { car_sound_state() }?;
    unsafe { gear_change_sound() }
}

#[cfg(test)]
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
            operands: vec![Operand::Buffer {
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
            plain: |a| run_at(a[0]),
        }];
        sparse(&blocks, 4000, 200);
    }
}
