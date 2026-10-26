//! `physics::ease_view_angles` (0x43846D): with the mode's record's change time (+0x3C)
//! not 0: the share done (`physics/camera_view.rs`) times the record's three angle
//! changes (+0x40 .. +0x48) x `flt_5C5D98` (`fistp`) << 16 added to `unk_71A694`,
//! `unk_71A698`, `unk_71A69C`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the last change added (or the time held, kept when
//!      the record has no time); the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::{rd, record, share_done};
use crate::physics::car_store::CAR;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fistp, fld};

/// The Rust version (the exact one, for `physics::update_camera`).
pub fn ease_view_angles() {
    let esi = record();
    if rd(esi + 0x3C) == 0 {
        return;
    }
    share_done(esi);
    for (k, to) in CAR.camera_offset_turns.iter().enumerate() {
        x87_1(
            (esi + 0x40 + 4 * k as u32) as usize,
            crate::race::store::asm_base() as usize,
        );
        to.set(to.get().wrapping_add(DRAW.scratch.get() << 16));
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x22400C] = ([s + 0] * [rc + 0x205260]) * [rc + 0x1C5D98]`
fn x87_1(s: usize, rc: usize) {
    let t0 = fld(s, 0);
    let t1 = fld(rc, 0x205260);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x1C5D98);
    let t4 = t2 * t3;
    fistp(rc, 0x22400C, t4);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fld dword ptr [{s}]", "fmul dword ptr [{rc} + 0x205260]", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "ease_view_angles line 26",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x1C5D98, In::F), (0x205260, In::F)],
                    writes: &[(0x22400C, Out::I)],
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
