//! `physics::car_physics_step` (0x43B635): one physics step of the work car (the copy at
//! `g_car`, loaded by `drive::load_car`). In order: the car's word stirred, its specs, gear
//! ratios, motion length, handling, the race timers; the step's notes cleared (sounds
//! `dword_6D1AD0` .., the camera's probe counts, contacts `unk_71A1FC` .. `71A20C`, the
//! hit counters `dword_614108` ..), the height before (`flt_614138`); the inverse matrix,
//! the ground, skid marks, the probes, the timers, the surfaces and their noise, the state
//! saved, the forces cleared, wheel weights, downforce, the gearbox, the drive force; the
//! side speed into `unk_71A1A0` held within ± `flt_613EC8`; the driven wheels (grip, step,
//! speeds, the free revs); the engine's speed held between `flt_6140F8` (only off the
//! clutch, `unk_71A614` 0) and `flt_614104`; wheels from revs, air drag, springs, the body
//! against the ground, the wheels' forces and torques, the motion (move, spin, levers,
//! matrix, angles); the speed's length into `flt_614100`. Then the contacts, up to
//! `dword_61414C` rounds (`dword_614148` the count): contacts cleared, the ground, the
//! crossed edge, the walls, the probes, the other cars, the faces, the motion kept, the
//! contacts' forces and torques; a round that restored nothing (`dword_614738` 0) ends
//! it, else the car stalled when still, its motion redone and the round counted; all
//! rounds used: `physics::limit_motion`, `physics::find_clear_place`, `dword_6141D0` + 1.
//! With rounds done, the motion put back and limited. Then the ground, the stuck reset,
//! the engine sound, the wheels' spin, the checkpoints, the pace notes
//! (`physics::pace_notes_step`), the count down, the wheel angles, the mean drive
//! speed (`unk_71A198`) and the height change (`unk_71A8E4`), the motion length, the
//! damage model and its parts and notices, the notices, neutral, the word +0x234, the
//! camera (`physics::camera_frame`, when the current car is listed in `dword_641680`), the
//! spin on the ground, broken parts, lights, the forces' sum, the wheel load, the engine
//! sound's share, the time mark, a false start, the value notice.
//!
//! the holds are `physics/flow.rs`'s.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read (the callers
//!      store the car next)
//!
//! Callers: `physics::race_cars_step` (0x4608BB), `sub_460835`, `sub_460B2F`. Refuses
//! (before anything) a current car past 7 (`dword_641680` is read by it), and what its
//! calls refuse; the work car and the physics' variables (`step_regions`) are put back then
//! (the callees put back their own other writes).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87::loaded;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics as ph;
use crate::physics::flow::{hold_high, hold_low, negate};
use crate::race::owner::Race;
use crate::scene::matrix3::rounded;

/// The motion: moved, spun, levered, the inverse matrix, the angles.
fn motion(race: &Race, scene: &crate::scene::owner::Scene) {
    ph::move_car::move_car(race);
    ph::spin_matrix::spin_matrix(race);
    ph::lever_pushes::lever_pushes(race, scene);
    crate::race::start_place::invert_car_matrix(race);
    ph::update_car_angles::update_car_angles(race, scene);
}

