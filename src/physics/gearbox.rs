//! The gearbox (IDA 2026-10-18): `physics::gearbox_step` (0x43C15C) and what it runs,
//! `physics::manual_gearbox` (0x43C2AE, and its one-call twin 0x43C2A8),
//! `physics::auto_gearbox` (0x43C5C2), `physics::pick_start_gear` (0x43C8AC). This file:
//! the steps they share, on the game's variables (the work car's gear fields,
//! `physics/gearing.rs`), with the registers threaded through as the originals leave
//! them, and the test they share.
//!
//! The shift points `flt_5CB958` .. `flt_5CB964`, `flt_614104` and `flt_6140F8` are set at
//! run time (0 in the file); the tests set them.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::physics::gear_bit::gear_bit;
use crate::physics::gearing::{
    GearPlaces, revs_from_speed_at, revs_from_wheels_at, wheels_from_revs_at,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};
use std::cmp::Ordering;

pub const GEAR: usize = 0x71A1AC; // g_carGear
pub const ENGINE: usize = 0x71A1B0; // g_carEngineSpeed
pub const LIMIT: usize = 0x71A1B4; // g_car + 0x1DC: the engine's top
pub const SPEED: usize = 0x71A184; // g_carSpeed
pub const HOLD: usize = 0x71A1A4; // g_car + 0x1CC: frames before the next change
pub const CHANGED: usize = 0x71A210; // g_car + 0x238: 1 changed, -1 refused
pub const CHANGED_2: usize = 0x71A214; // g_car + 0x23C
pub const GEAR_COUNT: usize = 0x614C10; // g_gearCount
pub const RATIOS: usize = 0x6140C0; // g_gearRatios (gear -1 at 0x6140BC)
pub const SCRATCH: usize = 0x62400C;

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// C0 of comparing `a` with `b`: below, or unordered.
pub fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The common end of a change: +0x1D0 = `dword_613FA4`, the hold 20, both words 1.
pub fn settle() {
    p(0x71A1A8, g(0x613FA4));
    p(HOLD, rc2k_core::physics::gear_shifts::HOLD);
    p(CHANGED, 1);
    p(CHANGED_2, 1);
}

/// Neutral, then `settle`.
pub fn neutral() {
    p(GEAR, 0);
    settle();
}

/// `gear_bit` with ecx `gear`: true for CF (the gear cannot be taken).
pub fn bit(r: &mut Regs, gear: u32) -> Result<bool, Refused> {
    r.ecx = gear;
    let f = unsafe { gear_bit(gear) }?;
    f.apply(r);
    Ok(f.carry)
}

pub fn wheels(r: &mut Regs) -> Result<(), Refused> {
    unsafe { wheels_from_revs_at(&GearPlaces::game()) }?.apply(r);
    Ok(())
}

pub fn revs(r: &mut Regs) -> Result<(), Refused> {
    unsafe { revs_from_wheels_at(&GearPlaces::game()) }?.apply(r);
    Ok(())
}

pub fn revs_speed(r: &mut Regs) -> Result<(), Refused> {
    unsafe { revs_from_speed_at(&GearPlaces::game()) }?.apply(r);
    Ok(())
}

/// The engine speed over its top (`fld; fdiv; fstp`).
pub fn over_top() -> u32 {
    x87::div(g(ENGINE), g(LIMIT))
}

/// A gear's ratio; refused outside -1 ..= 7.
pub fn ratio(gear: u32) -> Result<u32, Refused> {
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    Ok(g(RATIOS.wrapping_add(4 * gear as usize)))
}

/// What the gearbox writes (the work car, its copy of +8, the start gear's three, the
/// scratch).
pub fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x719FD8), 0xFE0),
        (a(0x6140EC), 8),
        (a(0x61413C), 12),
        (a(SCRATCH), 4),
        (a(0x614C40), 4),
    ]
}

/// Runs `step` with every register model start `entry`; on a refusal what was written is
/// put back. Returns all the registers as left.
pub fn guarded(
    entry: &Regs,
    step: &dyn Fn(&mut Regs) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match step(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

/// One test case's settings.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    pub gear: u32,
    pub bits: u32,
    pub mode: u32,
    pub kind_010: u32,
    pub up: u32,
    pub down: u32,
    pub auto_key: u32,
    pub hold: u32,
    pub block: u32,
    pub wait: u32,
    pub still: u32,
    pub fast: u32,
    pub engine: f32,
    pub speed: f32,
    pub asked: u32,
}

/// The inputs a case sets, beyond `regions`.
pub fn inputs() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x6138A0), 12),
        (a(GEAR_COUNT), 4),
        (a(0x6140BC), 0x24),
        (a(0x5CB958), 16),
        (a(0x614104), 4),
        (a(0x6140F8), 4),
        (a(0x62389C), 4),
    ]
}

