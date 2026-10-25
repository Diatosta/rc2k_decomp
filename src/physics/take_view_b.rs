//! `physics::take_view_b` (0x438024): `physics::take_view_a` for the second view: +4 ..
//! +0xC plus `dword_637638` .. into `flt_608BB0` ..; +0x10 .. +0x24 into `flt_608B8C`
//! .., `flt_608BD4` ..; +0x28 .. +0x38 into `dword_608C0C`, `608C10`, `608C14`,
//! `608C08`, `608C18`; the heading into `dword_608B68`; the next two halved (`sar`)
//! unless the mode before (`unk_71A660`) is 1, 2 or 3, into `dword_608B70`,
//! `dword_608B6C`.
//!
//! In:  esi the view's record
//! Out: eax, ebx the two; the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::rd;
use crate::physics::flow::{g, p};
use crate::scene::x87_mem::{fld, fstp};

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The eax and ebx it leaves are read by none of its callers.
pub unsafe fn take_view_b(s: u32) {
    x87_1(s as usize, crate::race::store::asm_base() as usize);
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B68, g(0x71A998));
    let (mut eax, mut ebx) = (g(0x71A99C), g(0x71A9A0));
    if !matches!(g(0x71A660), 1..=3) {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    p(0x608B70, eax);
    p(0x608B6C, ebx);
}

/// Where the record's +0x10 .. +0x38 go.
const COPIES: [usize; 11] = [
    0x608B8C, 0x608B90, 0x608B94, 0x608BD4, 0x608BD8, 0x608BDC, 0x608C0C, 0x608C10, 0x608C14,
    0x608C08, 0x608C18,
];

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x208BB0] = [s + 4] + [rc + 0x237638]`
/// `[rc + 0x208BB4] = [s + 8] + [rc + 0x23763C]`
/// `[rc + 0x208BB8] = [s + 0xC] + [rc + 0x237640]`
fn x87_1(s: usize, rc: usize) {
    unsafe {
        let s = s as *const u8;
        let rc = rc as *mut u8;
        let t0 = fld(s, 4);
        let t1 = fld(rc, 0x237638);
        let t2 = t0 + t1;
        fstp(rc, 0x208BB0, t2);
        let t3 = fld(s, 8);
        let t4 = fld(rc, 0x23763C);
        let t5 = t3 + t4;
        fstp(rc, 0x208BB4, t5);
        let t6 = fld(s, 0xC);
        let t7 = fld(rc, 0x237640);
        let t8 = t6 + t7;
        fstp(rc, 0x208BB8, t8);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fld dword ptr [{s} + 0x4]", "fadd dword ptr [{rc} + 0x237638]", "fstp dword ptr [{rc} + 0x208BB0]", "fld dword ptr [{s} + 0x8]", "fadd dword ptr [{rc} + 0x23763C]", "fstp dword ptr [{rc} + 0x208BB4]", "fld dword ptr [{s} + 0xC]", "fadd dword ptr [{rc} + 0x237640]", "fstp dword ptr [{rc} + 0x208BB8]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "take_view_b line 22",
            operands: vec![
                Operand::Buffer {
                    reads: &[(4, In::F), (8, In::F), (0xC, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x237638, In::F), (0x23763C, In::F), (0x237640, In::F)],
                    writes: &[(0x208BB0, Out::F), (0x208BB4, Out::F), (0x208BB8, Out::F)],
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
