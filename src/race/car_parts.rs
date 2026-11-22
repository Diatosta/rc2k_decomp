//! Makes the cars' parts ready to draw (`ready_car_parts`, `car_part_matrices`): each car's marks'
//! switches (`marks_on`, `mode_on`, `part_switches`, `marks_count`) from its state, and its four
//! parts' matrices (the wheels, turned about their axles' places) into `part_matrices`, then kept
//! in the car's drawing state (`race::car_parts_store`) for `draw_car`.

use crate::errors::{BadInput, Refused};
use crate::physics::figures::GROUND_DROP;
use crate::race::owner::Race;
use crate::scene::matrix::{
    PLACED_BASE, TURN_X_BASE, TURN_Y_BASE, matrix_of, placed, product, turned_x, turned_y,
};

/// The five axle values (`rc2k_core::race::car_parts::axles`) from the model's record
/// (`light_records.records[0].axles[0]` .. of its 0x134-byte record) and the lift.
fn axles(race: &Race, model: u32) -> Result<Option<[u32; 5]>, BadInput> {
    let axles = race
        .light_records()
        .read(|l| l.records.get(model as usize).map(|record| record.axles))
        .ok_or(BadInput("a car model past the 34 records"))?;
    let record: [u32; 5] = std::array::from_fn(|k| axles[k]);
    Ok(rc2k_core::race::car_parts::axles(record, GROUND_DROP))
}

/// For a car: `shown_engine_speed` the engine speed (not below `idle_engine_speed`); the switch
/// bytes from `lights_on`, `marks_mask` / `lamps_mask`, `brake`, `camera_view`, `gear` and
/// `parts_present`, each cut by its broken part (`electrical_failures`); `marks_count` the marks'
/// count x 0x20 (0 when `electrics_damage` is not below `hundred` or the lights are off);
/// `shown_steering` the steering; the axles from the model's record; the light and place; the car's
/// turn (`crash_matrix`) and place into `body_turn_matrix` / `placed_matrix`; each wheel placed
/// at minus its axle, turned about x by its spin (`driven_wheel_turns`: front, rear) and about y by
/// the steering (front only), placed at its axle with its height (`wheel_spring_force`), into
/// `part_matrices`. Refused (before anything): axle values whose
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
    // `c` is the block (the shown steering, the marks on), `part` the car's drawing state (the shown
    // speed, the switches, the marks), `p` the car as the drawing sees it (`frame_view`).
    let c = race.shared.race_car(car)?;
    let block_number = race.shared.work.get().block_number(car)?;
    let p = race.frame.cars.car(block_number)?;
    let Some(ax) = axles(race, p.car_kind.get())? else {
        return Err(BadInput("a car model whose axle scaling would fault").into());
    };
    let mut shown = loaded(&p.engine_speed);
    if below_or_unordered(
        shown,
        race.shared.physics_tables.get().idle_engine_speed.get(),
    ) {
        shown = loaded(&race.shared.physics_tables.get().idle_engine_speed);
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
    let slow = below_or_unordered(p.electrics_damage.get(), race.course.numbers.hundred.get());
    {
        let mut parts = race.car_parts_mut();
        let part = &mut parts.cars[block_number];
        part.shown_engine_speed = shown;
        part.mode_on = s.mode_on;
        part.part_switches[..s.bytes.len()].copy_from_slice(&s.bytes);
        part.marks_count = marks_bytes(s.count, slow, on);
    }
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
    race.frame.frame_work.borrow_mut().placed_matrix = car_place;
    // (x, z, the x turn, the height, the matrix): the four wheels.
    // The loop below writes part value 2, so the values it reads are copied first.
    let (c, o) = (*race.physics.camera(), *scene.object_work());
    for (n, (x, z, spin, lift, part)) in [
        (
            c.axle_front_neg,
            c.axle_height,
            o.part_values[0],
            o.part_weights[0],
            0,
        ),
        (
            c.axle_front,
            c.axle_height,
            o.part_values[0],
            o.part_weights[1],
            1,
        ),
        (
            c.axle_front_neg,
            c.axle_front_z,
            o.part_values[1],
            o.part_weights[2],
            2,
        ),
        (
            c.axle_front,
            c.axle_front_z,
            o.part_values[1],
            o.part_weights[3],
            3,
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
        let mut work = race.frame.frame_work.borrow_mut();
        work.part_matrices[part] = product(&lit, &at_axle, work.part_matrices[part]);
    }
    Ok(())
}

/// For each race car with its parts shown (`driver_kind`): `car_part_matrices`, then the six
/// matrices kept in the drawing state and the body's turn times the car's (`view_place`) in `whole_matrix`.
/// Refused (before anything): no race cars, more than eight, or one without its block.
pub(super) fn ready_car_parts(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let frame = &race.frame;
    use rc2k_core::race::car_parts::{check_part_cars, parts_shown};
    let count = race.shared.view.get().race_cars.get();
    check_part_cars(count, |n| {
        u32::from(race.shared.work.get().has_car_block(n as usize))
    })?;
    // With no cars the loop still looks at car 0.
    let mut c = 0u32;
    loop {
        let car = c as usize;
        // Refused when the car has no block.
        race.shared.race_car(car)?;
        let block_number = race.shared.work.get().block_number(car)?;
        let view = race.frame.cars.car(block_number)?;
        if parts_shown(view.driver_kind.get()) {
            car_part_matrices(race, car, scene)?;
            let work = *frame.frame_work.borrow();
            let body_turn = frame.matrices.borrow().body_turn_matrix;
            let old_place = frame.view_kept.borrow().view_place;
            let whole = product(&body_turn, &work.placed_matrix, old_place);
            frame.view_kept.borrow_mut().view_place = whole;
            let mut parts = race.car_parts_mut();
            let part = &mut parts.cars[block_number];
            part.part_matrices = std::array::from_fn(|k| work.part_matrices[k / 16][k % 16]);
            part.body_matrix = body_turn;
            part.place_matrix = work.placed_matrix;
            part.whole_matrix = whole;
        }
        c += 1;
        if c == count {
            break;
        }
    }
    Ok(())
}
