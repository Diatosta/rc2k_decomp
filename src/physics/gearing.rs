//! The gearing's sums between the engine's speed and the driven wheels' (fields of the
//! current car's work copy, `drive/car.rs`): `physics::revs_from_wheels` (0x43C0C9),
//! `physics::wheels_from_revs` (0x43C124). Both do nothing in gear 0 (neutral); else they
//! divide or multiply by the gear's ratio, `g_gearRatios[gear]` (0x6140C0; gear -1, the
//! reverse, at 0x6140BC; seven forward gears by `sub_4219E6`). With `g_carDriveKind` 1 the
//! two driven speeds are averaged / both set: by the look of it four-wheel drive (a
//! reading, not seen in a race). The exact x87 versions (`*_at`) serve the Rust callers
//! in the gearbox; the hooks run the converted ones (`*_plain_at`,
//! `rc2k_core::physics::gearing`).

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, game_addr, game_read, rva};
use crate::math::x87;
use crate::screens::harness::Left;
use rc2k_core::physics::gearing;

pub const GEAR_RVA: usize = rva(0x71A1AC); // g_carGear: -1 reverse, 0 neutral, 1 ..
pub const ENGINE_SPEED_RVA: usize = rva(0x71A1B0); // g_carEngineSpeed
pub const DRIVE_SPEED_RVA: usize = rva(0x71A190); // g_carDriveSpeed
pub const DRIVE_SPEED_2_RVA: usize = rva(0x71A194); // g_carDriveSpeed2: the second axle's
pub const DRIVE_KIND_RVA: usize = rva(0x71A018); // g_carDriveKind: 1 both axles
pub const GEAR_RATIOS_RVA: usize = rva(0x6140C0); // g_gearRatios, indexed by the gear
pub const SPEED_RVA: usize = rva(0x71A184); // g_carSpeed (a reading: times sines in sub_43CD79)
const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40, 0.5

/// The variables' addresses: the game's, or a test's (`ratios` is gear 0's entry).
#[derive(Clone, Copy)]
pub struct GearPlaces {
    pub gear: u32,
    pub engine_speed: u32,
    pub drive_speed: u32,
    pub drive_speed_2: u32,
    pub drive_kind: u32,
    pub ratios: u32,
    pub speed: u32,
}

