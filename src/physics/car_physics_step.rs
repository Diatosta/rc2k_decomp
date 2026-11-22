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
//! rounds used: `physics::limit_motion`, `physics::find_clear_place`. With
//! rounds done, the motion is put back and limited. Then the ground, the stuck reset, the engine
//! sound, the wheels' spin, the checkpoints, the pace notes (`physics::pace_notes_step`), the
//! count down, the wheel angles, the mean drive speed and the height change
//! (`vertical_speed_change`), the motion length, the damage model with its parts and notices,
//! neutral, `contact_flag`, the camera (`physics::camera_frame`, when the current car is in
//! `listed_car_sounds`), the spin on the ground, broken parts, lights, the forces' sum, the wheel
//! load, the engine sound's share, the time mark, a false start and the value notice. The holds
//! are in `physics/flow.rs`.
//!
//! Refuses (before anything) a current car past 7, and what its calls refuse; the work car and the
//! physics' variables are put back then (the callees put back their own other writes).

use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::{CellX87, PlainX87};
use crate::math::matrix3::rounded_value;
use crate::math::x87::loaded;
use crate::memory::session::DrawPlain;
use crate::physics as ph;
use crate::physics::car_store::Car;
use crate::physics::figures::SIDE_SPEED_LIMIT;
use crate::physics::flow::{hold_high, hold_high_to, hold_low, hold_low_to, negated};
use crate::physics::tables_store::PhysicsTables;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner::Race;
use crate::race::owner_course::Course;

/// The motion: moved, spun, levered, the inverse matrix, the angles.
fn motion(
    course: &Course,
    matrices: &DrawPlain<SceneMatrices>,
    switches: crate::race::switches_store::FrameSwitchesRef<'_>,
    physics_tables: &PhysicsTables,
    car: &Car,
    scene: &crate::scene::owner::Scene,
) {
    ph::move_car::move_car(car, &course.numbers);
    ph::spin_matrix::spin_matrix(course, matrices, car);
    ph::lever_pushes::lever_pushes(&course.numbers, switches, physics_tables, car);
    crate::race::start_place::invert_car_matrix(course, car, scene);
    ph::update_car_angles::update_car_angles(course, car);
}

