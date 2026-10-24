//! Race init's first step, the cars' starts (`race::start_cars`, 0x446463, IDA
//! 2026-10-17): each car of the race loaded into the work copy (`g_car`), set at its start
//! (`race::start_car`: its checkpoint, its place on the line and to the side, the nearest
//! checkpoint, then `race::reset_car_motion`: settled on the ground by
//! `race::settle_car`, its matrix undone, its motion cleared), two blocks cleared, stored
//! back. `sub_43EB5D` and `sub_45D305` are stream B's `physics::set_handling` and
//! `physics::probe_ground` (since 2026-10-17).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::race::best_line::{CHECKS_RVA, MOST_CHECKS};

const fn car(offset: usize) -> usize {
    rva(0x719FD8 + offset)
}

pub const CURRENT_CAR_RVA: usize = crate::drive::car::CURRENT_CAR_RVA; // g_currentCar
pub const RACE_CARS_RVA: usize = crate::race::objects::CARS_RVA; // g_raceCars
const CAR_Y_RVA: usize = car(0x138); // dword_71A110
const PITCH_RVA: usize = car(0x9C4); // dword_71A99C
const ROLL_RVA: usize = car(0x9C8); // dword_71A9A0
const HEADING_RVA: usize = car(0x9C0); // dword_71A998
const PITCH_STEP_RVA: usize = rva(0x61426C); // dword_61426C
const ROLL_STEP_RVA: usize = rva(0x614270); // dword_614270
const SETTLE_MARK_RVA: usize = rva(0x620470); // dword_620470
/// dword_6D2514 .. dword_6D2520: the four wheels' heights (made by `sub_45D305`).
pub const WHEELS_RVA: usize = rva(0x6D2514);
/// dword_6D1F14 .. dword_6D1F20: four more lowered with them.
pub const WHEELS_2_RVA: usize = rva(0x6D1F14);
/// dword_608B50, dword_608B54, dword_608B58: the start's heading, roll and pitch.
const START_ANGLES_RVA: usize = rva(0x608B50);
const MODE_RVA: usize = rva(0x62389C); // dword_62389C
const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18
const SIDE_RVA: usize = rva(0x52A910); // dword_52A910
const CARS_2_RVA: usize = rva(0x63DB94); // dword_63DB94
const PLACE_HEADING_RVA: usize = crate::race::start_place::PLACE_HEADING_RVA; // dword_5CCD60
const LINE_PLACE_RVA: usize = crate::race::start_place::LINE_PLACE_RVA; // dword_6365E4
const LINE_FRACTION_RVA: usize = crate::race::start_place::LINE_FRACTION_RVA; // dword_6365EC
const CAR_X_RVA: usize = crate::race::start_place::CAR_X_RVA; // dword_71A10C
const CAR_Z_RVA: usize = crate::race::start_place::CAR_Z_RVA; // dword_71A114
const PENDING_RVA: usize = car(0x68C); // dword_71A664
/// The most cars `g_carBlocks` has.
pub const MOST_CARS: u32 = 8;

unsafe fn add(at: usize, by: u32) {
    unsafe { game_write(at, game_read::<u32>(at).wrapping_add(by)) };
}

unsafe fn zero(at: usize, dwords: usize) {
    for k in 0..dwords {
        unsafe { game_write(at + 4 * k, 0u32) };
    }
}

unsafe fn wheels() -> [u32; 4] {
    unsafe { game_read(WHEELS_RVA) }
}

/// One bisection step of `settle_car`: `angle` moved by `step` towards the higher pair
/// (down when `low` - `high` is negative), the step halved (`shr`).
unsafe fn settle_step(low: u32, high: u32, angle: usize, step: usize) {
    unsafe {
        let s: u32 = game_read(step);
        if (low.wrapping_sub(high) as i32) < 0 {
            game_write(angle, game_read::<u32>(angle).wrapping_sub(s));
        } else {
            add(angle, s);
        }
        game_write(step, s >> 1);
    }
}

/// The matrix (`get_mat_crash`) and the wheels (`physics::probe_ground`).
unsafe fn matrix_and_wheels() -> Result<[u32; 4], Refused> {
    unsafe {
        crate::race::stage_prefs::get_mat_crash();
        crate::physics::probe_ground::probe_ground()?;
        Ok(wheels())
    }
}

