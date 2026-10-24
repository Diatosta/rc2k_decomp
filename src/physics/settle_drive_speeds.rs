//! `physics::settle_drive_speeds` (0x43BF6F): driven speeds brought toward 0 by a step
//! (`flt_613F3C`, 1.0, when `dword_71A614` is set, else `flt_613F40`, 0.01; stored in
//! `dword_62400C`), never past it: `g_carDriveSpeed` when `dword_71A220` is set and the car
//! is in neutral, `g_carDriveSpeed2` when `dword_71A224` is set (whatever the gear). A
//! speed of 0 (or NaN) is left; one that reaches or crosses 0 becomes +0 (a NaN after the
//! step too). Then, in gear, `physics::revs_from_wheels`.
//!
//! In:  nothing
//! Out: in gear, ebx = `g_carGear` (`revs_from_wheels`); the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B7AD, before `sub_43EA54`), every physics step. Refuses
//! (before anything) a gear outside -1 ..= 7 (`revs_from_wheels`' table).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, rva};
use crate::math::x87;
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};

const FAST_STEP_RVA: usize = rva(0x613F3C); // flt_613F3C, 1.0
const SLOW_STEP_RVA: usize = rva(0x613F40); // flt_613F40, 0.01

/// The switches and the scratch: the game's or a test's.
#[derive(Clone, Copy)]
pub struct SettlePlaces {
    /// `dword_71A220`: settle the first driven speed (in neutral).
    pub first: u32,
    /// `dword_71A224`: settle the second.
    pub second: u32,
    /// `dword_71A614`: the big step.
    pub fast: u32,
    /// `dword_62400C`.
    pub scratch: u32,
}

impl SettlePlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            first: g(0x71A220),
            second: g(0x71A224),
            fast: g(0x71A614),
            scratch: g(0x62400C),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// One speed at `v` toward 0 by the step at `step`.
unsafe fn settle(v: u32, step: u32) {
    unsafe {
        let x = f32::from_bits(at(v));
        if x == 0.0 || x.is_nan() {
            return;
        }
        if x < 0.0 {
            let y = x87::add(at(v), at(step));
            put(v, y);
            if f32::from_bits(y) >= 0.0 {
                put(v, 0);
            }
        } else {
            let y = x87::sub(at(v), at(step));
            put(v, y);
            let y = f32::from_bits(y);
            if y < 0.0 || y.is_nan() {
                put(v, 0);
            }
        }
    }
}

/// The Rust core.
pub unsafe fn settle_drive_speeds_at(g: &GearPlaces, s: &SettlePlaces) -> Result<(), Refused> {
    unsafe {
        let gear = at(g.gear);
        if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
            return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
        }
        let step = || {
            game_read::<u32>(if at(s.fast) != 0 {
                FAST_STEP_RVA
            } else {
                SLOW_STEP_RVA
            })
        };
        if at(s.first) != 0 && gear == 0 {
            put(s.scratch, step());
            settle(g.drive_speed, s.scratch);
        }
        if at(s.second) != 0 {
            put(s.scratch, step());
            settle(g.drive_speed_2, s.scratch);
        }
        if gear != 0 {
            return revs_from_wheels_at(g);
        }
        Ok(())
    }
}

/// The Rust version (the exact one, for `physics::car_physics_step`).
pub unsafe fn settle_drive_speeds() -> Result<(), Refused> {
    unsafe { settle_drive_speeds_at(&GearPlaces::game(), &SettlePlaces::game()) }
}