/// The steps. Each callee reads what it needs from memory.
pub fn steps(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    let (sound, scene) = (o.sound, o.scene);
    let inputs = crate::physics::step_inputs::StepInputs::of(race);
    ph::stir_car_word::stir_car_word(race.shared.car());
    ph::load_car_specs::load_car_specs(
        race.physics.car_specs(),
        &race.course,
        &mut race.shared.physics_tables_mut(),
        race.shared.car(),
        race.gear(),
    )?;
    ph::make_gear_ratios::make_gear_ratios(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        &race.course.numbers,
        race.gear(),
    )?;
    ph::store_motion_length::store_motion_length(race.shared.car());
    ph::set_handling::set_handling(
        &mut race.physics.damage_mut(),
        &race.course.numbers,
        &race.shared,
    )?;
    ph::race_timers::race_timers(o, race)?;
    {
        let mut camera = race.physics.camera_mut();
        camera.ground_probes = 0;
        camera.ground_found = 0;
        camera.ground_same = 0;
    }
    race.shared.physics_tables_mut().vertical_speed = loaded(race.shared.car().velocity.bits_at(1));
    race.shared.car().contact_flag.set(0);
    race.shared.car().contact_point.set([0.0; 3]);
    race.shared.car().contact_depth.set(0.0);
    crate::race::start_place::invert_car_matrix(&race.course, race.shared.car(), scene);
    ph::feel_ground::feel_ground(&inputs.ground())?;
    ph::start_skid_mark::start_skid_mark(race.shared.car());
    ph::probe_and_keep::probe_and_keep(&inputs.ground())?;
    ph::ease_car_value::ease_car_value(race.shared.car(), race.shared.vars.get());
    ph::count_down_timers::count_down_timers(race.shared.car(), race.shared.vars.get());
    ph::wheel_surfaces::wheel_surfaces(&inputs.wheels(), scene)?;
    ph::surface_noise::surface_noise(
        crate::race::prefs_store::StagePrefsRef::Store(&race.course.stage_prefs),
        &race.course.numbers,
        &race.shared,
    )?;
    ph::save_car_state::save_car_state(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        &mut race.physics.probes().borrow_mut(),
    )?;
    ph::clear_forces::clear_forces(&mut race.shared.physics_tables_mut())?;
    ph::set_wheel_weights::set_wheel_weights(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        race.physics.contact(),
        &race.course.numbers,
    );
    ph::set_downforce::set_downforce(race.shared.car(), &mut race.shared.physics_tables_mut());
    ph::gearbox_step::gearbox_step(&race.course.numbers, &race.shared, race.gear())?;
    { ph::drive_force::drive_force(&race.course, &race.shared, race.gear(), scene) }?;
    // The side speed held within the limit.
    race.shared
        .car()
        .side_speed_held
        .set(race.shared.car().local_velocity.at(0));
    hold_high_to(&race.shared.car().side_speed_held, SIDE_SPEED_LIMIT);
    hold_low_to(
        &race.shared.car().side_speed_held,
        negated(SIDE_SPEED_LIMIT),
    );
    ph::drive_grip_forces::drive_grip_forces(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        &race.course.numbers,
        race.course.grip_change(),
        race.engine_tables(),
        scene,
    );
    ph::drive_wheels_step::drive_wheels_step(
        &race.course,
        race.shared.vars.get(),
        &mut race.shared.physics_tables_mut(),
        race.physics.wheel_work(),
        race.shared.car(),
        scene,
    )?;
    ph::settle_drive_speeds::settle_drive_speeds(&race.course.numbers, &race.shared)?;
    ph::free_rev_in_neutral::free_rev_in_neutral(&race.course, &race.shared, race.gear(), scene);
    ph::free_rev_clutch::free_rev_clutch(&race.course, &race.shared, race.gear(), scene)?;
    // The engine's speed held.
    if race.shared.car().brake.get() == 0 {
        hold_low(
            &race.shared.car().engine_speed,
            &race.shared.physics_tables().idle_engine_speed,
        );
    }
    hold_high(
        &race.shared.car().engine_speed,
        &race.shared.physics_tables().rev_limit,
    );
    ph::wheels_from_revs::wheels_from_revs(race.shared.car(), &race.shared.physics_tables())?;
    ph::air_drag::air_drag(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        &race.course.numbers,
    );
    ph::wheel_springs::wheel_springs(
        race.car_blocks(),
        &race.physics.probes().borrow(),
        &race.course.numbers,
        &race.shared,
        scene,
    );
    ph::crash_contact::crash_contact(&inputs.contact(sound))?;
    ph::apply_wheel_forces::apply_wheel_forces(
        race.shared.car(),
        &race.course.numbers,
        &race.shared.physics_tables(),
    )?;
    ph::apply_wheel_torques::apply_wheel_torques(
        &race.course.numbers,
        &race.shared.physics_tables(),
        race.physics.contact(),
        race.shared.car(),
    )?;
    motion(
        &race.course,
        &race.frame.matrices,
        crate::race::switches_store::FrameSwitchesRef::Store(&race.shared.switches),
        &race.shared.physics_tables(),
        race.shared.car(),
        scene,
    );
    race.shared.physics_tables_mut().top_speed = rounded_value(ph::vector_length::vector_length(
        &race.shared.car().velocity,
    ));
    // The contacts' rounds.
    race.shared.physics_tables_mut().contact_step = 0;
    loop {
        ph::clear_contacts::clear_contacts(&mut race.shared.physics_tables_mut())?;
        ph::feel_ground::feel_ground(&inputs.ground())?;

        ph::crossed_edge::crossed_edge(&inputs.contact(sound))?;
        ph::wall_contacts::wall_contacts(&inputs.contact(sound), scene)?;
        ph::ground_contact::ground_contact(&inputs.contact(sound))?;
        { ph::car_collisions::car_collisions(&inputs.collisions(sound)) }?;
        ph::face_contacts::face_contacts(&inputs.contact(sound), scene)?;
        ph::keep_motion::keep_motion(race.shared.car(), &mut race.shared.physics_tables_mut());
        ph::apply_contact_forces::apply_contact_forces(
            race.shared.car(),
            &race.course.numbers,
            &race.shared.physics_tables(),
        )?;
        ph::apply_contact_torques::apply_contact_torques(
            &race.course.numbers,
            &race.shared.physics_tables(),
            race.physics.contact(),
            race.shared.car(),
        )?;

        if race.shared.physics_tables().clear_place_count == 0 {
            break;
        }
        ph::stall_when_still::stall_when_still(
            race.shared.car(),
            &mut race.shared.physics_tables_mut(),
        );
        ph::keep_motion_vectors::keep_motion_vectors(
            race.shared.car(),
            &mut race.shared.physics_tables_mut(),
        );

        motion(
            &race.course,
            &race.frame.matrices,
            crate::race::switches_store::FrameSwitchesRef::Store(&race.shared.switches),
            &race.shared.physics_tables(),
            race.shared.car(),
            scene,
        );
        race.shared.physics_tables_mut().clear_place_count = 0;
        let step = race.shared.physics_tables().contact_step.wrapping_add(1);
        race.shared.physics_tables_mut().contact_step = step;
        if race.shared.physics_tables().contact_step >= race.shared.physics_tables().contact_steps {
            ph::limit_motion::limit_motion(race.shared.car(), &race.shared.physics_tables());
            ph::find_clear_place::find_clear_place(&inputs.clear_place(sound), scene)?;
            break;
        }
    }
    if race.shared.physics_tables().contact_step != 0 {
        ph::put_back_motion_vectors::put_back_motion_vectors(
            race.shared.car(),
            &race.shared.physics_tables(),
        );
        ph::limit_motion::limit_motion(race.shared.car(), &race.shared.physics_tables());
    }
    ph::feel_ground::feel_ground(&inputs.ground())?;
    ph::reset_when_stuck::reset_when_stuck(&inputs.reset(), scene)?;
    let view_turn = race.frame.view_kept.borrow().view_turn_matrix;
    ph::engine_sound::engine_sound(&race.course.numbers, &view_turn, &race.shared, sound)?;
    ph::spin_wheels::spin_wheels(race.shared.car(), &race.course.numbers);
    ph::track_checkpoints::track_checkpoints(&inputs.checkpoints(), scene, sound, o.audio)?;
    ph::pace_notes_step::pace_notes_step(&inputs.checkpoints(), sound, o.audio)?;
    ph::count_down_notice::count_down_notice(race.shared.car());
    ph::accumulate_mud::accumulate_mud(
        race.shared.car(),
        race.shared.vars.get(),
        race.kind_tables(),
    );
    x87_1(race, race.course.numbers.half);
    ph::store_motion_length::store_motion_length(race.shared.car());
    ph::damage_step::damage_step(
        race.shared.car(),
        &mut race.physics.damage_mut(),
        &race.shared.work(),
    )?;
    let view_turn = race.frame.view_kept.borrow().view_turn_matrix;
    ph::hit_part_checks::hit_part_checks(
        &race.course.numbers,
        &view_turn,
        &race.shared,
        race.kind_tables(),
        scene,
        sound,
    )?;
    ph::damage_notices::damage_notices(race.shared.car(), &mut race.physics.damage_mut())?;
    ph::take_notice::take_notice(race.shared.car());
    ph::drop_to_neutral::drop_to_neutral(race.shared.car(), &race.shared.physics_tables());
    // A step that set only flags nobody reads is left out.
    // The step's three callers run it for the cars from 0 up to the race's car count, at most 8.
    let car = race.shared.work().current_car as usize;
    let listed = *race
        .frame
        .frame_work
        .borrow()
        .listed_car_sounds
        .get(car)
        .ok_or(BadInput("a car number past the eight"))?;
    if listed != u32::MAX {
        ph::camera_frame::camera_frame(&inputs.camera(), scene)?;
    }
    ph::hold_spin_on_ground::hold_spin_on_ground(
        race.shared.car(),
        &mut race.shared.physics_tables_mut(),
    );
    ph::drop_broken_parts::drop_broken_parts(
        &race.course.numbers,
        &mut race.shared.work_mut(),
        race.shared.car(),
        scene,
    )?;
    ph::car_lighting::car_lighting(&inputs.lighting(), scene)?;
    ph::sum_forces::sum_forces(
        &race.course.numbers,
        &race.shared.physics_tables(),
        race.physics.wheel_work(),
    );
    ph::blend_wheel_load::blend_wheel_load(race.shared.car(), &race.course.numbers);
    ph::share_engine_sound::share_engine_sound(race.shared.car(), sound, &race.shared.work())?;
    ph::false_start::false_start(&inputs.reset(), scene, &o.net.race_slot)?;
    ph::value_notice::value_notice(race.shared.car());
    Ok(())
}

pub fn car_physics_step(o: crate::owners::Owners<'static>, race: &Race) -> Result<(), Refused> {
    if race.shared.work().current_car > 7 {
        return Err(BadInput("a current car past 7 (the per-car table's room)").into());
    }
    steps(o, race)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[mean_drive_speed] = ([drive_speed_a] + [drive_speed_b]) * [half]`
/// `[vertical_speed_change] = [car vertical velocity] - [vertical_speed]`
fn x87_1(race: &Race, half: crate::memory::session::ConstFloat) {
    let t0 = race.shared.car().drive_speed_a.ld();
    let t1 = race.shared.car().drive_speed_b.ld();
    let t2 = t0 + t1;
    let t3 = half.ld();
    let t4 = t2 * t3;
    race.shared.car().mean_drive_speed.st(t4);
    let t5 = race.shared.car().velocity.ld(1);
    let t6 = race.shared.physics_tables().vertical_speed.ld();
    let t7 = t5 - t6;
    race.shared.car().vertical_speed_change.st(t7);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
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
            plain: |_| {
                x87_1(
                    race(),
                    crate::test_support::race_places::number_constant(".half"),
                );
            },
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.car_physics_step", &blocks(), 1200, 100);
    }
}