/// `race::settle_car` (0x43FC32): `sub_43EB5D` (`physics::set_handling`); the work car's
/// pitch and roll (+0x9C4, +0x9C8) 0, both steps 0x20000000, `dword_620470` 0x80; eight
/// times the matrix (`get_mat_crash`) and the wheels (`sub_45D305`), the roll moved by the
/// step towards wheels 0 + 2 against 1 + 3, the pitch towards 0 + 1 against 2 + 3, each
/// step halved; once more the matrix and the wheels, then the car's y, the four wheels and
/// the four at 0x6D1F14 lowered by the lowest wheel (from 0x640000). The registers it
/// leaves are read by no caller (conversion phase 3, 2026-10-24).
pub unsafe fn settle_car() -> Result<(), Refused> {
    unsafe {
        crate::physics::set_handling::set_handling()?;
        game_write(PITCH_RVA, 0u32);
        game_write(ROLL_RVA, 0u32);
        game_write(PITCH_STEP_RVA, 0x2000_0000u32);
        game_write(ROLL_STEP_RVA, 0x2000_0000u32);
        game_write(SETTLE_MARK_RVA, 0x80u32);
        for _ in 0..8 {
            let w = matrix_and_wheels()?;
            settle_step(
                w[0].wrapping_add(w[2]),
                w[1].wrapping_add(w[3]),
                ROLL_RVA,
                ROLL_STEP_RVA,
            );
            settle_step(
                w[0].wrapping_add(w[1]),
                w[2].wrapping_add(w[3]),
                PITCH_RVA,
                PITCH_STEP_RVA,
            );
        }
        let w = matrix_and_wheels()?;
        let mut lowest = 0x64_0000u32;
        for &h in &w {
            if (h.wrapping_sub(lowest) as i32) < 0 {
                lowest = h;
            }
        }
        game_write(CAR_Y_RVA, game_read::<u32>(CAR_Y_RVA).wrapping_sub(lowest));
        for (k, h) in w.iter().enumerate() {
            game_write(WHEELS_RVA + 4 * k, h.wrapping_sub(lowest));
        }
        for k in 0..4 {
            let at = WHEELS_2_RVA + 4 * k;
            game_write(at, game_read::<u32>(at).wrapping_sub(lowest));
        }
    }
    Ok(())
}

/// `race::reset_car_motion` (0x43FB47): `settle_car`, `invert_car_matrix`; the work car's
/// +0x198 .. +0x1A0 0, +0x1A4 1.0, +0x1A8 0, `g_carSpeed` (+0x1AC) 0, +0x184 .. +0x18C 0;
/// heading, pitch and roll copied to `dword_608B50` / `58` / `54` and +0x6B0 / +0x6B8 /
/// +0x6B4; +0x1E8, `g_carDriveSpeed`, `g_carDriveSpeed2`, `g_carGear`, +0x200, +0x204,
/// +0x1C4, +0x1F4 0; with +0x68C not 0, +0x688 and +0x684 it less one and +0x68C 0.
pub unsafe fn reset_car_motion() -> Result<(), Refused> {
    unsafe {
        settle_car()?;
        crate::race::start_place::invert_car_matrix();
        zero(car(0x198), 3);
        game_write(car(0x1A4), 0x3F80_0000u32);
        game_write(car(0x1A8), 0u32);
        game_write(car(0x1AC), 0u32);
        zero(car(0x184), 3);
        let heading: u32 = game_read(HEADING_RVA);
        game_write(START_ANGLES_RVA, heading);
        game_write(car(0x6B0), heading);
        let pitch: u32 = game_read(PITCH_RVA);
        game_write(START_ANGLES_RVA + 8, pitch);
        game_write(car(0x6B8), pitch);
        let roll: u32 = game_read(ROLL_RVA);
        game_write(START_ANGLES_RVA + 4, roll);
        game_write(car(0x6B4), roll);
        for offset in [0x1E8, 0x1B8, 0x1BC, 0x1D4, 0x200, 0x204, 0x1C4, 0x1F4] {
            game_write(car(offset), 0u32);
        }
        let pending: u32 = game_read(PENDING_RVA);
        if pending != 0 {
            game_write(car(0x688), pending - 1);
            game_write(car(0x684), pending - 1);
            game_write(PENDING_RVA, 0u32);
        }
    }
    Ok(())
}

/// `math::turn_point_xz` of (x, 0, z) by the start place's heading: x and z moved. The
/// original's y there is a leftover in ebx; it lands only in `g_scratchY`, which every
/// reader writes first.
unsafe fn move_turned(x: u32, z: u32) {
    unsafe {
        let [x, _, z, _] = crate::math::turn_point_xz::turn_point_xz(
            x as i32,
            0,
            z as i32,
            game_read(PLACE_HEADING_RVA),
        );
        add(CAR_X_RVA, x);
        add(CAR_Z_RVA, z);
    }
}

