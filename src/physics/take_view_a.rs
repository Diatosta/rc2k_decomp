//! `physics::take_view_a` (0x437F53): a view's figures (the record at esi) into the
//! first view: +4 .. +0xC plus `dword_637638` .. into `flt_608B98` ..; +0x10 .. +0x24
//! into `flt_608B80` .., `flt_608BC8` ..; +0x28 .. +0x38 into `dword_608BF8`,
//! `608BFC`, `608C00`, `608BF4`, `608C04`; the heading (`unk_71A998`) into
//! `dword_608B5C`; the next two (`unk_71A99C`, `unk_71A9A0`), halved (`sar`) when the
//! mode's record has +0x4C set, into `dword_608B64`, `dword_608B60`.
//!
//! In:  esi the view's record
//! Out: eax, ebx the two, esi the mode's record; the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::camera_view::{rd, record};
use crate::physics::car_store::CAR;
use crate::scene::x87_mem::{fld, fstp};

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The registers it leaves are read by none of its callers.
pub fn take_view_a(s: u32) {
    x87_1(s as usize, crate::race::store::asm_base() as usize);
    for (k, put) in copies().iter().enumerate() {
        put(rd(s + 0x10 + 4 * k as u32));
    }
    CAMERA_WORK.view_a_heading.set(CAR.car_heading.get());
    let (mut eax, mut ebx) = (CAR.car_pitch.get(), CAR.car_roll.get());
    if rd(record() + 0x4C) != 0 {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    CAMERA_WORK.view_a_pitch.set(eax);
    CAMERA_WORK.view_a_roll.set(ebx);
}

/// Where the record's +0x10 .. +0x38 go (the statics deref at run time, so a function).
fn copies() -> [fn(u32); 11] {
    [
        |v| CAMERA_WORK.view_a_turn[0].set_bits(v),
        |v| CAMERA_WORK.view_a_turn[1].set_bits(v),
        |v| CAMERA_WORK.view_a_turn[2].set_bits(v),
        |v| CAMERA_WORK.view_a_point[0].set_bits(v),
        |v| CAMERA_WORK.view_a_point[1].set_bits(v),
        |v| CAMERA_WORK.view_a_point[2].set_bits(v),
        |v| CAMERA_WORK.view_a_shares[0].set_bits(v),
        |v| CAMERA_WORK.view_a_shares[1].set_bits(v),
        |v| CAMERA_WORK.view_a_shares[2].set_bits(v),
        |v| CAMERA_WORK.view_a_aux.set(v),
        |v| CAMERA_WORK.view_a_aux_2.set(v),
    ]
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x208B98] = [s + 4] + [rc + 0x237638]`
/// `[rc + 0x208B9C] = [s + 8] + [rc + 0x23763C]`
/// `[rc + 0x208BA0] = [s + 0xC] + [rc + 0x237640]`
fn x87_1(s: usize, rc: usize) {
    let t0 = fld(s, 4);
    let t1 = fld(rc, 0x237638);
    let t2 = t0 + t1;
    fstp(rc, 0x208B98, t2);
    let t3 = fld(s, 8);
    let t4 = fld(rc, 0x23763C);
    let t5 = t3 + t4;
    fstp(rc, 0x208B9C, t5);
    let t6 = fld(s, 0xC);
    let t7 = fld(rc, 0x237640);
    let t8 = t6 + t7;
    fstp(rc, 0x208BA0, t8);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fld dword ptr [{s} + 0x4]", "fadd dword ptr [{rc} + 0x237638]", "fstp dword ptr [{rc} + 0x208B98]", "fld dword ptr [{s} + 0x8]", "fadd dword ptr [{rc} + 0x23763C]", "fstp dword ptr [{rc} + 0x208B9C]", "fld dword ptr [{s} + 0xC]", "fadd dword ptr [{rc} + 0x237640]", "fstp dword ptr [{rc} + 0x208BA0]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "take_view_a line 22",
            operands: vec![
                Operand::Buffer {
                    reads: &[(4, In::F), (8, In::F), (0xC, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x237638, In::F), (0x23763C, In::F), (0x237640, In::F)],
                    writes: &[(0x208B98, Out::F), (0x208B9C, Out::F), (0x208BA0, Out::F)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| x87_1(a[0], a[1]),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
