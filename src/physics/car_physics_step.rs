//! One physics step of the work car (the copy loaded by `drive::load_car`). In order: the car's
//! word stirred, its specs, gear ratios, motion length, handling and the race timers; the step's
//! notes cleared (sounds, the camera's probe counts, the car's contact fields, the hit counters)
//! and the height before (`vertical_speed`); the inverse matrix, the ground, skid marks, the
//! probes, the timers, the surfaces and their noise, the state saved, the forces cleared, wheel
//! weights, downforce, the gearbox and the drive force; the side speed held within
//! `side_speed_limit`; the driven wheels (grip, step, speeds, the free revs); the engine speed
//! held between `idle_engine_speed` (only off the clutch, `brake` 0) and `rev_limit`; wheels from
//! revs, air drag, springs, the body against the ground, the wheels' forces and torques, the
//! motion (move, spin, levers, matrix, angles); the speed's length into `top_speed`. Then the
//! contacts, up to `contact_steps` rounds (`contact_step` the count): contacts cleared, the
//! ground, the crossed edge, the walls, the probes, the other cars, the faces, the motion kept,
//! the contacts' forces and torques; a round that restored nothing (`clear_place_count` 0) ends
//! it, else the car stalls when still, its motion is redone and the round counted; with all
//! rounds used: `physics::limit_motion`, `physics::find_clear_place`, `step_count` + 1. With
//! rounds done, the motion is put back and limited. Then the ground, the stuck reset, the engine
//! sound, the wheels' spin, the checkpoints, the pace notes (`physics::pace_notes_step`), the
//! count down, the wheel angles, the mean drive speed and the height change
//! (`vertical_speed_change`), the motion length, the damage model with its parts and notices,
//! neutral, `contact_flag`, the camera (`physics::camera_frame`, when the current car is in
//! `listed_car_sounds`), the spin on the ground, broken parts, lights, the forces' sum, the wheel
//! load, the engine sound's share, the time mark, a false start and the value notice. The holds
//! are in `physics/flow.rs`.
//!
//! Callers: `physics::race_cars_step`, `race::step_replay_cars`, `drive::replay_step`.
//! Refuses (before anything) a current car past 7, and what its calls refuse; the work car and
//! the physics' variables are put back then (the callees put back their own
//! other writes).

use crate::effects::cell_x87::CellX87;
use crate::errors::{BadInput, Refused};
use crate::math::x87::loaded;
use crate::physics as ph;
use crate::physics::car_store::Car;
use crate::physics::flow::{hold_high, hold_low, negate};
use crate::physics::tables_store::PhysicsTables;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::switches_store::FrameSwitches;
use crate::scene::matrix3::rounded_value;
use crate::session::DwordLike;

/// The motion: moved, spun, levered, the inverse matrix, the angles.
fn motion(
    course: &Course,
    matrices: &SceneMatrices,
    switches: &FrameSwitches,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    ph::move_car::move_car(car, physics_tables, course.numbers.get(), scene);
    ph::spin_matrix::spin_matrix(course, matrices, physics_tables, car, scene);
    ph::lever_pushes::lever_pushes(course.numbers.get(), switches, physics_tables, car, scene);
    crate::race::start_place::invert_car_matrix(course, car, scene);
    ph::update_car_angles::update_car_angles(course, car, scene);
}

