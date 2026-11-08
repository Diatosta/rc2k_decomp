//! The cars' parts made ready to draw (`race::ready_car_parts` -> `race::car_part_matrices`, part
//! R3, IDA 2026-10-18): each car's marks' switches (+0xAB4 .. +0xAD0) from its state, and its four
//! parts' matrices (the wheels: turned about their axles' places) into `flt_6417E0` .., then
//! kept in the block (+0x740 .. +0x8FF) for `race::draw_car`.

use crate::errors::{BadInput, Refused};
use crate::math::cells::copy_cells;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::race::trap::car_at;
use crate::scene::matrix::{multiply_matrices, set_place, set_turn_x, set_turn_y};
use crate::session::Float;

/// The five axle values (`rc2k_core::race::car_parts::axles`) from the model's record
/// (`dword_5C9098` .. of its 0x134-byte record) and the lift.
fn axles(shared: &Shared, model: u32) -> Result<Option<[u32; 5]>, BadInput> {
    let axles = &LIGHT_RECORDS
        .records
        .get(model as usize)
        .ok_or(BadInput("a car model past the 34 records"))?
        .axles;
    let record: [u32; 5] = std::array::from_fn(|k| axles[k].get());
    Ok(rc2k_core::race::car_parts::axles(
        record,
        shared.physics_tables.ground_drop.get(),
    ))
}

