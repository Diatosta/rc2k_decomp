//! Places the camera. Seven figures from `place_second` (with `split` at 2) or `place_usual` go
//! into the mode record `ViewAndRecords.records[4]` and the view's other fields; the offset
//! (`camera_offset_x` ..) is turned by the camera's turns
//! (`math::rotate_point_about_y_ebx`, `rotate_point_about_y_ecx`, `rotate_point_about_y`)
//! into the place `camera_position`; then the second offset (`view_turn_x` .., / 65536) is
//! turned by the crash matrix (`scene::rotate_point`), times 65536 and
//! rounded, plus `camera_car`, and added to it.

use crate::math::engine_tables::EngineTables;
use crate::math::x87_float::{f80, int, ld};
use crate::physics::camera_store::CameraWork;
use crate::race::frame_store::FrameWork;
use crate::race::groups_store::ViewAndRecords;
use crate::race::lights_store::LightsData;
use crate::race::numbers_store::Numbers;
use crate::race::owner_shared::RaceShared;
use crate::race::view_vars_store::ViewVars;
use rc2k_core::physics::camera_place::{SECOND_RECORD_MODE, add_offset};

/// The turns: `math::rotate_point_about_y_ebx`, `_ecx` and `rotate_point_about_y`, exact or plain.
type Turns = (
    fn(&EngineTables, i32, i32, i32, u32) -> [u32; 3],
    fn(&EngineTables, i32, i32, i32, u32) -> [u32; 3],
    fn(&EngineTables, i32, i32, i32, u32) -> [u32; 4],
);

/// The steps both versions share; `second` the second offset turned with the car.
#[allow(
    clippy::too_many_arguments,
    reason = "the camera's variables, tables and records the placing reads, each its own owner part"
)]
fn place_with(
    camera: &CameraWork,
    lights_data: &LightsData,
    view_records: &ViewAndRecords,
    frame_work: &FrameWork,
    shared: &RaceShared,
    turns: Turns,
    second: &dyn Fn() -> [i32; 3],
) {
    let figures = if shared.vars.get().split.get() == SECOND_RECORD_MODE {
        &lights_data.place_second
    } else {
        &lights_data.place_usual
    };
    // Records 4, 5 and 6: the offset's y and z, then the first angle.
    let [r4, r5, r6] = [4, 5, 6].map(|n| ViewVars::record(view_records, n));
    let places = [
        &r4.offset[1],
        &r4.offset[2],
        &r5.offset[1],
        &r5.offset[2],
        &r6.offset[1],
        &r6.offset[2],
    ];
    for (k, to) in places.iter().enumerate() {
        to.set(figures[k]);
    }
    let last = figures[6];
    for to in [&r4.angles[0], &r5.angles[0], &r6.angles[0]] {
        to.set(last);
    }
    // The offset (x, y, z) turned about each axis in turn: the `rotate_point_about_y*` functions
    // take the three coordinates in the order x, y, z.
    let (mut x, mut y, mut z) = (
        camera.camera_offset_x,
        camera.camera_offset_y,
        camera.camera_offset_z,
    );
    for (y_turn, z_turn, x_turn) in [
        (
            &shared.car().camera_offset_turns[2],
            &shared.car().camera_offset_turns[0],
            &shared.car().camera_offset_turns[1],
        ),
        (
            &shared.car().camera_roll_turn,
            &shared.car().camera_pitch,
            &shared.car().camera_heading,
        ),
    ] {
        let i = |v: u32| v as i32;
        let [new_y, new_x, _] = {
            turns.0(
                &shared.buffers.reads.get().engine_tables,
                i(y),
                i(z),
                i(x),
                y_turn.get(),
            )
        };
        y = new_y;
        x = new_x;
        let [new_z, new_y, _] = {
            turns.1(
                &shared.buffers.reads.get().engine_tables,
                i(z),
                i(x),
                i(y),
                z_turn.get(),
            )
        };
        z = new_z;
        y = new_y;
        let [new_x, _, new_z, _] = {
            turns.2(
                &shared.buffers.reads.get().engine_tables,
                i(x),
                i(y),
                i(z),
                x_turn.get(),
            )
        };
        x = new_x;
        z = new_z;
    }
    shared.car().camera_position[0].set(x);
    shared.car().camera_position[1].set(y);
    shared.car().camera_position[2].set(z);
    // The second offset, turned with the car.
    let whole = second();
    let car = [
        frame_work.camera_car[0].get(),
        frame_work.camera_car[1].get(),
        frame_work.camera_car[2].get(),
    ];
    let place = [
        shared.car().camera_position[0].get(),
        shared.car().camera_position[1].get(),
        shared.car().camera_position[2].get(),
    ];
    let (_, place) = add_offset(place, whole, car);
    shared.car().camera_position[0].set(place[0]);
    shared.car().camera_position[1].set(place[1]);
    shared.car().camera_position[2].set(place[2]);
}

/// The exact version, for `physics::camera_avoid_ground`.
pub fn place_camera(
    camera: &CameraWork,
    lights_data: &LightsData,
    view_records: &ViewAndRecords,
    numbers: &Numbers,
    frame_work: &FrameWork,
    shared: &RaceShared,
) {
    use crate::math::{rotate_point_about_y, rotate_point_about_y_ebx, rotate_point_about_y_ecx};
    let second = || {
        let turns = [camera.view_turn_x, camera.view_turn_y, camera.view_turn_z];
        let parts = turns.map(|turn| view_turn_part(turn as i32, numbers));
        let turned =
            { crate::scene::rotate_point::rotate_point(&shared.car().crash_matrix, parts) };
        let k = numbers.k65536.get();
        turned.map(|e| crate::math::x87::ext_to_int(crate::math::x87::ext_times(e, k)))
    };
    place_with(
        camera,
        lights_data,
        view_records,
        frame_work,
        shared,
        (
            rotate_point_about_y_ebx::rotate_point_about_y_ebx,
            rotate_point_about_y_ecx::rotate_point_about_y_ecx,
            rotate_point_about_y::rotate_point_about_y,
        ),
        &second,
    );
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = int [v + 0] / [k65536]`; the value is `o`'s ten bytes.
fn view_turn_part(v: i32, numbers: &Numbers) -> [u8; 10] {
    let t0 = int(v);
    let t1 = ld(numbers.k65536.get());
    let t2 = t0 / t1;
    f80(t2)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87_mem::{Base, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

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
            plain: |a| {
                a[2].set_tbyte(
                    0,
                    view_turn_part(
                        view::<crate::memory::session::Dword>(a[0], 0).get() as i32,
                        crate::test_support::own::race().course.numbers.get(),
                    ),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.place_camera", &blocks(), 1200, 100);
    }
}
