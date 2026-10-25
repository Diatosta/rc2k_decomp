//! `drive::steer_inputs` (0x439DB1): the work car's steering from the input word
//! `dword_613890`. Key steering (the word 0, or a car not driven by device, `unk_71A234`
//! not 2, with the keys' bit 0x1000 of `unk_71A440`): the step 0xA00000 x `flt_614C0C`
//! (`fistp`) moves the two steering values `unk_71A620`, `unk_71A61C` toward the side the
//! word says, held at ± 0x20000000, or, with the word 0, back toward 0 at twice the step
//! (stopping at 0; each by the sign of the result, as the original). Device steering:
//! the word x 0x20000000 >> 8 (`imul`, `shrd`) x `flt_614C08`, held ± 0x20000000, into
//! both (`unk_71A61C` 0 first). Then the trim (0x20000000 x `unk_71A3E0`, negated with
//! `unk_719FE8` bit 0x10) into `unk_71A1E0`; `unk_71A624` = `unk_71A61C` + it,
//! `unk_71A628` = `unk_71A620` + 4 x it; both 0 unless `unk_71A308` is below
//! `flt_5C5D14` (or unordered).
//!
//! In:  nothing
//! Out: eax `unk_71A628` before the last test, ebx `unk_71A440` or 0x20000000 (device),
//!      edx the product's high dword (device); the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`.

use crate::physics::car_store::CAR;
use crate::physics::flow::c0;
use crate::physics::tables_store::PHYSICS;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fistp, fld};
use crate::session::Dword;

/// `fild dword [rc + from]; fmul dword [rc + by]; fistp dword [rc + to]` over the race
/// block at `rc`.
fn scale(rc: usize, from: usize, by: usize, to: usize) {
    let rc = rc as *mut u8;
    unsafe { fistp(rc, to, fild(rc, from) * fld(rc, by)) }
}

fn base() -> usize {
    crate::race::store::asm_base() as usize
}
const LEFT: u32 = 0xE000_0000;
const RIGHT: u32 = 0x2000_0000;

fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The key step: 0xA00000 x `flt_614C0C`, through `dword_62400C`.
fn key_step() -> u32 {
    DRAW.scratch.set(0xA0_0000);
    scale(base(), 0x22400C, 0x214C0C, 0x22400C);
    DRAW.scratch.get()
}

/// One steering value moved by the key step (0x439DFA .. / 0x439E95 ..).
fn steer(cell: &Dword, step: u32) {
    let word = PHYSICS.input_steer.get();
    let v = cell.get();
    if word != 0 {
        if sign(word) {
            let n = v.wrapping_sub(step);
            cell.set(if sign(n.wrapping_sub(LEFT)) { LEFT } else { n });
        } else {
            let n = v.wrapping_add(step);
            cell.set(if sign(n.wrapping_sub(RIGHT)) {
                n
            } else {
                RIGHT
            });
        }
        return;
    }
    let step = step.wrapping_add(step);
    if v == 0 {
        return;
    }
    if sign(v) {
        let n = v.wrapping_add(step);
        cell.set(if sign(n) { n } else { 0 });
    } else {
        let n = v.wrapping_sub(step);
        cell.set(if sign(n) { 0 } else { n });
    }
}

/// The Rust version (the exact one, for `drive::simple_car_step`, `drive::replay_step`
/// and `race::step_replay_cars`).
pub unsafe fn steer_inputs() {
    let mut keys = PHYSICS.input_steer.get() == 0;
    if !keys && CAR.driver_kind.get() != 2 {
        keys = CAR.input_keys.get() & 0x1000 != 0;
    }
    if keys {
        steer(&CAR.steering_raw, key_step());
        steer(&CAR.steering, key_step());
    } else {
        CAR.steering.set(0);
        let product = i64::from(PHYSICS.input_steer.get() as i32) * i64::from(RIGHT as i32);
        DRAW.scratch.set((product >> 8) as u32);
        scale(base(), 0x22400C, 0x214C08, 0x22400C);
        if sign(DRAW.scratch.get().wrapping_sub(LEFT)) {
            DRAW.scratch.set(LEFT);
        }
        if !sign(DRAW.scratch.get().wrapping_sub(RIGHT)) {
            DRAW.scratch.set(RIGHT);
        }
        CAR.steering_raw.set(DRAW.scratch.get());
        CAR.steering.set(DRAW.scratch.get());
    }
    // 0x439F7D: the trim.
    DRAW.scratch.set(RIGHT);
    scale(base(), 0x22400C, 0x31A3E0, 0x31A1E0);
    if CAR.car_flags.get() & 0x10 != 0 {
        CAR.steer_trim.set(
            crate::physics::car_store::CAR
                .steer_trim
                .get()
                .wrapping_neg(),
        );
    }
    let trim = CAR.steer_trim.get();
    CAR.steer_angle.set(
        crate::physics::car_store::CAR
            .steering
            .get()
            .wrapping_add(trim),
    );
    CAR.steer_angle_visual.set(
        crate::physics::car_store::CAR
            .steering_raw
            .get()
            .wrapping_add(trim.wrapping_mul(4)),
    );
    if !c0(CAR.steering_damage.bits(), NUMBERS.hundred.bits()) {
        CAR.steer_angle.set(0);
        CAR.steer_angle_visual.set(0);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_key(rc)
        "fild dword ptr [{rc} + 0x22400C]",
        "fmul dword ptr [{rc} + 0x214C0C]",
        "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_device(rc)
        "fild dword ptr [{rc} + 0x22400C]",
        "fmul dword ptr [{rc} + 0x214C08]",
        "fistp dword ptr [{rc} + 0x22400C]");
    reference!(ref_trim(rc)
        "fild dword ptr [{rc} + 0x22400C]",
        "fmul dword ptr [{rc} + 0x31A3E0]",
        "fistp dword ptr [{rc} + 0x31A1E0]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "steer_inputs key step",
                operands: vec![Operand::Buffer {
                    reads: &[(0x22400C, In::I), (0x214C0C, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_key(a[0]) },
                plain: |a| scale(a[0], 0x22400C, 0x214C0C, 0x22400C),
            },
            Block {
                name: "steer_inputs device",
                operands: vec![Operand::Buffer {
                    reads: &[(0x22400C, In::I), (0x214C08, In::F)],
                    writes: &[(0x22400C, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_device(a[0]) },
                plain: |a| scale(a[0], 0x22400C, 0x214C08, 0x22400C),
            },
            Block {
                name: "steer_inputs trim",
                operands: vec![Operand::Buffer {
                    reads: &[(0x22400C, In::I), (0x31A3E0, In::F)],
                    writes: &[(0x31A1E0, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_trim(a[0]) },
                plain: |a| scale(a[0], 0x22400C, 0x31A3E0, 0x31A1E0),
            },
        ]
    }

    #[test]
    fn steer_inputs_matches_the_assembly() {
        sparse(&blocks(), 4000, 203);
    }
}
