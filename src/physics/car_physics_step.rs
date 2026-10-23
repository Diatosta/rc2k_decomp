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
//! The flow and the calls are Rust (each callee's Rust version with the registers its
//! hook would leave); the holds are `physics/flow.rs`'s.
//!
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read (the callers
//!      store the car next)
//!
//! Callers: `physics::race_cars_step` (0x4608BB), `sub_460835`, `sub_460B2F`. It begins
//! with a call: hooked by a private copy of the whole function (0x37F bytes, one chunk)
//! with its 83 calls relocated, and a hand-made patch.
//! Refuses (before anything) a current car past 7 (`dword_641680` is read by it), and
//! what its calls refuse; the work car and the physics' variables (`step_regions`) are
//! put back then (the callees put back their own other writes). Shadow mode runs the
//! original alone (a whole step: its parts are compared by their own hooks).
//! Self-test (in a race; skipped otherwise): the original and the Rust version from the
//! same state (everything the step writes saved and put back), the car where it is and
//! moved into the ground, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, Regs, game_addr, rva};
use crate::math::x87::loaded;
use crate::physics as ph;
use crate::physics::flow::{a, base, g, hold_high, hold_low, negate, p, x87_run};
use crate::physics::reset_parts::game_part;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::rounded;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore};

/// A callee that changes the registers it is given, of which its hook reports `regs`.
fn keep(r: &mut Regs, regs: &[u8], f: unsafe fn(&mut Regs)) {
    let mut g = *r;
    unsafe { f(&mut g) };
    let mut left = Left::nothing();
    for &reg in regs {
        left = left.with(reg, *field(&g, reg));
    }
    left.apply(r);
}

fn field(g: &Regs, reg: u8) -> &u32 {
    match reg {
        EAX => &g.eax,
        EBX => &g.ebx,
        ECX => &g.ecx,
        EDX => &g.edx,
        ESI => &g.esi,
        _ => &g.edi,
    }
}

/// The motion: moved, spun, levered, the inverse matrix, the angles.
fn motion(r: &mut Regs) {
    unsafe {
        r.eax = ph::move_car::move_car();
        ph::spin_matrix::spin_matrix().apply(r);
        ph::lever_pushes::lever_pushes().apply(r);
        crate::race::start_place::invert_car_matrix();
        ph::update_car_angles::update_car_angles().apply(r);
    }
}