/// `race::car_part_matrices` (0x460E7C): car esi's marks' switches and its parts' matrices
/// (see the header): +0x718 the size from +0x1D8 (not below `flt_6140F8`); the switch bytes
/// from +0x84, `dword_62DAD0` / `dword_62DACC`, +0x63C, +0x684, +0x1D4, each cut by its
/// broken part (+0x410 .. +0x424; the original cuts them twice); +0xABC the marks' count x
/// 0x20 (0 when +0x310 is not below `flt_5C5D14` or +0x84 is 0); +0xAAC the steering; the
/// axles from the model's record; the light and place; the car's turn (+0xB4) and place
/// into `flt_5EA08C` / `flt_641760`; each wheel placed at minus its axle, turned about x by
/// its spin (+0x1F8 front, +0x1FC rear) and about y by the steering (front only), placed at
/// its axle with its height (+0x910 ..), into `flt_6417E0` .. `flt_6418A0`. Refuses (before
/// anything) axle values whose scaling would fault.
pub fn car_part_matrices(
    race: &Race,
    car: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::car_parts::{axles, mark_switches, marks_bytes,
    // steering}` (the size's and the speed's floats the x87's).
    use crate::math::x87::loaded;
    use crate::math::x87_load::below_or_unordered;
    use rc2k_core::race::car_parts::{PartState, mark_switches, marks_bytes, steering};
    // `c` the block (what the drawing keeps there: the shown speed and steering, the switches,
    // the marks), `p` the car as the drawing sees it (`race::frame_view`).
    let c = car_at(car)?;
    let p = crate::race::frame_view::car_at(car)?;
    let Some(ax) = axles(&race.shared, p.car_kind.bits())? else {
        return Err(BadInput("a car model whose axle scaling would fault").into());
    };
    let shown = &c.shown_engine_speed;
    shown.set_bits(loaded(p.engine_speed.bits()));
    if below_or_unordered(
        shown.bits(),
        race.shared.physics_tables.idle_engine_speed.bits(),
    ) {
        shown.set_bits(loaded(race.shared.physics_tables.idle_engine_speed.bits()));
    }
    let on = p.lights_on.bits();
    let s = mark_switches(&PartState {
        on,
        marks_mask: scene.lights.work.marks_mask.get(),
        lamps_mask: scene.lights.work.lamps_mask.get(),
        inside: p.brake.bits(),
        mode: p.camera_view.bits(),
        reversing: p.gear.bits(),
        broken: std::array::from_fn(|k| p.electrical_failures[1 + k].bits()),
        flags: p.parts_present.bits(),
    });
    c.marks_on.set(s.marks_on);
    c.mode_on.set(s.mode_on);
    c.part_switches
        .write_at(0, &s.bytes, "a car block's switches")?;
    let slow = below_or_unordered(
        p.electrics_damage.bits(),
        race.course.numbers.hundred.bits(),
    );
    c.marks_count.set(marks_bytes(s.count, slow, on));
    c.shown_steering.set(steering(p.steer_angle_visual.bits()));
    let [front, rear_lift, height, front_z, _] = ax;
    race.physics.camera.axle_front.set(front);
    race.physics.camera.axle_front_neg.set(front.wrapping_neg());
    race.physics.camera.axle_rear_lift.set(rear_lift);
    race.physics.camera.axle_height.set(height);
    race.physics.camera.axle_front_z.set(front_z);
    race.physics.camera.car_axles.set(ax[4]);
    scene.drawing.object_work.size.set(0x10);
    scene.drawing.object_work.part_values[2].set(c.shown_steering.get());
    scene.drawing.object_work.part_values[0].set_bits(p.driven_wheel_turns[0].bits());
    scene.drawing.object_work.part_values[1].set_bits(p.driven_wheel_turns[1].bits());
    for i in 0..3 {
        scene.drawing.object_work.place[i].set_bits(p.position[i].bits());
    }
    copy_cells(&p.crash_matrix, &race.frame.matrices.body_turn_matrix, 16);
    for i in 0..4 {
        scene.drawing.object_work.part_weights[i].set_bits(p.wheel_spring_force[i].bits());
    }
    set_place(
        &race.frame.matrices,
        [
            scene.drawing.object_work.place[0].get(),
            scene.drawing.object_work.place[1].get(),
            scene.drawing.object_work.place[2].get(),
        ],
    );
    crate::math::cells::copy_cells(
        &race.frame.matrices.placed_matrix,
        &race.frame.frame_work.placed_matrices,
        16,
    );
    // (x, z, the x turn, the height, the matrix): the four wheels.
    let (c, o) = (&race.physics.camera, &scene.drawing.object_work);
    let parts = |n: usize| &race.frame.frame_work.part_matrices[n..n + 16];
    for (n, (x, z, spin, lift, out)) in [
        (
            &c.axle_front_neg,
            &c.axle_height,
            &o.part_values[0],
            &o.part_weights[0],
            parts(0),
        ),
        (
            &c.axle_front,
            &c.axle_height,
            &o.part_values[0],
            &o.part_weights[1],
            parts(16),
        ),
        (
            &c.axle_front_neg,
            &c.axle_front_z,
            &o.part_values[1],
            &o.part_weights[2],
            parts(32),
        ),
        (
            &c.axle_front,
            &c.axle_front_z,
            &o.part_values[1],
            &o.part_weights[3],
            parts(48),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if n == 2 {
            scene.drawing.object_work.part_values[2].set(0);
        }
        set_place(
            &race.frame.matrices,
            [
                x.get().wrapping_neg(),
                race.physics.camera.axle_rear_lift.get().wrapping_neg(),
                z.get().wrapping_neg(),
            ],
        );
        set_turn_x(&race.frame.matrices, spin.get(), scene);
        let placed = &race.frame.matrices.placed_matrix;
        let turned = &race.frame.matrices.turned_matrix;
        let lit = &race.frame.matrices.lit_matrix;
        multiply_matrices(placed, &race.frame.matrices.turn_x_matrix, turned);
        set_turn_y(
            &race.frame.matrices,
            scene.drawing.object_work.part_values[2].get(),
            scene,
        );
        multiply_matrices(turned, &race.frame.matrices.turn_y_matrix, lit);
        set_place(&race.frame.matrices, [x.get(), lift.get(), z.get()]);
        multiply_matrices(lit, placed, out);
    }
    Ok(())
}

/// `race::ready_car_parts` (0x460D5C): for each race car with its parts shown (+0x25C):
/// `car_part_matrices` (esi the block), then the six matrices kept in the block
/// (+0x740 .. +0x8BF) and the body's turn times the car's (`flt_5EA3CC`) at +0x8C0. Refuses
/// (before anything) no race cars (the original's loop would run away), more than eight, or
/// one without its block.
pub fn ready_car_parts(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let frame = &race.frame;
    // Converted: `rc2k_core::race::car_parts::{check_part_cars, parts_shown,
    // KEPT_MATRICES}`.
    use rc2k_core::race::car_parts::{KEPT_MATRICES, check_part_cars, parts_shown};
    let count = race.shared.view.race_cars.get();
    check_part_cars(count, |n| race.shared.work.car_block(n as usize))?;
    // The loop's test is at its end: no cars still looks at car 0.
    let mut c = 0u32;
    loop {
        let car = race.shared.work.car_block(c as usize);
        let block = car_at(car)?;
        if parts_shown(crate::race::frame_view::car_at(car)?.driver_kind.get()) {
            car_part_matrices(race, car, scene)?;
            let wheels = &block.part_matrices;
            let kept: [&[Float]; KEPT_MATRICES as usize] = [
                &wheels[..16],
                &wheels[16..32],
                &wheels[32..48],
                &wheels[48..],
                &block.body_matrix,
                &block.place_matrix,
            ];
            let parts = &frame.frame_work.part_matrices;
            let from: [&[Float]; KEPT_MATRICES as usize] = [
                &parts[..16],
                &parts[16..32],
                &parts[32..48],
                &parts[48..],
                &frame.matrices.body_turn_matrix,
                &frame.frame_work.placed_matrices[..16],
            ];
            for (from, to) in from.into_iter().zip(kept) {
                copy_cells(from, to, 16);
            }
            multiply_matrices(
                &frame.matrices.body_turn_matrix,
                &frame.frame_work.placed_matrices,
                &frame.matrices.view_place,
            );
            copy_cells(&frame.matrices.view_place, &block.whole_matrix, 16);
        }
        c += 1;
        if c == count {
            break;
        }
    }
    Ok(())
}