/// The steps. What the callees leave in the registers is read by none of them (each reads
/// what it needs from memory), nor by the callers.
pub fn steps(race: &Race) -> Result<(), Refused> {
    let system = crate::system::owner::system();
    let sound = crate::sound::owner::sound();
    let scene = crate::scene::owner::scene();
    let race_owner = race;
    ph::stir_car_word::stir_car_word(&race.shared);
    ph::load_car_specs::load_car_specs(race, scene)?;
    ph::make_gear_ratios::make_gear_ratios(race)?;
    ph::store_motion_length::store_motion_length(race);
    ph::set_handling::set_handling(race)?;
    ph::race_timers::race_timers(race, sound)?;
    for cell in [
        &race_owner.frame.sound_spots.sound_scratch_a,
        &race_owner.frame.sound_spots.sound_scratch_b,
        &race_owner.frame.sound_spots.sound_scratch_c,
    ] {
        cell.set(0);
    }
    race_owner.physics.contact.contact_normal_speed[0].set_bits(0);
    for cell in [
        &race_owner.physics.camera.ground_probes,
        &race_owner.physics.camera.ground_found,
        &race_owner.physics.camera.ground_same,
    ] {
        cell.set(0);
    }
    race.shared
        .physics_tables
        .vertical_speed
        .set_bits(loaded(race.shared.car.velocity[1].bits()));
    for cell in [
        &race.shared.physics_tables.clear_place_mark,
        &race.shared.car.contact_flag,
        &race.shared.physics_tables.scene_point_hits,
        &race.shared.physics_tables.object_hits,
        &race.shared.physics_tables.point_hits,
    ] {
        cell.set(0);
    }
    for figure in race
        .shared
        .car
        .contact_point
        .iter()
        .chain([&race.shared.car.contact_depth])
    {
        figure.set_bits(0);
    }
    crate::race::start_place::invert_car_matrix(race);
    ph::feel_ground::feel_ground(race)?;
    ph::start_skid_mark::start_skid_mark(&race.shared);
    ph::probe_and_keep::probe_and_keep(race)?;
    ph::ease_car_value::ease_car_value(&race.shared);
    ph::count_down_timers::count_down_timers(&race.shared);
    ph::wheel_surfaces::wheel_surfaces(race)?;
    ph::surface_noise::surface_noise(race, scene)?;
    ph::save_car_state::save_car_state(race)?;
    ph::clear_forces::clear_forces(&race.shared)?;
    ph::set_wheel_weights::set_wheel_weights(race, scene);
    ph::set_downforce::set_downforce(race);
    ph::gearbox_step::gearbox_step(race, scene)?;
    { ph::drive_force::drive_force(race) }?;
    // 0x43B737: the side speed held within the limit.
    race.shared
        .car
        .side_speed_held
        .set_bits(race.shared.car.local_velocity[0].bits());
    hold_high(
        &race.shared.car.side_speed_held,
        &race.shared.physics_tables.side_speed_limit,
    );
    negate(&race.shared.physics_tables.side_speed_limit);
    hold_low(
        &race.shared.car.side_speed_held,
        &race.shared.physics_tables.side_speed_limit,
    );
    negate(&race.shared.physics_tables.side_speed_limit);
    ph::drive_grip_forces::drive_grip_forces(race, scene);
    ph::drive_wheels_step::drive_wheels_step(race)?;
    ph::settle_drive_speeds::settle_drive_speeds(race)?;
    ph::free_rev_in_neutral::free_rev_in_neutral(race);
    ph::free_rev_clutch::free_rev_clutch(race)?;
    // 0x43B7BC: the engine's speed held.
    if race.shared.car.brake.get() == 0 {
        hold_low(
            &race.shared.car.engine_speed,
            &race.shared.physics_tables.idle_engine_speed,
        );
    }
    hold_high(
        &race.shared.car.engine_speed,
        &race.shared.physics_tables.rev_limit,
    );
    ph::wheels_from_revs::wheels_from_revs(&race.shared)?;
    ph::air_drag::air_drag(race);
    ph::wheel_springs::wheel_springs(race, scene);
    ph::crash_contact::crash_contact(race)?;
    ph::apply_wheel_forces::apply_wheel_forces(&race.shared)?;
    ph::apply_wheel_torques::apply_wheel_torques(race)?;
    motion(race, scene);
    race.shared
        .physics_tables
        .top_speed
        .set_bits(rounded(ph::vector_length::vector_length(
            &race.shared.car.velocity,
        )));
    // 0x43B851: the contacts' rounds.
    race.shared.physics_tables.contact_step.set(0);
    loop {
        ph::clear_contacts::clear_contacts(&race.shared)?;
        ph::feel_ground::feel_ground(race)?;

        ph::crossed_edge::crossed_edge(race)?;
        ph::wall_contacts::wall_contacts(race)?;
        ph::ground_contact::ground_contact(race)?;
        { ph::car_collisions::car_collisions(race) }?;
        ph::face_contacts::face_contacts(race)?;
        ph::keep_motion::keep_motion(&race.shared);
        ph::apply_contact_forces::apply_contact_forces(&race.shared)?;
        ph::apply_contact_torques::apply_contact_torques(race)?;

        if race.shared.physics_tables.clear_place_count.get() == 0 {
            break;
        }
        ph::stall_when_still::stall_when_still(race);
        ph::keep_motion_vectors::keep_motion_vectors(race);

        motion(race, scene);
        race.shared.physics_tables.clear_place_count.set(0);
        race.shared.physics_tables.contact_step.set(
            race.shared
                .physics_tables
                .contact_step
                .get()
                .wrapping_add(1),
        );
        if race.shared.physics_tables.contact_step.get()
            >= race.shared.physics_tables.contact_steps.get()
        {
            ph::limit_motion::limit_motion(race, scene);
            ph::find_clear_place::find_clear_place(race)?;

            race.shared
                .physics_tables
                .step_count
                .set(race.shared.physics_tables.step_count.get().wrapping_add(1));
            break;
        }
    }
    if race.shared.physics_tables.contact_step.get() != 0 {
        ph::put_back_motion_vectors::put_back_motion_vectors(race);
        ph::limit_motion::limit_motion(race, scene);
    }
    ph::feel_ground::feel_ground(race)?;
    ph::reset_when_stuck::reset_when_stuck(race)?;
    ph::engine_sound::engine_sound(race, scene)?;
    ph::spin_wheels::spin_wheels(race);
    ph::track_checkpoints::track_checkpoints(race, scene, sound)?;
    ph::pace_notes_step::pace_notes_step(race, scene, sound)?;
    ph::count_down_notice::count_down_notice(&race.shared);
    ph::turn_wheel_angles::turn_wheel_angles(race, scene);
    x87_1(race);
    ph::store_motion_length::store_motion_length(race);
    ph::damage_step::damage_step(race);
    ph::hit_part_checks::hit_part_checks(race, scene)?;
    ph::damage_notices::damage_notices(race)?;
    ph::take_notice::take_notice(&race.shared);
    ph::drop_to_neutral::drop_to_neutral(&race.shared);
    // 0x43B96B: `physics::test_car_word_234` set only flags nobody reads: left out.
    // A car number past 7 (refused further down) reads past the field.
    let car = race.shared.work.current_car.get() as usize;
    let listed = crate::session::read_on_in(
        &race_owner.frame.frame_work,
        &race_owner.frame.frame_work.listed_car_sounds,
        car,
        "a listed car sound",
    )?;
    if listed != u32::MAX {
        ph::camera_frame::camera_frame(race)?;
    }
    ph::hold_spin_on_ground::hold_spin_on_ground(race);
    ph::drop_broken_parts::drop_broken_parts(race, scene)?;
    ph::car_lighting::car_lighting(race, scene, system)?;
    ph::sum_forces::sum_forces(race);
    ph::blend_wheel_load::blend_wheel_load(race);
    ph::share_engine_sound::share_engine_sound(race)?;
    ph::note_time_mark::note_time_mark(race);
    ph::false_start::false_start(race)?;
    ph::value_notice::value_notice(&race.shared);
    Ok(())
}

