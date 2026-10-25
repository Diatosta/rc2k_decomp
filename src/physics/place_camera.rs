//! `physics::place_camera` (0x438E5A): the camera placed. Seven figures from
//! `unk_6077CC` (with `dword_6238A4` at 2) or `unk_6077B0` into `dword_606678`,
//! `60667C`, `6066D4`, `6066D8`, `606730`, `606734` and the last into `dword_60668C`,
//! `6066E8`, `606744`; the offset (`dword_608B30` ..) turned by `unk_71A69C`,
//! `71A694`, `71A698` then `71A68C`, `71A690`, `71A688` (`math::turn_point_xz_ebx`,
//! `_ecx`, `turn_point_xz`, in the original's registers) into the place `unk_71A67C`
//! ..; then the second offset (`dword_608B74` .., / 65536) turned by `g_crashMatrix`
//! (`scene::rotate_point`), x 65536 (`fistp`), plus `dword_6416E8` .., added to it.
//!
//! In:  nothing
//! Out: eax, ebx, ecx the last offset added, edx the last turn's, esi `g_crashMatrix`;
//!      the rest kept; no flags read
//!
//! Callers: `sub_43907F` (seven).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::rd;
use crate::physics::car_store::CAR;
use crate::physics::flow::{a, g, p};
use crate::race::frame_store::FRAME_WORK;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fld, fstp80};
use rc2k_core::physics::camera_place::{SECOND_RECORD_MODE, add_offset};

/// The turns: `math::turn_point_xz_ebx`, `_ecx` and `turn_point_xz`, exact or plain.
type Turns = (
    unsafe fn(i32, i32, i32, u32) -> [u32; 3],
    unsafe fn(i32, i32, i32, u32) -> [u32; 3],
    unsafe fn(i32, i32, i32, u32) -> [u32; 4],
);

/// The steps both versions share; `second` the second offset turned with the car.
unsafe fn place_with(turns: Turns, second: &dyn Fn() -> [i32; 3]) {
    let esi = if RACE_VARS.split.get() == SECOND_RECORD_MODE {
        a(0x6077CC)
    } else {
        a(0x6077B0)
    };
    for (k, to) in [
        0x606678usize,
        0x60667C,
        0x6066D4,
        0x6066D8,
        0x606730,
        0x606734,
    ]
    .iter()
    .enumerate()
    {
        p(*to, rd(esi + 4 * k as u32));
    }
    let last = rd(esi + 0x18);
    for to in [0x60668C, 0x6066E8, 0x606744] {
        p(to, last);
    }
    let (mut eax, mut ebx, mut ecx) = (g(0x608B30), g(0x608B34), g(0x608B38));
    for (ebx_turn, ecx_turn, eax_turn) in [
        (
            &CAR.camera_offset_turns[2],
            &CAR.camera_offset_turns[0],
            &CAR.camera_offset_turns[1],
        ),
        (
            &CAR.camera_roll_turn,
            &CAR.camera_pitch,
            &CAR.camera_heading,
        ),
    ] {
        let i = |v: u32| v as i32;
        let [x, z, _] = unsafe { turns.0(i(ebx), i(ecx), i(eax), ebx_turn.get()) };
        ebx = x;
        eax = z;
        let [x, z, _] = unsafe { turns.1(i(ecx), i(eax), i(ebx), ecx_turn.get()) };
        ecx = x;
        ebx = z;
        let [x, _, z, _] = unsafe { turns.2(i(eax), i(ebx), i(ecx), eax_turn.get()) };
        eax = x;
        ecx = z;
    }
    CAR.camera_position[0].set(eax);
    CAR.camera_position[1].set(ebx);
    CAR.camera_position[2].set(ecx);
    // 0x438F14: the second offset, turned with the car.
    let whole = second();
    DRAW.scratch.set(whole[2] as u32);
    let car = [
        FRAME_WORK.camera_car[0].get(),
        FRAME_WORK.camera_car[1].get(),
        FRAME_WORK.camera_car[2].get(),
    ];
    let place = [
        CAR.camera_position[0].get(),
        CAR.camera_position[1].get(),
        CAR.camera_position[2].get(),
    ];
    let (_, place) = add_offset(place, whole, car);
    CAR.camera_position[0].set(place[0]);
    CAR.camera_position[1].set(place[1]);
    CAR.camera_position[2].set(place[2]);
}

/// The Rust version (the exact one, for `physics::camera_avoid_ground`). The registers it
/// leaves are read by none of its callers.
pub unsafe fn place_camera() {
    use crate::math::{turn_point_xz, turn_point_xz_ebx, turn_point_xz_ecx};
    let second = || {
        let mut parts = [[0u8; 10]; 3];
        for (k, x) in [0x608B74usize, 0x608B78, 0x608B7C].iter().enumerate() {
            x87_1(
                a(*x) as usize,
                crate::race::store::asm_base() as usize,
                (parts[k].as_mut_ptr()) as usize,
            );
        }
        let turned = unsafe {
            crate::scene::rotate_point::rotate_point(CAR.crash_matrix[0].addr() as u32, parts)
        };
        let k = NUMBERS.k65536.bits();
        turned.map(|e| crate::math::x87_run::ext_to_int(crate::math::x87_run::ext_times(e, k)))
    };
    unsafe {
        place_with(
            (
                turn_point_xz_ebx::turn_point_xz_ebx,
                turn_point_xz_ecx::turn_point_xz_ecx,
                turn_point_xz::turn_point_xz,
            ),
            &second,
        )
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] / [rc + 0x1C5D98]`
fn x87_1(v: usize, rc: usize, o: usize) {
    unsafe {
        let v = v as *const u8;
        let rc = rc as *const u8;
        let o = o as *mut u8;
        let t0 = fild(v, 0);
        let t1 = fld(rc, 0x1C5D98);
        let t2 = t0 / t1;
        fstp80(o, 0, t2);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(v, rc, o) "fild dword ptr [{v}]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp tbyte ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "place_camera line 91",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::T)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
            plain: |a| x87_1(a[0], a[1], a[2]),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