/// The steps. Each callee reads what it needs from memory.
pub fn steps(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (system, sound, scene) = (o.system, o.sound, o.scene);
    let inputs = crate::physics::step_inputs::StepInputs::of(race);
    ph::stir_car_word::stir_car_word(race.shared.car());
    ph::load_car_specs::load_car_specs(
        race.physics.car_specs(),
        &race.course,
        race.shared.physics_tables.get(),
        race.shared.car(),
        race.gear(),
        scene,
    )?;
    ph::make_gear_ratios::make_gear_ratios(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.course.numbers.get(),
        race.gear(),
        scene,
    )?;
    ph::store_motion_length::store_motion_length(race.shared.car());
    ph::set_handling::set_handling(
        race.physics.damage(),
        race.course.numbers.get(),
        &race.shared,
    )?;
    ph::race_timers::race_timers(o, race)?;
    for cell in [
        &race.frame.sound_spots().sound_scratch_a,
        &race.frame.sound_spots().sound_scratch_b,
        &race.frame.sound_spots().sound_scratch_c,
    ] {
        cell.set(0);
    }
    race.physics.contact.get().contact_normal_speed[0].set(0.0);
    for cell in [
        &race.physics.camera().ground_probes,
        &race.physics.camera().ground_found,
        &race.physics.camera().ground_same,
    ] {
        cell.set(0);
    }
    race.shared
        .physics_tables
        .get()
        .vertical_speed
        .set(loaded(&race.shared.car().velocity[1]));
    let cleared: [&dyn DwordLike; 5] = [
        &race.shared.physics_tables.get().clear_place_mark,
        &race.shared.car().contact_flag,
        &race.shared.physics_tables.get().scene_point_hits,
        &race.shared.physics_tables.get().object_hits,
        &race.shared.physics_tables.get().point_hits,
    ];
    for cell in cleared {
        cell.set(0);
    }
    for figure in race
        .shared
        .car()
        .contact_point
        .iter()
        .chain([&race.shared.car().contact_depth])
    {
        figure.set(0.0);
    }
    crate::race::start_place::invert_car_matrix(&race.course, race.shared.car(), scene);
    ph::feel_ground::feel_ground(&inputs.ground(), scene)?;
    ph::start_skid_mark::start_skid_mark(race.shared.car());
    ph::probe_and_keep::probe_and_keep(&inputs.ground(), scene)?;
    ph::ease_car_value::ease_car_value(race.shared.car(), race.shared.vars.get());
    ph::count_down_timers::count_down_timers(race.shared.car(), race.shared.vars.get());
    ph::wheel_surfaces::wheel_surfaces(&inputs.wheels(), scene)?;
    ph::surface_noise::surface_noise(
        race.course.stage_prefs.get(),
        race.course.numbers.get(),
        &race.shared,
        scene,
    )?;
    ph::save_car_state::save_car_state(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.physics.contact.get(),
    )?;
    ph::clear_forces::clear_forces(race.shared.physics_tables.get())?;
    ph::set_wheel_weights::set_wheel_weights(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.physics.contact.get(),
        race.course.numbers.get(),
        scene,
    );
    ph::set_downforce::set_downforce(race.shared.car(), race.shared.physics_tables.get());
    ph::gearbox_step::gearbox_step(race.course.numbers.get(), &race.shared, race.gear(), scene)?;
    { ph::drive_force::drive_force(&race.course, &race.shared, race.gear(), scene) }?;
    // The side speed held within the limit.
    race.shared
        .car()
        .side_speed_held
        .set(race.shared.car().local_velocity[0].get());
    hold_high(
        &race.shared.car().side_speed_held,
        &race.shared.physics_tables.get().side_speed_limit,
    );
    negate(&race.shared.physics_tables.get().side_speed_limit);
    hold_low(
        &race.shared.car().side_speed_held,
        &race.shared.physics_tables.get().side_speed_limit,
    );
    negate(&race.shared.physics_tables.get().side_speed_limit);
    ph::drive_grip_forces::drive_grip_forces(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.course.numbers.get(),
        race.engine_tables(),
        scene,
    );
    ph::drive_wheels_step::drive_wheels_step(
        &race.course,
        race.shared.vars.get(),
        race.shared.physics_tables.get(),
        race.shared.car(),
        scene,
    )?;
    ph::settle_drive_speeds::settle_drive_speeds(race.course.numbers.get(), &race.shared, scene)?;
    ph::free_rev_in_neutral::free_rev_in_neutral(&race.course, &race.shared, race.gear(), scene);
    ph::free_rev_clutch::free_rev_clutch(&race.course, &race.shared, race.gear(), scene)?;
    // The engine's speed held.
    if race.shared.car().brake.get() == 0 {
        hold_low(
            &race.shared.car().engine_speed,
            &race.shared.physics_tables.get().idle_engine_speed,
        );
    }
    hold_high(
        &race.shared.car().engine_speed,
        &race.shared.physics_tables.get().rev_limit,
    );
    ph::wheels_from_revs::wheels_from_revs(race.shared.car(), race.shared.physics_tables.get())?;
    ph::air_drag::air_drag(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.course.numbers.get(),
    );
    ph::wheel_springs::wheel_springs(
        race.car_blocks(),
        race.physics.contact.get(),
        race.course.numbers.get(),
        &race.shared,
        scene,
    );
    ph::crash_contact::crash_contact(&inputs.contact(sound), scene)?;
    ph::apply_wheel_forces::apply_wheel_forces(
        race.shared.car(),
        race.shared.physics_tables.get(),
        scene,
    )?;
    ph::apply_wheel_torques::apply_wheel_torques(
        race.shared.physics_tables.get(),
        race.physics.contact.get(),
        race.shared.car(),
    )?;
    motion(
        &race.course,
        race.frame.matrices.get(),
        race.shared.switches.get(),
        race.shared.physics_tables.get(),
        race.shared.car(),
        scene,
    );
    race.shared
        .physics_tables
        .get()
        .top_speed
        .set(rounded_value(ph::vector_length::vector_length(
            &race.shared.car().velocity,
        )));
    // The contacts' rounds.
    race.shared.physics_tables.get().contact_step.set(0);
    loop {
        ph::clear_contacts::clear_contacts(race.shared.physics_tables.get())?;
        ph::feel_ground::feel_ground(&inputs.ground(), scene)?;

        ph::crossed_edge::crossed_edge(&inputs.contact(sound), scene)?;
        ph::wall_contacts::wall_contacts(&inputs.contact(sound), scene)?;
        ph::ground_contact::ground_contact(&inputs.contact(sound), scene)?;
        { ph::car_collisions::car_collisions(&inputs.collisions(sound), scene) }?;
        ph::face_contacts::face_contacts(&inputs.contact(sound), scene)?;
        ph::keep_motion::keep_motion(race.shared.car(), race.shared.physics_tables.get());
        ph::apply_contact_forces::apply_contact_forces(
            race.shared.car(),
            race.shared.physics_tables.get(),
            scene,
        )?;
        ph::apply_contact_torques::apply_contact_torques(
            race.shared.physics_tables.get(),
            race.physics.contact.get(),
            race.shared.car(),
        )?;

        if race.shared.physics_tables.get().clear_place_count.get() == 0 {
            break;
        }
        ph::stall_when_still::stall_when_still(
            race.shared.car(),
            race.shared.physics_tables.get(),
            scene,
        );
        ph::keep_motion_vectors::keep_motion_vectors(
            race.shared.car(),
            race.shared.physics_tables.get(),
        );

        motion(
            &race.course,
            race.frame.matrices.get(),
            race.shared.switches.get(),
            race.shared.physics_tables.get(),
            race.shared.car(),
            scene,
        );
        race.shared.physics_tables.get().clear_place_count.set(0);
        race.shared.physics_tables.get().contact_step.set(
            race.shared
                .physics_tables
                .get()
                .contact_step
                .get()
                .wrapping_add(1),
        );
        if race.shared.physics_tables.get().contact_step.get()
            >= race.shared.physics_tables.get().contact_steps.get()
        {
            ph::limit_motion::limit_motion(
                race.shared.car(),
                race.shared.physics_tables.get(),
                scene,
            );
            ph::find_clear_place::find_clear_place(&inputs.clear_place(sound), scene)?;

            race.shared.physics_tables.get().step_count.set(
                race.shared
                    .physics_tables
                    .get()
                    .step_count
                    .get()
                    .wrapping_add(1),
            );
            break;
        }
    }
    if race.shared.physics_tables.get().contact_step.get() != 0 {
        ph::put_back_motion_vectors::put_back_motion_vectors(
            race.shared.car(),
            race.shared.physics_tables.get(),
        );
        ph::limit_motion::limit_motion(race.shared.car(), race.shared.physics_tables.get(), scene);
    }
    ph::feel_ground::feel_ground(&inputs.ground(), scene)?;
    ph::reset_when_stuck::reset_when_stuck(&inputs.reset(), scene)?;
    ph::engine_sound::engine_sound(
        race.course.numbers.get(),
        race.frame.matrices.get(),
        &race.shared,
        scene,
        sound,
    )?;
    ph::spin_wheels::spin_wheels(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.course.numbers.get(),
        scene,
    );
    ph::track_checkpoints::track_checkpoints(&inputs.checkpoints(), scene, sound, o.audio)?;
    ph::pace_notes_step::pace_notes_step(&inputs.checkpoints(), scene, sound, o.audio)?;
    ph::count_down_notice::count_down_notice(race.shared.car());
    ph::accumulate_mud::accumulate_mud(
        race.shared.car(),
        race.shared.vars.get(),
        race.kind_tables(),
        scene,
    );
    x87_1(race);
    ph::store_motion_length::store_motion_length(race.shared.car());
    ph::damage_step::damage_step(
        race.shared.car(),
        race.physics.damage(),
        race.shared.work.get(),
    )?;
    ph::hit_part_checks::hit_part_checks(
        race.course.numbers.get(),
        race.frame.matrices.get(),
        &race.shared,
        race.kind_tables(),
        scene,
        sound,
    )?;
    ph::damage_notices::damage_notices(race.shared.car(), race.physics.damage())?;
    ph::take_notice::take_notice(race.shared.car());
    ph::drop_to_neutral::drop_to_neutral(race.shared.car(), race.shared.physics_tables.get());
    // A step that set only flags nobody reads is left out.
    // The step's three callers run it for the cars from 0 up to the race's car count, at most 8.
    let car = race.shared.work.get().current_car.get() as usize;
    let listed = race
        .frame
        .frame_work
        .get()
        .listed_car_sounds
        .get(car)
        .ok_or(BadInput("a car number past the eight"))?
        .get();
    if listed != u32::MAX {
        ph::camera_frame::camera_frame(&inputs.camera(), scene)?;
    }
    ph::hold_spin_on_ground::hold_spin_on_ground(
        race.shared.car(),
        race.shared.physics_tables.get(),
    );
    ph::drop_broken_parts::drop_broken_parts(
        race.course.numbers.get(),
        race.shared.work.get(),
        race.shared.car(),
        scene,
    )?;
    ph::car_lighting::car_lighting(&inputs.lighting(), scene, system)?;
    ph::sum_forces::sum_forces(race.shared.physics_tables.get());
    ph::blend_wheel_load::blend_wheel_load(
        race.shared.car(),
        race.shared.physics_tables.get(),
        race.course.numbers.get(),
        scene,
    );
    ph::share_engine_sound::share_engine_sound(race.shared.car(), sound, race.shared.work.get())?;
    ph::false_start::false_start(&inputs.reset(), scene)?;
    ph::value_notice::value_notice(race.shared.car());
    Ok(())
}