/// `race::start_car` (0x4464AD): the work car's +0x478, +0x694 0, +0xAD4/+0xAD8
/// 0x80000000, +0x1E4, +0x1D8, +0x1D4 0, +0x644 .. +0x650 and +0x638 .. +0x640 0. Unless
/// `dword_62389C` is 4: its checkpoint `first_check_checkpoint` (less the car's number /
/// 2 but with `dword_62389C` 1 or 3) into `dword_6365E4`, `dword_6365EC` 0 (both 0 with
/// `dword_64BB18`); `place_car_on_line`; without `dword_64BB18` moved back (0, -0x20000
/// turned by `dword_5CCD60`) and, with `dword_52A910` or (`dword_62389C` 0 and
/// `dword_63DB94` not 1), to the side (+-0x14000 by the car's number's low bit). Then
/// `nearest_checkpoint` into +0x98, +0x94, +0x9C; its check count less one held to 0 ..
/// 100 into +0x90 and +0xAC; `reset_car_motion`; +0x264 .. +0x26C 0. The registers it
/// leaves are read by no caller. Refuses a nearest checkpoint past the lists (when met).
pub unsafe fn start_car() -> Result<(), Refused> {
    unsafe {
        game_write(car(0x478), 0u32);
        game_write(car(0x694), 0u32);
        game_write(car(0xAD4), 0x8000_0000u32);
        game_write(car(0xAD8), 0x8000_0000u32);
        game_write(car(0x1E4), 0u32);
        game_write(car(0x1D8), 0u32);
        game_write(car(0x1D4), 0u32);
        zero(car(0x644), 4);
        zero(car(0x638), 3);
        if game_read::<u32>(MODE_RVA) != 4 {
            // Converted: `rc2k_core::race::start::{start_checkpoint, starts_aside,
            // side_offset, start_checks}`.
            use rc2k_core::race::start::{side_offset, start_checkpoint, starts_aside};
            let first = crate::race::race_init::first_check_checkpoint()?;
            let mode: u32 = game_read(MODE_RVA);
            let place = start_checkpoint(first, mode, game_read(CURRENT_CAR_RVA));
            game_write(LINE_PLACE_RVA, place);
            game_write(LINE_FRACTION_RVA, 0u32);
            if game_read::<u32>(NO_PLACE_RVA) != 0 {
                game_write(LINE_PLACE_RVA, 0u32);
                game_write(LINE_FRACTION_RVA, 0u32);
            }
            crate::race::start_place::place_car_on_line()?;
            if game_read::<u32>(NO_PLACE_RVA) == 0 {
                move_turned(0, 0xFFFE_0000);
                let side = starts_aside(
                    game_read(SIDE_RVA),
                    || game_read(MODE_RVA),
                    || game_read(CARS_2_RVA),
                );
                if side {
                    move_turned(side_offset(game_read(CURRENT_CAR_RVA)), 0);
                }
            }
        }
        let nearest = crate::race::start_place::nearest_checkpoint()?;
        game_write(car(0x98), nearest);
        game_write(car(0x94), nearest);
        game_write(car(0x9C), nearest);
        if nearest >= MOST_CHECKS {
            return Err(BadInput("a nearest checkpoint past the lists").into());
        }
        let checks =
            rc2k_core::race::start::start_checks(game_read(CHECKS_RVA + 4 * nearest as usize));
        game_write(car(0x90), checks);
        game_write(car(0xAC), checks);
        reset_car_motion()?;
        zero(car(0x264), 3);
    }
    Ok(())
}

/// `race::start_cars` (0x446463): for each car of the race (`g_raceCars`), `g_currentCar`
/// it, `drive::load_car`, `start_car`, the work car's +0xA3C (8 dwords) and +0xA5C (0x11)
/// 0, `drive::store_car`; every register kept round each car (`pusha` / `popa`). Refuses
/// no cars (the original's `jnz` runs on) or more than 8, before anything.
pub unsafe fn start_cars() -> Result<(), Refused> {
    let cars: u32 = unsafe { game_read(RACE_CARS_RVA) };
    if cars == 0 || cars > MOST_CARS {
        return Err(BadInput("no cars or more than eight").into());
    }
    for n in 0..cars {
        unsafe {
            game_write(CURRENT_CAR_RVA, n);
            crate::drive::load_car::load_car()?;
            start_car()?;
            zero(car(0xA3C), 8);
            zero(car(0xA5C), 0x11);
            crate::drive::store_car::store_car()?;
        }
    }
    Ok(())
}