/// The steps; `r` as the original leaves it.
pub fn steps(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        r.eax = ph::stir_car_word::stir_car_word();
        ph::load_car_specs::load_car_specs()?.apply(r);
        r.ebx = ph::make_gear_ratios::make_gear_ratios()?;
        ph::store_motion_length::store_motion_length();
        ph::set_handling::set_handling()?.apply(r);
        ph::race_timers::race_timers(r)?.apply(r);
    }
    r.ebx = 0;
    for x in [
        0x6D1AD0, 0x6D1AD4, 0x6D1AD8, 0x6D30E4, 0x608C3C, 0x608C40, 0x608C44,
    ] {
        p(x, 0);
    }
    p(0x614138, loaded(g(0x71A174)));
    for x in [
        0x614C28, 0x71A1FC, 0x71A200, 0x71A204, 0x71A208, 0x71A20C, 0x614108, 0x61410C, 0x614110,
    ] {
        p(x, 0);
    }
    unsafe {
        crate::race::start_place::invert_car_matrix();
        ph::feel_ground::feel_ground(r)?;
    }
    keep(r, &[EAX], ph::start_skid_mark::start_skid_mark);
    {
        let mut g = *r;
        unsafe { ph::probe_and_keep::probe_and_keep(&mut g) }?;
        Left::nothing()
            .with(EAX, g.eax)
            .with(EBX, g.ebx)
            .with(ECX, g.ecx)
            .with(EDX, g.edx)
            .with(ESI, g.esi)
            .with(EDI, g.edi)
            .apply(r);
    }
    keep(r, &[EAX], ph::ease_car_value::ease_car_value);
    keep(r, &[EAX], ph::count_down_timers::count_down_timers);
    unsafe {
        ph::wheel_surfaces::wheel_surfaces()?.apply(r);
        ph::surface_noise::surface_noise(r).apply(r);
        ph::save_car_state::save_car_state()?.apply(r);
        ph::clear_forces::clear_forces()?.apply(r);
    }
    keep(r, &[ESI], ph::set_wheel_weights::set_wheel_weights);
    unsafe {
        ph::set_downforce::set_downforce();
        ph::gearbox_step::gearbox_step(r)?.apply(r);
        ph::drive_force::drive_force()?.apply(r);
    }
    // 0x43B737: the side speed held within the limit.
    p(0x71A1A0, g(0x71A17C));
    hold_high(0x71A1A0, 0x613EC8);
    negate(0x613EC8);
    hold_low(0x71A1A0, 0x613EC8);
    negate(0x613EC8);
    unsafe {
        let [eax, ebx, edx] = ph::drive_grip_forces::drive_grip_forces(r.ebx);
        r.eax = eax;
        r.ebx = ebx;
        r.edx = edx;
        let [eax, ebx] = ph::drive_wheels_step::drive_wheels_step(r.eax, r.ebx)?;
        r.eax = eax;
        r.ebx = ebx;
        ph::settle_drive_speeds::settle_drive_speeds()?.apply(r);
        ph::free_rev_in_neutral::free_rev_in_neutral().apply(r);
        ph::free_rev_clutch::free_rev_clutch(r)?.apply(r);
    }
    // 0x43B7BC: the engine's speed held.
    if g(0x71A614) == 0 {
        hold_low(0x71A1B0, 0x6140F8);
    }
    hold_high(0x71A1B0, 0x614104);
    unsafe {
        ph::wheels_from_revs::wheels_from_revs()?.apply(r);
        ph::air_drag::air_drag();
        ph::wheel_springs::wheel_springs().apply(r);
        ph::crash_contact::crash_contact(r)?.apply(r);
        ph::apply_wheel_forces::apply_wheel_forces()?.apply(r);
        ph::apply_wheel_torques::apply_wheel_torques()?.apply(r);
    }
    motion(r);
    r.esi = a(0x71A170);
    p(
        0x614100,
        rounded(unsafe { ph::vector_length::vector_length(r.esi) }),
    );
    // 0x43B851: the contacts' rounds.
    p(0x614148, 0);
    loop {
        unsafe {
            ph::clear_contacts::clear_contacts()?.apply(r);
            ph::feel_ground::feel_ground(r)?;
            ph::crossed_edge::crossed_edge(r)?.apply(r);
            ph::wall_contacts::wall_contacts(r)?.apply(r);
            ph::ground_contact::ground_contact(r)?.apply(r);
            ph::car_collisions::car_collisions(r)?.apply(r);
            ph::face_contacts::face_contacts(r)?.apply(r);
            ph::keep_motion::keep_motion();
            ph::apply_contact_forces::apply_contact_forces()?.apply(r);
            ph::apply_contact_torques::apply_contact_torques()?.apply(r);
        }
        if g(0x614738) == 0 {
            break;
        }
        keep(r, &[ESI, EDI], ph::stall_when_still::stall_when_still);
        unsafe { ph::keep_motion_vectors::keep_motion_vectors() };
        motion(r);
        p(0x614738, 0);
        p(0x614148, g(0x614148).wrapping_add(1));
        r.eax = g(0x614148);
        if r.eax >= g(0x61414C) {
            unsafe {
                ph::limit_motion::limit_motion().apply(r);
                ph::find_clear_place::find_clear_place(r)?.apply(r);
            }
            p(0x6141D0, g(0x6141D0).wrapping_add(1));
            break;
        }
    }
    if g(0x614148) != 0 {
        unsafe {
            ph::put_back_motion_vectors::put_back_motion_vectors();
            ph::limit_motion::limit_motion().apply(r);
        }
    }
    unsafe {
        ph::feel_ground::feel_ground(r)?;
        *r = ph::reset_when_stuck::reset_when_stuck(r, &mut game_part)?;
        ph::engine_sound::engine_sound()?.apply(r);
        r.eax = ph::spin_wheels::spin_wheels();
        ph::track_checkpoints::track_checkpoints_in(r, &mut ph::track_checkpoints::game_part)?;
        ph::pace_notes_step::pace_notes_step_in(r, &mut ph::queue_pace_note::pace_part)?;
    }
    keep(r, &[EAX], ph::count_down_notice::count_down_notice);
    unsafe { ph::turn_wheel_angles::turn_wheel_angles() }.apply(r);
    x87_run!(
        "fld dword ptr [{b} + 0x31A190]",
        "fadd dword ptr [{b} + 0x31A194]",
        "fmul dword ptr [{b} + 0x1C5E40]",
        "fstp dword ptr [{b} + 0x31A198]",
        "fld dword ptr [{b} + 0x31A174]",
        "fsub dword ptr [{b} + 0x214138]",
        "fstp dword ptr [{b} + 0x31A8E4]";
        b = base()
    );
    unsafe {
        ph::store_motion_length::store_motion_length();
        ph::damage_step::damage_step(r).apply(r);
        ph::hit_part_checks::hit_part_checks(r)?.apply(r);
        ph::damage_notices::damage_notices(r)?.apply(r);
        ph::take_notice::take_notice().apply(r);
        ph::drop_to_neutral::drop_to_neutral();
        ph::test_car_word_234::test_car_word_234(r.eax).apply(r);
    }
    r.eax = g(0x71B3F8);
    r.eax = g(0x641680 + 4 * r.eax as usize);
    if r.eax != u32::MAX {
        *r = unsafe { ph::camera_frame::camera_frame(r, &mut game_part) }?;
    }
    keep(r, &[ECX, EBX], ph::hold_spin_on_ground::hold_spin_on_ground);
    unsafe {
        ph::drop_broken_parts::drop_broken_parts(r)?.apply(r);
        *r = ph::car_lighting::car_lighting(r, &mut game_part)?;
        ph::sum_forces::sum_forces();
    }
    keep(r, &[EDX], ph::blend_wheel_load::blend_wheel_load);
    unsafe { ph::share_engine_sound::share_engine_sound() }?.apply(r);
    keep(r, &[EAX], ph::note_time_mark::note_time_mark);
    unsafe {
        *r = ph::false_start::false_start(r, &mut game_part)?;
        ph::value_notice::value_notice();
    }
    Ok(())
}

