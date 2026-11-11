//! `physics::place_camera` (0x438E5A): the camera placed. Seven figures from
//! `place_second` (with `split` at 2) or `place_usual` into `ViewAndRecords.records[4].offset[1]`,
//! `60667C`, `6066D4`, `6066D8`, `606730`, `606734` and the last into `ViewAndRecords.records[4].angles[0]`,
//! `6066E8`, `606744`; the offset (`camera_offset_x` ..) turned by `Car.camera_offset_turns[2]`,
//! `71A694`, `71A698` then `71A68C`, `71A690`, `71A688` (`math::rotate_point_about_y_ebx`,
//! `_ecx`, `rotate_point_about_y`) into the place `camera_position`
//! ..; then the second offset (`view_turn_x` .., / 65536) turned by `g_crashMatrix`
//! (`scene::rotate_point`), x 65536 (`fistp`), plus `camera_car` .., added to it.
//!
//! Callers: `physics::camera_avoid_ground` (seven).

use crate::math::x87_float::{f80, int, ld};
use crate::physics::camera_store::CameraWork;
use crate::race::frame_store::FrameWork;
use crate::race::lights_store::LightsData;
use crate::race::numbers_store::Numbers;
use crate::race::owner::race;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::ViewVars;
use crate::session::Cell32;
use rc2k_core::physics::camera_place::{SECOND_RECORD_MODE, add_offset};

/// The turns: `math::rotate_point_about_y_ebx`, `_ecx` and `rotate_point_about_y`, exact or plain.
type Turns = (
    fn(i32, i32, i32, u32) -> [u32; 3],
    fn(i32, i32, i32, u32) -> [u32; 3],
    fn(i32, i32, i32, u32) -> [u32; 4],
);

/// The steps both versions share; `second` the second offset turned with the car.
fn place_with(
    camera: &CameraWork,
    lights_data: &LightsData,
    frame_work: &FrameWork,
    shared: &RaceShared,
    turns: Turns,
    second: &dyn Fn() -> [i32; 3],
    scene: &crate::scene::owner::Scene,
) {
    let figures = if shared.vars.split.get() == SECOND_RECORD_MODE {
        &lights_data.place_second
    } else {
        &lights_data.place_usual
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
    // The offset (x, y, z) turned about each axis in turn: the `rotate_point_about_y*` functions
    // take the three coordinates in the order x, y, z.
    let (mut x, mut y, mut z) = (
        camera.camera_offset_x.get(),
        camera.camera_offset_y.get(),
        camera.camera_offset_z.get(),
    );
    for (y_turn, z_turn, x_turn) in [
        (
            &shared.car.camera_offset_turns[2],
            &shared.car.camera_offset_turns[0],
            &shared.car.camera_offset_turns[1],
        ),
        (
            &shared.car.camera_roll_turn,
            &shared.car.camera_pitch,
            &shared.car.camera_heading,
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
    shared.car.camera_position[0].set(x);
    shared.car.camera_position[1].set(y);
    shared.car.camera_position[2].set(z);
    // 0x438F14: the second offset, turned with the car.
    let whole = second();
    scene.drawing.scene.scratch.set(whole[2] as u32);
    let car = [
        frame_work.camera_car[0].get(),
        frame_work.camera_car[1].get(),
        frame_work.camera_car[2].get(),
    ];
    let place = [
        shared.car.camera_position[0].get(),
        shared.car.camera_position[1].get(),
        shared.car.camera_position[2].get(),
    ];
    let (_, place) = add_offset(place, whole, car);
    shared.car.camera_position[0].set(place[0]);
    shared.car.camera_position[1].set(place[1]);
    shared.car.camera_position[2].set(place[2]);
}

/// The exact version, for `physics::camera_avoid_ground`.
pub fn place_camera(
    camera: &CameraWork,
    lights_data: &LightsData,
    numbers: &Numbers,
    frame_work: &FrameWork,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) {
    use crate::math::{rotate_point_about_y, rotate_point_about_y_ebx, rotate_point_about_y_ecx};
    let second = || {
        let turns = [
            &camera.view_turn_x,
            &camera.view_turn_y,
            &camera.view_turn_z,
        ];
        let parts = turns.map(view_turn_part);
        let turned = { crate::scene::rotate_point::rotate_point(&shared.car.crash_matrix, parts) };
        let k = numbers.k65536.bits();
        turned.map(|e| crate::math::x87::ext_to_int(crate::math::x87::ext_times(e, k)))
    };
    place_with(
        camera,
        lights_data,
        frame_work,
        shared,
        (
            rotate_point_about_y_ebx::rotate_point_about_y_ebx,
            rotate_point_about_y_ecx::rotate_point_about_y_ecx,
            rotate_point_about_y::rotate_point_about_y,
        ),
        &second,
        scene,
    );
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] / [k65536]`; the value is `o`'s ten bytes.
fn view_turn_part(v: &impl Cell32) -> [u8; 10] {
    let t0 = int(v.bits() as i32);
    let t1 = ld(race().course.numbers.k65536.bits());
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
