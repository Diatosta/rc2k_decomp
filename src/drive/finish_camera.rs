//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.

use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::race::owner::race;
use crate::race::view_vars_store::{ViewRecord, ViewVars};

/// View record 6, the finish camera's (`dword_606728` ..).
fn record() -> &'static ViewRecord {
    ViewVars::record(6)
}

/// `dword_606748`, `606730`, `606734` from the eased time, the share and the lines'
/// constants, over the race block at `rc` (the dwords at RVA offsets).
fn lines() {
    let replay = &race().replay;
    record().angles[1].set_bits(st(
        ld(replay.finish.eased.bits()) * ld(replay.finish.unknown_2c[0].bits())
    ));
    let a = ld(replay.finish.eased.bits()) * ld(replay.finish.unknown_2c[4].bits())
        + ld(replay.finish.unknown_2c[2].bits());
    let b = ld(replay.finish.share.bits()) * ld(replay.finish.unknown_64.bits());
    record().offset[1].set_bits(st(a + b));
    record().offset[2].set_bits(st(ld(replay.finish.eased.bits())
        * ld(replay.finish.unknown_2c[3].bits())
        + ld(replay.finish.unknown_2c[1].bits())));
}

/// `dword_606744` from the share and its constant.
fn last_line() {
    let replay = &race().replay;
    record().angles[0].set_bits(st(
        ld(replay.finish.share.bits()) * ld(replay.finish.unknown_60.bits())
    ));
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_camera() {
    lines();
    record().offset[0].set(0);
    last_line();
}

#[cfg(all(test, target_arch = "x86"))]
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
                operands: vec![Operand::RaceBlock {
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
                plain: |_| lines(),
            },
            Block {
                name: "finish_camera last line",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB6C, In::F), (0x24BB78, In::F)],
                    writes: &[(0x206744, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_last(a[0]) },
                plain: |_| last_line(),
            },
        ]
    }

    #[test]
    fn finish_camera_matches_the_assembly() {
        sparse(&blocks(), 3000, 201);
    }
}