pub fn car_physics_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    if race.shared.work.get().current_car.get() > 7 {
        return Err(BadInput("a current car past 7 (dword_641680's room)").into());
    }
    steps(o, race)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[mean_drive_speed] = ([drive_speed_a] + [drive_speed_b]) * [half]`
/// `[vertical_speed_change] = [car vertical velocity] - [vertical_speed]`
fn x87_1(race: &Race) {
    let t0 = race.shared.car().drive_speed_a.ld();
    let t1 = race.shared.car().drive_speed_b.ld();
    let t2 = t0 + t1;
    let t3 = race.course.numbers.get().half.ld();
    let t4 = t2 * t3;
    race.shared.car().mean_drive_speed.st(t4);
    let t5 = race.shared.car().velocity[1].ld();
    let t6 = race.shared.physics_tables.get().vertical_speed.ld();
    let t7 = t5 - t6;
    race.shared.car().vertical_speed_change.st(t7);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "car_physics_step line 183",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E40, In::F),
                    (0x214138, In::F),
                    (0x31A174, In::F),
                    (0x31A190, In::F),
                    (0x31A194, In::F),
                ],
                writes: &[(0x31A198, Out::F), (0x31A8E4, Out::F)],
            }],
            ulps: 0,
            plain: |_| x87_1(race()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.car_physics_step", &blocks(), 1200, 100);
    }
}
