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
use crate::physics as ph;
use crate::physics::car_store::CAR;
use crate::physics::flow::{g, hold_high, hold_low, negate, p};
use crate::physics::tables_store::PHYSICS;
use crate::race::work_store::RACE_WORK;
use crate::scene::matrix3::rounded;
use crate::scene::x87_mem::{fld, fstp};

/// The motion: moved, spun, levered, the inverse matrix, the angles.
fn motion() {
    unsafe {
        ph::move_car::move_car();
        ph::spin_matrix::spin_matrix();
        ph::lever_pushes::lever_pushes();
        crate::race::start_place::invert_car_matrix();
        ph::update_car_angles::update_car_angles();
    }
}

/// The steps. What the callees leave in the registers is read by none of them (each reads
/// what it needs from memory), nor by the callers.
pub fn steps() -> Result<(), Refused> {
    unsafe {
        ph::stir_car_word::stir_car_word();
        ph::load_car_specs::load_car_specs()?;
        ph::make_gear_ratios::make_gear_ratios()?;
        ph::store_motion_length::store_motion_length();
        ph::set_handling::set_handling()?;
        ph::race_timers::race_timers()?;
    }
    for x in [
        0x6D1AD0, 0x6D1AD4, 0x6D1AD8, 0x6D30E4, 0x608C3C, 0x608C40, 0x608C44,
    ] {
        p(x, 0);
    }
    PHYSICS
        .vertical_speed
        .set_bits(loaded(CAR.velocity[1].bits()));
    for cell in [
        &PHYSICS.clear_place_mark,
        &CAR.contact_flag,
        &PHYSICS.scene_point_hits,
        &PHYSICS.object_hits,
        &PHYSICS.point_hits,
    ] {
        cell.set(0);
    }
    for figure in CAR.contact_point.iter().chain([&CAR.contact_depth]) {
        figure.set_bits(0);
    }
    unsafe {
        crate::race::start_place::invert_car_matrix();
        ph::feel_ground::feel_ground()?;
        ph::start_skid_mark::start_skid_mark();
        ph::probe_and_keep::probe_and_keep()?;
        ph::ease_car_value::ease_car_value();
        ph::count_down_timers::count_down_timers();
        ph::wheel_surfaces::wheel_surfaces()?;
        ph::surface_noise::surface_noise();
        ph::save_car_state::save_car_state()?;
        ph::clear_forces::clear_forces()?;
        ph::set_wheel_weights::set_wheel_weights();
        ph::set_downforce::set_downforce();
    }
    ph::gearbox_step::gearbox_step()?;
    unsafe { ph::drive_force::drive_force() }?;
    // 0x43B737: the side speed held within the limit.
    CAR.side_speed_held.set_bits(CAR.local_velocity[0].bits());
    hold_high(&CAR.side_speed_held, &PHYSICS.side_speed_limit);
    negate(&PHYSICS.side_speed_limit);
    hold_low(&CAR.side_speed_held, &PHYSICS.side_speed_limit);
    negate(&PHYSICS.side_speed_limit);
    unsafe {
        ph::drive_grip_forces::drive_grip_forces();
        ph::drive_wheels_step::drive_wheels_step()?;
        ph::settle_drive_speeds::settle_drive_speeds()?;
        ph::free_rev_in_neutral::free_rev_in_neutral();
        ph::free_rev_clutch::free_rev_clutch()?;
    }
    // 0x43B7BC: the engine's speed held.
    if CAR.brake.get() == 0 {
        hold_low(&CAR.engine_speed, &PHYSICS.idle_engine_speed);
    }
    hold_high(&CAR.engine_speed, &PHYSICS.rev_limit);
    unsafe {
        ph::wheels_from_revs::wheels_from_revs()?;
        ph::air_drag::air_drag();
        ph::wheel_springs::wheel_springs();
    }
    ph::crash_contact::crash_contact()?;
    unsafe {
        ph::apply_wheel_forces::apply_wheel_forces()?;
        ph::apply_wheel_torques::apply_wheel_torques()?;
    }
    motion();
    PHYSICS.top_speed.set_bits(rounded(unsafe {
        ph::vector_length::vector_length(CAR.velocity[0].addr() as u32)
    }));
    // 0x43B851: the contacts' rounds.
    PHYSICS.contact_step.set(0);
    loop {
        unsafe {
            ph::clear_contacts::clear_contacts()?;
            ph::feel_ground::feel_ground()?;
        }
        ph::crossed_edge::crossed_edge()?;
        ph::wall_contacts::wall_contacts()?;
        ph::ground_contact::ground_contact()?;
        unsafe { ph::car_collisions::car_collisions() }?;
        ph::face_contacts::face_contacts()?;
        unsafe {
            ph::keep_motion::keep_motion();
            ph::apply_contact_forces::apply_contact_forces()?;
            ph::apply_contact_torques::apply_contact_torques()?;
        }
        if PHYSICS.clear_place_count.get() == 0 {
            break;
        }
        unsafe {
            ph::stall_when_still::stall_when_still();
            ph::keep_motion_vectors::keep_motion_vectors();
        }
        motion();
        PHYSICS.clear_place_count.set(0);
        PHYSICS.contact_step.set(
            crate::physics::tables_store::PHYSICS
                .contact_step
                .get()
                .wrapping_add(1),
        );
        if PHYSICS.contact_step.get() >= PHYSICS.contact_steps.get() {
            unsafe {
                ph::limit_motion::limit_motion();
                ph::find_clear_place::find_clear_place()?;
            }
            PHYSICS.step_count.set(
                crate::physics::tables_store::PHYSICS
                    .step_count
                    .get()
                    .wrapping_add(1),
            );
            break;
        }
    }
    if PHYSICS.contact_step.get() != 0 {
        unsafe {
            ph::put_back_motion_vectors::put_back_motion_vectors();
            ph::limit_motion::limit_motion();
        }
    }
    unsafe {
        ph::feel_ground::feel_ground()?;
        ph::reset_when_stuck::reset_when_stuck()?;
        ph::engine_sound::engine_sound()?;
        ph::spin_wheels::spin_wheels();
    }
    ph::track_checkpoints::track_checkpoints()?;
    ph::pace_notes_step::pace_notes_step()?;
    unsafe {
        ph::count_down_notice::count_down_notice();
        ph::turn_wheel_angles::turn_wheel_angles();
    }
    x87_1(crate::race::store::asm_base() as usize);
    unsafe { ph::store_motion_length::store_motion_length() };
    ph::damage_step::damage_step();
    ph::hit_part_checks::hit_part_checks()?;
    ph::damage_notices::damage_notices()?;
    unsafe {
        ph::take_notice::take_notice();
        ph::drop_to_neutral::drop_to_neutral();
    }
    // 0x43B96B: `physics::test_car_word_234` set only flags nobody reads: left out.
    if g(0x641680 + 4 * RACE_WORK.current_car.get() as usize) != u32::MAX {
        ph::camera_frame::camera_frame()?;
    }
    unsafe { ph::hold_spin_on_ground::hold_spin_on_ground() };
    ph::drop_broken_parts::drop_broken_parts()?;
    ph::car_lighting::car_lighting()?;
    unsafe {
        ph::sum_forces::sum_forces();
        ph::blend_wheel_load::blend_wheel_load();
        ph::share_engine_sound::share_engine_sound()?;
        ph::note_time_mark::note_time_mark();
        ph::false_start::false_start()?;
        ph::value_notice::value_notice();
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn car_physics_step() -> Result<(), Refused> {
    if RACE_WORK.current_car.get() > 7 {
        return Err(BadInput("a current car past 7 (dword_641680's room)").into());
    }
    steps()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A198] = ([rc + 0x31A190] + [rc + 0x31A194]) * [rc + 0x1C5E40]`
/// `[rc + 0x31A8E4] = [rc + 0x31A174] - [rc + 0x214138]`
fn x87_1(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x31A190);
        let t1 = fld(rc, 0x31A194);
        let t2 = t0 + t1;
        let t3 = fld(rc, 0x1C5E40);
        let t4 = t2 * t3;
        fstp(rc, 0x31A198, t4);
        let t5 = fld(rc, 0x31A174);
        let t6 = fld(rc, 0x214138);
        let t7 = t5 - t6;
        fstp(rc, 0x31A8E4, t7);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x31A190]", "fadd dword ptr [{rc} + 0x31A194]", "fmul dword ptr [{rc} + 0x1C5E40]", "fstp dword ptr [{rc} + 0x31A198]", "fld dword ptr [{rc} + 0x31A174]", "fsub dword ptr [{rc} + 0x214138]", "fstp dword ptr [{rc} + 0x31A8E4]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "car_physics_step line 183",
            operands: vec![Operand::Buffer {
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
            plain: |a| x87_1(a[0]),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