impl GearPlaces {
    pub fn game() -> Self {
        Self {
            gear: game_addr(GEAR_RVA) as u32,
            engine_speed: game_addr(ENGINE_SPEED_RVA) as u32,
            drive_speed: game_addr(DRIVE_SPEED_RVA) as u32,
            drive_speed_2: game_addr(DRIVE_SPEED_2_RVA) as u32,
            drive_kind: game_addr(DRIVE_KIND_RVA) as u32,
            ratios: game_addr(GEAR_RATIOS_RVA) as u32,
            speed: game_addr(SPEED_RVA) as u32,
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The gear (None for neutral) and its ratio; refused outside the ratio table (-1 ..= 7).
unsafe fn gear_ratio(p: &GearPlaces) -> Result<Option<(u32, u32)>, Refused> {
    unsafe {
        let gear = at(p.gear);
        if gear == 0 {
            return Ok(None);
        }
        if !(-1..=7).contains(&(gear as i32)) {
            return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
        }
        Ok(Some((
            gear,
            at(p.ratios.wrapping_add(gear.wrapping_mul(4))),
        )))
    }
}

/// `physics::revs_from_wheels` (0x43C0C9): in gear, engine speed = the driven speed (with
/// drive kind 1 the two driven speeds' sum times 0.5) over the gear's ratio. Leaves ebx
/// the gear (untouched in neutral).
pub unsafe fn revs_from_wheels_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = gear_ratio(p)? else {
            return Ok(Left::nothing());
        };
        let speed = if at(p.drive_kind) == 1 {
            crate::math::x87_run::sum_times_over(
                at(p.drive_speed),
                at(p.drive_speed_2),
                game_read(HALF_RVA),
                ratio,
            )
        } else {
            x87::div(at(p.drive_speed), ratio)
        };
        put(p.engine_speed, speed);
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// `physics::wheels_from_revs` (0x43C124): in gear, driven speed = engine speed times the
/// gear's ratio; with drive kind 1 the second driven speed the same (reloaded: `fld;
/// fstp`). Leaves ebx the gear (untouched in neutral).
pub unsafe fn wheels_from_revs_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = gear_ratio(p)? else {
            return Ok(Left::nothing());
        };
        let speed = x87::mul(at(p.engine_speed), ratio);
        put(p.drive_speed, speed);
        if at(p.drive_kind) == 1 {
            put(p.drive_speed_2, x87::loaded(speed));
        }
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// `physics::revs_from_speed` (0x43C101): in gear, engine speed = `g_carSpeed` over the
/// gear's ratio. Leaves ebx the gear (untouched in neutral).
pub unsafe fn revs_from_speed_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = gear_ratio(p)? else {
            return Ok(Left::nothing());
        };
        put(p.engine_speed, x87::div(at(p.speed), ratio));
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// The gear (None for neutral) and its ratio, through the core's `ratio_index`.
unsafe fn plain_ratio(p: &GearPlaces) -> Result<Option<(u32, f32)>, Refused> {
    unsafe {
        let gear = at(p.gear);
        let Some(index) = gearing::ratio_index(gear)? else {
            return Ok(None);
        };
        let ratio = at(p.ratios.wrapping_sub(4).wrapping_add(4 * index as u32));
        Ok(Some((gear, f32::from_bits(ratio))))
    }
}

/// `revs_from_wheels_at` converted (`rc2k_core::physics::gearing`, plain floats): the
/// hook's.
// Converted: `rc2k_core::physics::gearing::revs_from_wheels`.
pub unsafe fn revs_from_wheels_plain_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = plain_ratio(p)? else {
            return Ok(Left::nothing());
        };
        let f = |a: u32| f32::from_bits(at(a));
        let both = at(p.drive_kind) == 1;
        // The second speed and the half read only with drive kind 1, as the original.
        let (second, half) = if both {
            (f(p.drive_speed_2), f32::from_bits(game_read(HALF_RVA)))
        } else {
            (0.0, 0.0)
        };
        let speed = gearing::revs_from_wheels(f(p.drive_speed), second, both, half, ratio);
        put(p.engine_speed, speed.to_bits());
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// `wheels_from_revs_at` converted (`rc2k_core::physics::gearing`, plain floats): the
/// hook's.
// Converted: `rc2k_core::physics::gearing::wheels_from_revs`.
pub unsafe fn wheels_from_revs_plain_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = plain_ratio(p)? else {
            return Ok(Left::nothing());
        };
        let engine = f32::from_bits(at(p.engine_speed));
        let both = at(p.drive_kind) == 1;
        let (speed, second) = gearing::wheels_from_revs(engine, ratio, both);
        put(p.drive_speed, speed.to_bits());
        if let Some(second) = second {
            put(p.drive_speed_2, second.to_bits());
        }
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// `revs_from_speed_at` converted (`rc2k_core::physics::gearing`, plain floats): the
/// hook's.
// Converted: `rc2k_core::physics::gearing::revs_from_speed`.
pub unsafe fn revs_from_speed_plain_at(p: &GearPlaces) -> Result<Left, Refused> {
    unsafe {
        let Some((gear, ratio)) = plain_ratio(p)? else {
            return Ok(Left::nothing());
        };
        let speed = gearing::revs_from_speed(f32::from_bits(at(p.speed)), ratio);
        put(p.engine_speed, speed.to_bits());
        Ok(Left::nothing().with(EBX, gear))
    }
}

/// `dispatch_gearing` without its hook (Rust callers; a refusal is the caller's).
pub fn gearing_entry(
    r: &mut crate::framework::Regs,
    rust: crate::service::harness::Rust,
) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, rust)
}