/// The work car and the physics' variables (put back on a refusal; compared by the
/// self-test with the callees' own regions).
pub fn step_regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x719FD8), 0xFE0),
        (a(0x613800), 0x1800), // 0x613800 .. 0x615000
        (a(0x620400), 0x3E00), // 0x620400 .. 0x624200
        (a(0x6D1A00), 0x2300), // 0x6D1A00 .. 0x6D3D00
        (a(0x608B00), 0x200),
        (a(0x6396F0), 0x1B0),
        (a(0x5C5F40), 0x30),
        (a(0x5EA000), 0x500),
        (a(0x75B340), 0x600),
    ]
}

/// The Rust version.
pub unsafe fn car_physics_step(entry: &Regs) -> Result<Left, Refused> {
    if g(0x71B3F8) > 7 {
        return Err(BadInput("a current car past 7 (dword_641680's room)").into());
    }
    let all = step_regions();
    let saved = capture(&all);
    let mut r = *entry;
    match steps(&mut r) {
        Ok(()) => Ok(left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { car_physics_step(r) }?;
    left.apply(r);
    Ok(())
}

/// Everything the step and its calls write, for the tests.
pub fn test_regions() -> Vec<Region> {
    let mut all = step_regions();
    all.extend(ph::find_clear_place::regions());
    all.extend(ph::update_camera::camera_regions());
    all.extend(ph::damage_model::regions());
    all.push(ph::face_contact::faces_region());
    all
}
