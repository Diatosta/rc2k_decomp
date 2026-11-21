//! Makes the cars' parts ready to draw (`ready_car_parts`, `car_part_matrices`): each car's marks'
//! switches (`marks_on`, `mode_on`, `part_switches`, `marks_count`) from its state, and its four
//! parts' matrices (the wheels, turned about their axles' places) into `part_matrices`, then kept
//! in the block for `draw_car`.

use crate::errors::{BadInput, Refused};
use crate::math::cells::copy_cells;
use crate::memory::session::PlainFloat;
use crate::physics::figures::GROUND_DROP;
use crate::race::owner::Race;
use crate::scene::matrix::{
    PLACED_BASE, TURN_X_BASE, TURN_Y_BASE, matrix_of, placed, product, store_matrix, turned_x,
    turned_y,
};

/// The five axle values (`rc2k_core::race::car_parts::axles`) from the model's record
/// (`LightsAndGear.light_records.records[0].axles[0]` .. of its 0x134-byte record) and the lift.
fn axles(race: &Race, model: u32) -> Result<Option<[u32; 5]>, BadInput> {
    let axles = &race
        .light_records()
        .records
        .get(model as usize)
        .ok_or(BadInput("a car model past the 34 records"))?
        .axles;
    let record: [u32; 5] = std::array::from_fn(|k| axles[k].get());
    Ok(rc2k_core::race::car_parts::axles(record, GROUND_DROP))
}

