//! `physics::ease_view_turn` (0x438B4F): the camera's three turns (`unk_71A688`,
//! `71A690`, `71A68C`) each moved toward its target (`dword_608B50`, `608B58`,
//! `608B54`) by its share (`dword_608BE8`, `608BE4`, `608BEC`; all 1 first when
//! `dword_6EF7CC` is set): the difference `fild`ed, times the share, `fistp`ed, added.
//!
//! In:  nothing
//! Out: eax the last step; the rest kept; no flags read
//!
//! Caller: `sub_43907F`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::flow::{g, p};
use crate::scene::x87_mem::{fild, fistp, fld};

/// The Rust version (the exact one, for `physics::camera_avoid_ground`).
pub unsafe fn ease_view_turn() {
    if g(0x6EF7CC) != 0 {
        for x in [0x608BE4, 0x608BE8, 0x608BEC] {
            p(x, 0x3F80_0000);
        }
    }
    for (target, turn, share) in [
        (0x608B50, 0x71A688, 0x608BE8),
        (0x608B58, 0x71A690, 0x608BE4),
        (0x608B54, 0x71A68C, 0x608BEC),
    ] {
        p(0x62400C, g(target).wrapping_sub(g(turn)));
        x87_1(
            crate::race::store::asm_base() as usize,
            crate::physics::flow::a(share) as usize,
        );
        p(turn, g(turn).wrapping_add(g(0x62400C)));
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = int [rc + 0x22400C] * [s + 0]`
fn x87_1(rc: usize, s: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let s = s as *const u8;
        let t0 = fild(rc, 0x22400C);
        let t1 = fld(s, 0);
        let t2 = t0 * t1;
        fistp(rc, 0x22400C, t2);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc, s) "fild dword ptr [{rc} + 0x22400C]", "fmul dword ptr [{s}]", "fistp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "ease_view_turn line 29",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x22400C, In::I)],
                    writes: &[(0x22400C, Out::I)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
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