/// Puts a case in place.
pub fn set_case(c: &Case) {
    p(GEAR, c.gear);
    p(0x71A40C, c.bits);
    p(0x71A234, c.mode);
    p(0x71A010, c.kind_010);
    p(0x6138A0, c.up);
    p(0x6138A4, c.down);
    p(0x6138A8, c.auto_key);
    p(HOLD, c.hold);
    p(0x71A408, c.block);
    p(0x71A404, c.wait);
    p(0x71A22C, c.still);
    p(0x71A614, c.fast);
    p(ENGINE, c.engine.to_bits());
    p(LIMIT, 6000.0f32.to_bits());
    p(SPEED, c.speed.to_bits());
    p(0x71A190, (c.speed * 1.1).to_bits());
    p(0x71A194, (c.speed * 1.05).to_bits());
    p(0x71A018, c.gear & 1);
    p(GEAR_COUNT, 5);
    for (k, r) in [-4.0f32, 0.0, 3.5, 2.2, 1.5, 1.1, 0.9, 0.8, 0.7]
        .iter()
        .enumerate()
    {
        p(0x6140BC + 4 * k, r.to_bits());
    }
    for (k, v) in [0.9f32, 0.5, 0.6, 0.8].iter().enumerate() {
        p(0x5CB958 + 4 * k, v.to_bits());
    }
    p(0x614104, 2500.0f32.to_bits());
    p(0x6140F8, 1500.0f32.to_bits());
    p(0x62389C, 1);
    p(0x71A080, 0);
    p(0x719FEC, 0x400);
    p(0x71A440, (0x1234_50FF & !0xF00) | (c.asked << 8));
    p(0x71A290, 0);
    p(0x71A3CC, 1);
    p(0x71A1A8, 0x5A5A);
}

/// The cases the tests run through: gears, bits, modes, keys, flags, speeds.
pub fn cases() -> Vec<Case> {
    let base = Case {
        gear: 1,
        bits: 0,
        mode: 1,
        kind_010: 0,
        up: 0,
        down: 0,
        auto_key: 0,
        hold: 0,
        block: 0,
        wait: 0,
        still: 0,
        fast: 0,
        engine: 3000.0,
        speed: 20.0,
        asked: 0xE,
    };
    let mut out = vec![];
    for gear in [u32::MAX, 0, 1, 2, 3, 5] {
        for engine in [400.0f32, 3000.0, 5800.0] {
            for (up, down) in [(0u32, 0u32), (1, 0), (0, 1)] {
                for mode in [1u32, 2] {
                    out.push(Case {
                        gear,
                        engine,
                        up,
                        down,
                        mode,
                        ..base
                    });
                }
            }
        }
    }
    for bits in [0x004u32, 0x1F0, 0x001] {
        for gear in [0u32, 2, 3] {
            out.push(Case {
                gear,
                bits,
                up: 1,
                ..base
            });
            out.push(Case {
                gear,
                bits,
                down: 1,
                ..base
            });
        }
    }
    out.push(Case { hold: 5, ..base });
    out.push(Case { block: 1, ..base });
    out.push(Case { wait: 1, ..base });
    out.push(Case {
        kind_010: 1,
        ..base
    });
    out.push(Case {
        kind_010: 2,
        ..base
    });
    out.push(Case {
        still: 1,
        mode: 2,
        ..base
    });
    for fast in [0u32, 1] {
        for (gear, speed) in [(u32::MAX, -2.0f32), (1, 0.5), (1, 3.0), (0, 0.0)] {
            out.push(Case {
                gear,
                speed,
                fast,
                auto_key: u32::MAX,
                ..base
            });
            out.push(Case {
                gear,
                speed,
                fast,
                mode: 2,
                ..base
            });
        }
    }
    out
}

/// The test of one of them on the game's variables (saved and put back): `original`
/// against `rust` on `cases()` with `fix` applied (a function's own settings).
pub fn test_gearbox(
    t: &mut SelfTest,
    original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
    fix: &dyn Fn(&mut Case),
) {
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    for c in cases() {
        let mut c = c;
        fix(&mut c);
        check_state(
            t,
            &format!("{c:?}"),
            original,
            Regs::SENTINEL,
            &|| set_case(&c),
            &all,
            rust,
        );
    }
    restore(&all, &saved);
}