/// For a car: `shown_engine_speed` the engine speed (not below `idle_engine_speed`); the switch
/// bytes from `lights_on`, `marks_mask` / `lamps_mask`, `brake`, `camera_view`, `gear` and
/// `parts_present`, each cut by its broken part (`electrical_failures`); `marks_count` the marks'
/// count x 0x20 (0 when `electrics_damage` is not below `hundred` or the lights are off);
/// `shown_steering` the steering; the axles from the model's record; the light and place; the car's
/// turn (`crash_matrix`) and place into `body_turn_matrix` / `placed_matrices`; each wheel placed
/// at minus its axle, turned about x by its spin (`driven_wheel_turns`: front, rear) and about y by
/// the steering (front only), placed at its axle with its height (`wheel_spring_force`), into
/// `part_matrices` (`FrameWork.part_matrices[48]`). Refused (before anything): axle values whose
/// scaling would fault.
fn car_part_matrices(
    race: &Race,
    car: usize,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    // The size's and the speed's floats are the x87's; the rest is in `rc2k_core::race::car_parts`.
    use crate::math::x87::below_or_unordered;
    use crate::math::x87::loaded;
    use rc2k_core::race::car_parts::{PartState, mark_switches, marks_bytes, steering};
    // `c` is the block (what the drawing keeps there: the shown speed and steering, the switches,
    // the marks), `p` the car as the drawing sees it (`frame_view`).
    let c = race.shared.race_car(car)?;
    let p = race
        .frame
        .cars
        .car(race.shared.work.get().block_number(car)?)?;
    let Some(ax) = axles(race, p.car_kind.get())? else {
        return Err(BadInput("a car model whose axle scaling would fault").into());
    };
    let shown = &c.shown_engine_speed;
    shown.set(loaded(&p.engine_speed));
    if below_or_unordered(
        shown,
        race.shared.physics_tables.get().idle_engine_speed.get(),
    ) {
        shown.set(loaded(&race.shared.physics_tables.get().idle_engine_speed));
    }
    let on = p.lights_on.get();
    let s = mark_switches(&PartState {
        on,
        marks_mask: scene.light_work().marks_mask,
        lamps_mask: scene.light_work().lamps_mask,
        inside: p.brake.get(),
        mode: p.camera_view.get(),
        reversing: p.gear.get(),
        broken: std::array::from_fn(|k| p.electrical_failures[1 + k].get()),
        flags: p.parts_present.get(),
    });
    c.marks_on.set(s.marks_on);
    c.mode_on.set(s.mode_on);
    c.part_switches
        .write_at(0, &s.bytes, "a car block's switches")?;
    let slow = below_or_unordered(p.electrics_damage.get(), race.course.numbers.hundred.get());
    c.marks_count.set(marks_bytes(s.count, slow, on));
    c.shown_steering.set(steering(p.steer_angle_visual.get()));
    let [front, rear_lift, height, front_z, _] = ax;
    {
        let mut camera = race.physics.camera_mut();
        camera.axle_front = front;
        camera.axle_front_neg = front.wrapping_neg();
        camera.axle_rear_lift = rear_lift;
        camera.axle_height = height;
        camera.axle_front_z = front_z;
    }
    scene.object_work_mut().part_values[2] = c.shown_steering.get();
    scene.object_work_mut().part_values[0] = p.driven_wheel_turns[0].get();
    scene.object_work_mut().part_values[1] = p.driven_wheel_turns[1].get();
    race.frame.matrices.borrow_mut().body_turn_matrix = matrix_of(&p.crash_matrix);
    for (weight, force) in scene
        .object_work_mut()
        .part_weights
        .iter_mut()
        .zip(&p.wheel_spring_force)
    {
        *weight = force.get();
    }
    let matrices = &race.frame.matrices;
    let car_place = placed(
        PLACED_BASE,
        [
            p.position[0].get(),
            p.position[1].get(),
            p.position[2].get(),
        ],
    );
    store_matrix(&race.frame.frame_work.get().placed_matrices, &car_place);
    // (x, z, the x turn, the height, the matrix): the four wheels.
    // The loop below writes part value 2, so the values it reads are copied first.
    let (c, o) = (*race.physics.camera(), *scene.object_work());
    let parts = |n: usize| &race.frame.frame_work.get().part_matrices[n..n + 16];
    for (n, (x, z, spin, lift, out)) in [
        (
            c.axle_front_neg,
            c.axle_height,
            o.part_values[0],
            o.part_weights[0],
            parts(0),
        ),
        (
            c.axle_front,
            c.axle_height,
            o.part_values[0],
            o.part_weights[1],
            parts(16),
        ),
        (
            c.axle_front_neg,
            c.axle_front_z,
            o.part_values[1],
            o.part_weights[2],
            parts(32),
        ),
        (
            c.axle_front,
            c.axle_front_z,
            o.part_values[1],
            o.part_weights[3],
            parts(48),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        if n == 2 {
            scene.object_work_mut().part_values[2] = 0;
        }
        let back = placed(
            PLACED_BASE,
            [
                x.wrapping_neg(),
                c.axle_rear_lift.wrapping_neg(),
                z.wrapping_neg(),
            ],
        );
        let (turn_x, _) = turned_x(race.engine_tables(), TURN_X_BASE, spin);
        let turned = {
            let mut kept = matrices.borrow_mut();
            kept.turned_matrix = product(&back, &turn_x, kept.turned_matrix);
            kept.turned_matrix
        };
        let (turn_y, _) = turned_y(
            race.engine_tables(),
            TURN_Y_BASE,
            scene.object_work().part_values[2],
        );
        let lit = {
            let mut kept = matrices.borrow_mut();
            kept.lit_matrix = product(&turned, &turn_y, kept.lit_matrix);
            kept.lit_matrix
        };
        let at_axle = placed(back, [x, lift, z]);
        store_matrix(out, &product(&lit, &at_axle, matrix_of(out)));
    }
    Ok(())
}

/// For each race car with its parts shown (`driver_kind`): `car_part_matrices`, then the six
/// matrices kept in the block and the body's turn times the car's (`view_place`) in `whole_matrix`.
/// Refused (before anything): no race cars, more than eight, or one without its block.
pub(super) fn ready_car_parts(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let frame = &race.frame;
    use rc2k_core::race::car_parts::{KEPT_MATRICES, check_part_cars, parts_shown};
    let count = race.shared.view.get().race_cars.get();
    check_part_cars(count, |n| {
        u32::from(race.shared.work.get().has_car_block(n as usize))
    })?;
    // With no cars the loop still looks at car 0.
    let mut c = 0u32;
    loop {
        let car = c as usize;
        let block = race.shared.race_car(car)?;
        let view = race
            .frame
            .cars
            .car(race.shared.work.get().block_number(car)?)?;
        if parts_shown(view.driver_kind.get()) {
            car_part_matrices(race, car, scene)?;
            let wheels = &block.part_matrices;
            let kept: [&[PlainFloat]; KEPT_MATRICES as usize] = [
                &wheels[..16],
                &wheels[16..32],
                &wheels[32..48],
                &wheels[48..],
                &block.body_matrix,
                &block.place_matrix,
            ];
            let work = frame.frame_work.get();
            let parts = &work.part_matrices;
            for (from, to) in [&parts[..16], &parts[16..32], &parts[32..48], &parts[48..]]
                .into_iter()
                .zip(&kept)
            {
                copy_cells(from, to, 16);
            }
            let body_turn = frame.matrices.borrow().body_turn_matrix;
            for (to, value) in kept[4].iter().zip(body_turn) {
                to.set(value);
            }
            copy_cells(&work.placed_matrices[..16], kept[5], 16);
            let whole = product(
                &body_turn,
                &matrix_of(&frame.frame_work.get().placed_matrices),
                matrix_of(&frame.view_kept.get().view_place),
            );
            store_matrix(&frame.view_kept.get().view_place, &whole);
            copy_cells(&frame.view_kept.get().view_place, &block.whole_matrix, 16);
        }
        c += 1;
        if c == count {
            break;
        }
    }
    Ok(())
}
