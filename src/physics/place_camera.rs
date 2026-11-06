//! `physics::place_camera` (0x438E5A): the camera placed. Seven figures from
//! `unk_6077CC` (with `dword_6238A4` at 2) or `unk_6077B0` into `dword_606678`,
//! `60667C`, `6066D4`, `6066D8`, `606730`, `606734` and the last into `dword_60668C`,
//! `6066E8`, `606744`; the offset (`dword_608B30` ..) turned by `unk_71A69C`,
//! `71A694`, `71A698` then `71A68C`, `71A690`, `71A688` (`math::turn_point_xz_ebx`,
//! `_ecx`, `turn_point_xz`) into the place `unk_71A67C`
//! ..; then the second offset (`dword_608B74` .., / 65536) turned by `g_crashMatrix`
//! (`scene::rotate_point`), x 65536 (`fistp`), plus `dword_6416E8` .., added to it.
//!
//! Callers: `sub_43907F` (seven).

use crate::math::x87_float::{f80, int, ld};
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::car_store::CAR;
use crate::race::frame_store::FRAME_WORK;
use crate::race::lights_store::LIGHTS_DATA;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_vars_store::ViewVars;
use crate::scene::draw_store::DRAW;
use crate::session::Cell32;
use rc2k_core::physics::camera_place::{SECOND_RECORD_MODE, add_offset};

/// The turns: `math::turn_point_xz_ebx`, `_ecx` and `turn_point_xz`, exact or plain.
type Turns = (
    fn(i32, i32, i32, u32) -> [u32; 3],
    fn(i32, i32, i32, u32) -> [u32; 3],
    fn(i32, i32, i32, u32) -> [u32; 4],
);

/// The steps both versions share; `second` the second offset turned with the car.
fn place_with(turns: Turns, second: &dyn Fn() -> [i32; 3]) {
    let figures = if RACE_VARS.split.get() == SECOND_RECORD_MODE {
        &LIGHTS_DATA.place_second
    } else {
        &LIGHTS_DATA.place_usual
    };
    // Records 4, 5 and 6: the offset's y and z, then the first angle.
    let [r4, r5, r6] = [4, 5, 6].map(ViewVars::record);
    let places = [
        &r4.offset[1],
        &r4.offset[2],
        &r5.offset[1],
        &r5.offset[2],
        &r6.offset[1],
        &r6.offset[2],
    ];
    for (k, to) in places.iter().enumerate() {
        to.set(figures[k].get());
    }
    let last = figures[6].get();
    for to in [&r4.angles[0], &r5.angles[0], &r6.angles[0]] {
        to.set(last);
    }
    // The offset (x, y, z) turned about each axis in turn: the `turn_point_xz*` functions
    // take the three coordinates in the order of the original's registers, which name
    // them here (`eax` x, `ebx` y, `ecx` z).
    let (mut x, mut y, mut z) = (
        CAMERA_WORK.camera_offset_x.get(),
        CAMERA_WORK.camera_offset_y.get(),
        CAMERA_WORK.camera_offset_z.get(),
    );
    for (y_turn, z_turn, x_turn) in [
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
        let [new_y, new_x, _] = { turns.0(i(y), i(z), i(x), y_turn.get()) };
        y = new_y;
        x = new_x;
        let [new_z, new_y, _] = { turns.1(i(z), i(x), i(y), z_turn.get()) };
        z = new_z;
        y = new_y;
        let [new_x, _, new_z, _] = { turns.2(i(x), i(y), i(z), x_turn.get()) };
        x = new_x;
        z = new_z;
    }
    CAR.camera_position[0].set(x);
    CAR.camera_position[1].set(y);
    CAR.camera_position[2].set(z);
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

/// The Rust version (the exact one, for `physics::camera_avoid_ground`).
pub fn place_camera() {
    use crate::math::{turn_point_xz, turn_point_xz_ebx, turn_point_xz_ecx};
    let second = || {
        let turns = [
            &CAMERA_WORK.view_turn_x,
            &CAMERA_WORK.view_turn_y,
            &CAMERA_WORK.view_turn_z,
        ];
        let parts = turns.map(view_turn_part);
        let turned = { crate::scene::rotate_point::rotate_point(&CAR.crash_matrix, parts) };
        let k = NUMBERS.k65536.bits();
        turned.map(|e| crate::math::x87_run::ext_to_int(crate::math::x87_run::ext_times(e, k)))
    };
    place_with(
        (
            turn_point_xz_ebx::turn_point_xz_ebx,
            turn_point_xz_ecx::turn_point_xz_ecx,
            turn_point_xz::turn_point_xz,
        ),
        &second,
    );
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] / [rc + 0x1C5D98]`; the value is `o`'s ten bytes.
fn view_turn_part(v: &impl Cell32) -> [u8; 10] {
    let t0 = int(v.bits() as i32);
    let t1 = ld(NUMBERS.k65536.bits());
    let t2 = t0 / t1;
    f80(t2)
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::{Base, HostCell};

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
                Operand::RaceBlock {
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
            plain: |a| a[2].set_tbyte(0, view_turn_part(&HostCell(a[0]))),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
