//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.

use crate::physics::flow::p;
use crate::scene::x87_mem::{fld, fstp};

/// `dword_606748`, `606730`, `606734` from the eased time, the share and the lines'
/// constants, over the race block at `rc` (the dwords at RVA offsets).
fn lines(rc: usize) {
    let rc = rc as *mut u8;
    unsafe {
        fstp(rc, 0x206748, fld(rc, 0x24BB68) * fld(rc, 0x24BB4C));
        let a = fld(rc, 0x24BB68) * fld(rc, 0x24BB5C) + fld(rc, 0x24BB54);
        let b = fld(rc, 0x24BB6C) * fld(rc, 0x24BB7C);
        fstp(rc, 0x206730, a + b);
        fstp(
            rc,
            0x206734,
            fld(rc, 0x24BB68) * fld(rc, 0x24BB58) + fld(rc, 0x24BB50),
        );
    }
}

/// `dword_606744` from the share and its constant.
fn last_line(rc: usize) {
    let rc = rc as *mut u8;
    unsafe { fstp(rc, 0x206744, fld(rc, 0x24BB6C) * fld(rc, 0x24BB78)) }
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_camera() {
    lines(crate::race::store::asm_base() as usize);
    p(0x60672C, 0);
    last_line(crate::race::store::asm_base() as usize);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_lines(rc)
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB4C]",
        "fstp dword ptr [{rc} + 0x206748]",
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB5C]",
        "fadd dword ptr [{rc} + 0x24BB54]",
        "fld dword ptr [{rc} + 0x24BB6C]",
        "fmul dword ptr [{rc} + 0x24BB7C]",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fstp dword ptr [{rc} + 0x206730]",
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB58]",
        "fadd dword ptr [{rc} + 0x24BB50]",
        "fstp dword ptr [{rc} + 0x206734]");
    reference!(ref_last(rc)
        "fld dword ptr [{rc} + 0x24BB6C]",
        "fmul dword ptr [{rc} + 0x24BB78]",
        "fstp dword ptr [{rc} + 0x206744]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "finish_camera lines",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x24BB68, In::F),
                        (0x24BB4C, In::F),
                        (0x24BB5C, In::F),
                        (0x24BB54, In::F),
                        (0x24BB6C, In::F),
                        (0x24BB7C, In::F),
                        (0x24BB58, In::F),
                        (0x24BB50, In::F),
                    ],
                    writes: &[(0x206748, Out::F), (0x206730, Out::F), (0x206734, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_lines(a[0]) },
                plain: |a| lines(a[0]),
            },
            Block {
                name: "finish_camera last line",
                operands: vec![Operand::Buffer {
                    reads: &[(0x24BB6C, In::F), (0x24BB78, In::F)],
                    writes: &[(0x206744, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_last(a[0]) },
                plain: |a| last_line(a[0]),
            },
        ]
    }

    #[test]
    fn finish_camera_matches_the_assembly() {
        sparse(&blocks(), 3000, 201);
    }
}
