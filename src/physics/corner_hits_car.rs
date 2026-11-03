//! `physics::corner_hits_car` (0x45F703): whether a corner of another car (`dword_6D3BD4`
//! .., floats) lies inside the work car. In `pusha` / `popa`: the corner x 65536
//! (`flt_5C5D98`, unrounded) turned by `flt_6D3C40` (`scene::rotate_point`) and / 65536
//! into `flt_6D3BC8` ... Inside when each part's size is below the box's (`flt_6D3BBC`
//! ..; unordered counts as below, as `fcomp` / `jnb`): `dword_6D3BB4` = 1,
//! `physics::restore_saved_car` (in `pusha` / `popa`), `physics::bump_cars`.
//!
//! In:  edi the other car's block (passed to `bump_cars`)
//! Out: outside, every register kept; inside, as `bump_cars` leaves them; no flags read
//!      (the caller goes on)
//!
//! Caller: `physics::car_collisions` (eight corners a car). Refuses what
//! `restore_saved_car` and `bump_cars` refuse (what was written put back).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_float::{from_f80, ld, st};
use crate::math::x87_run::{abs_compare, ext_times};
use crate::physics::contact_store::CONTACT_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::scene::matrix3::extended;
use crate::scene::rotate_point::rotate_point;
use crate::session::Cell32;

/// Whether the corner is inside (`flt_6D3BC8` .. written either way).
pub fn corner_inside() -> bool {
    let k = NUMBERS.k65536.bits();
    let corner = CONTACT_WORK
        .corner_in
        .each_ref()
        .map(|x| ext_times(extended(x.bits()), k));
    let turned = rotate_point(&CONTACT_WORK.corner_matrix, corner);
    for (n, e) in turned.iter().enumerate() {
        x87_1(e, &CONTACT_WORK.corner_turned[n]);
    }
    (0..3).all(|n| {
        abs_compare(
            CONTACT_WORK.corner_turned[n].bits(),
            CONTACT_WORK.box_size[n].bits(),
        ) & 0x100
            != 0
    })
}

/// The Rust version: `block` the other car's block (edi). The registers it leaves are
/// read by none of its callers.
pub fn corner_hits_car(block: u32) -> Result<(), Refused> {
    if !corner_inside() {
        return Ok(());
    }
    CONTACT_WORK.corner_inside.set(1);
    { crate::physics::restore_saved_car::restore_saved_car() }?;
    { crate::physics::bump_cars::bump_cars(block) }?;
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0] = ext [e + 0] / [rc + 0x1C5D98]`
fn x87_1(e: &[u8; 10], o: &impl Cell32) {
    let t0 = from_f80(*e);
    let t1 = ld(NUMBERS.k65536.bits());
    let t2 = t0 / t1;
    o.set_bits(st(t2));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::{HostCell, view};

    // The original's instructions, one function per run.
    reference!(ref_0(e, rc, o) "fld tbyte ptr [{e}]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "corner_hits_car line 30",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::T)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
            plain: |a| x87_1(view(a[0], 0), &HostCell(a[2])),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