/// The Rust version.
pub fn car_physics_step(race: &Race) -> Result<(), Refused> {
    if race.shared.work.current_car.get() > 7 {
        return Err(BadInput("a current car past 7 (dword_641680's room)").into());
    }
    steps(race)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A198] = ([rc + 0x31A190] + [rc + 0x31A194]) * [rc + 0x1C5E40]`
/// `[rc + 0x31A8E4] = [rc + 0x31A174] - [rc + 0x214138]`
fn x87_1(race: &Race) {
    let t0 = ld(race.shared.car.drive_speed_a.bits());
    let t1 = ld(race.shared.car.drive_speed_b.bits());
    let t2 = t0 + t1;
    let t3 = ld(race.course.numbers.half.bits());
    let t4 = t2 * t3;
    race.shared.car.mean_drive_speed.set_bits(st(t4));
    let t5 = ld(race.shared.car.velocity[1].bits());
    let t6 = ld(race.shared.physics_tables.vertical_speed.bits());
    let t7 = t5 - t6;
    race.shared.car.unknown_90c.set_bits(st(t7));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x31A190]", "fadd dword ptr [{rc} + 0x31A194]", "fmul dword ptr [{rc} + 0x1C5E40]", "fstp dword ptr [{rc} + 0x31A198]", "fld dword ptr [{rc} + 0x31A174]", "fsub dword ptr [{rc} + 0x214138]", "fstp dword ptr [{rc} + 0x31A8E4]");

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
            reference: |a| unsafe { ref_0(a[0]) },
            plain: |_| x87_1(race()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
