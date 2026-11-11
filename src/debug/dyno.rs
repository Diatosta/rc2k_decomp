//! The dyno graph (the "Dyno" tab; RAL.EXE 0x428DB3 .. 0x4291C9, no references to it, IDA
//! 2026-10-30). For each forward gear, speeds from 0 to 71 (`flt_5F36C8`): the driven speeds set,
//! `physics::revs_from_wheels`, and below the engine's limit full throttle's
//! 0x43CAAB (`torque`, `physics::drive_force` but for the cut's own check); the colours say
//! where each point lies against the gearbox's shift points. Then the drag at each speed, the
//! best gear's drive less it, and the acceleration test's samples
//! (`physics::test_drive_run`). The original drew pixels on the race's picture on every frame;
//! here the curves are measured once for the car, and every variable the measuring
//! writes is put back (the original kept seven; the rest were the frame's scratch).

use crate::debug::ui_plot::Plot;
use crate::math::x87::{mul_div2, sum_times_int_over_minus};
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque::engine_torque;
use crate::physics::gear_store::GEAR;
use crate::physics::revs_from_wheels::revs_from_wheels;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::scene;
use crate::session::Cell32;
use dear_imgui_rs::Ui;
use std::sync::Mutex;

/// `flt_5F36C8`: the speed at the graph's right edge.
const TOP_SPEED: f32 = 71.0;
/// Points across the graph (the original: one in two of the screen's columns).
const POINTS: usize = 400;

/// Where a point lies against the shift points (the original's colours 3, 5, 4, 0xB).
#[derive(Clone, Copy, PartialEq)]
enum Zone {
    /// At or above `g_shiftUpPoint`.
    ShiftUp,
    /// Between the points.
    Normal,
    /// Below `g_shiftDownClutchPoint`.
    Clutch,
    /// Below `g_shiftDownPoint`.
    ShiftDown,
}

impl Zone {
    fn colour(self) -> [f32; 4] {
        match self {
            Zone::ShiftUp => [1.0, 0.35, 0.3, 1.0],
            Zone::Normal => [0.35, 0.9, 0.4, 1.0],
            Zone::Clutch => [1.0, 0.85, 0.3, 1.0],
            Zone::ShiftDown => [0.4, 0.7, 1.0, 1.0],
        }
    }
}

/// What the gear of a sample of the test drive is drawn in (`dword_5FF258`'s seven, by eye).
const GEAR_COLOURS: [[f32; 4]; 7] = [
    [0.7, 0.7, 0.7, 1.0],
    [1.0, 0.35, 0.3, 1.0],
    [1.0, 0.6, 0.2, 1.0],
    [1.0, 0.9, 0.3, 1.0],
    [0.35, 0.9, 0.4, 1.0],
    [0.4, 0.7, 1.0, 1.0],
    [0.8, 0.5, 1.0, 1.0],
];

/// One measuring.
struct Measured {
    /// Each gear's points: (speed, drive, zone).
    gears: Vec<Vec<(f32, f32, Zone)>>,
    /// At each point's speed: (speed, drag).
    drag: Vec<(f32, f32)>,
    /// At each point's speed: the best gear's drive less the drag.
    net: Vec<(f32, f32)>,
    /// `g_gearCount`, the limit and the three shift points when measured.
    limit: f32,
    points: [f32; 3],
}

static MEASURED: Mutex<Option<Measured>> = Mutex::new(None);

/// Every variable the measuring writes.
fn places(shared: &RaceShared) -> [&dyn Cell32; 15] {
    let drawing = &scene().drawing;
    [
        &shared.car.gear,
        &shared.car.throttle,
        &shared.car.engine_speed,
        &shared.car.local_velocity[2],
        &shared.car.drive_speed_a,
        &shared.car.drive_speed_b,
        &shared.physics_tables.drive_torque,
        &shared.physics_tables.drive_ratio,
        &shared.physics_tables.engine_force,
        &shared.physics_tables.engine_torque,
        &shared.physics_tables.torque_share,
        &drawing.scene.scratch,
        &drawing.scene.work_dword,
        &drawing.scene.work_float_a,
        &drawing.scene.work_float_b,
    ]
}

/// `f` run, the variables put back after it.
fn kept<T>(shared: &RaceShared, f: impl FnOnce() -> T) -> T {
    let places = places(shared);
    let saved = places.map(super::super::session::Cell32::bits);
    let result = f();
    for (place, bits) in places.iter().zip(saved) {
        place.set_bits(bits);
    }
    result
}

/// 0x43CAAB: the drive at the engine's speed and the car's throttle.
fn torque(race: &Race) {
    let shared = &race.shared;
    let course = &race.course;
    engine_torque(race, crate::scene::owner::scene());
    shared
        .physics_tables
        .drive_ratio
        .set_bits(sum_times_int_over_minus(
            shared.physics_tables.engine_force.bits(),
            shared.physics_tables.engine_torque.bits(),
            shared.car.throttle.get() as i32,
            course.numbers.k256.bits(),
        ));
    let gear = shared.car.gear.get() as i32;
    shared.physics_tables.drive_torque.set_bits(mul_div2(
        shared.physics_tables.drive_ratio.bits(),
        GEAR.drive_torque_figure.bits(),
        shared.physics_tables.gear_ratios[(gear + 1) as usize].bits(),
        course.numbers.k10000.bits(),
    ));
    if shared.car.drive_cut_steps.get() != 0 {
        shared.physics_tables.drive_torque.set_bits(0);
    }
    drive_torque_share(
        &race.shared.car,
        &race.shared.physics_tables,
        &race.course.numbers,
        crate::scene::owner::scene(),
    );
}

/// The drive at `speed`, full throttle: None at or above the limit, or where the engine's
/// speed is no number (a ratio of 0).
fn drive_at(race: &Race, speed: f32) -> Option<(f32, f32)> {
    let shared = &race.shared;
    shared.car.drive_speed_a.set_bits(speed.to_bits());
    shared.car.drive_speed_b.set_bits(speed.to_bits());
    shared.car.local_velocity[2].set_bits(speed.to_bits());
    revs_from_wheels(
        &race.shared.car,
        &race.shared.physics_tables,
        &race.course.numbers,
    )
    .ok()?;
    let revs = f32::from_bits(shared.car.engine_speed.bits());
    let limit = f32::from_bits(shared.car.engine_speed_limit.bits());
    if revs.is_nan() || revs >= limit || revs < 0.0 {
        return None;
    }
    shared.car.throttle.set(0x100);
    torque(race);
    Some((
        f32::from_bits(shared.physics_tables.drive_torque.bits()),
        revs / limit,
    ))
}

/// 0x428F42, without its drawing.
fn measure(race: &Race) -> Option<Measured> {
    let shared = &race.shared;
    let course = &race.course;
    let limit = f32::from_bits(shared.car.engine_speed_limit.bits());
    let count = shared.physics_tables.gear_count.get().min(7) as usize;
    if limit.is_nan() || limit <= 0.0 || count == 0 {
        return None;
    }
    let figure = |f: &crate::session::Float| f32::from_bits(f.bits());
    let up = figure(&GEAR.shift_up_point);
    let clutch = figure(&GEAR.shift_down_clutch_point);
    let down = figure(&GEAR.shift_down_point);
    let air = figure(&shared.physics_tables.air_drag);
    let rolling = figure(&shared.physics_tables.rolling_drag);
    let k = f32::from_bits(course.numbers.k10000.bits());
    Some(kept(shared, || {
        let mut gears = Vec::new();
        let mut best = [f32::MIN; POINTS];
        for gear in 1..=count {
            shared.car.gear.set(gear as u32);
            let mut curve = Vec::new();
            for (n, best) in best.iter_mut().enumerate() {
                let speed = n as f32 / POINTS as f32 * TOP_SPEED;
                let Some((drive, part)) = drive_at(race, speed) else {
                    continue;
                };
                *best = best.max(drive);
                let zone = if part >= up {
                    Zone::ShiftUp
                } else if part < down {
                    Zone::ShiftDown
                } else if part < clutch {
                    Zone::Clutch
                } else {
                    Zone::Normal
                };
                curve.push((speed, drive, zone));
            }
            gears.push(curve);
        }
        let mut drag = Vec::new();
        let mut net = Vec::new();
        for (n, best) in best.iter().enumerate() {
            let speed = n as f32 / POINTS as f32 * TOP_SPEED;
            let d = speed * speed * air / k + rolling;
            drag.push((speed, d));
            if *best > f32::MIN {
                net.push((speed, best - d));
            }
        }
        Measured {
            gears,
            drag,
            net,
            limit,
            points: [up, clutch, down],
        }
    }))
}

/// The car's speed and its drive right now, full throttle (0x4291C9), in a forward gear
/// below the limit.
fn cursor(race: &Race) -> Option<(f32, f32, f32, i32)> {
    let shared = &race.shared;
    let gear = shared.car.gear.get() as i32;
    let speed = f32::from_bits(shared.car.drive_speed_a.bits());
    if !(1..=7).contains(&gear)
        || !speed.is_finite()
        || f32::from_bits(shared.car.engine_speed_limit.bits()).is_nan()
        || f32::from_bits(shared.car.engine_speed_limit.bits()) <= 0.0
    {
        return None;
    }
    let (drive, part) = kept(&race.shared, || drive_at(race, speed))?;
    Some((speed, drive, part, gear))
}

/// The tab.
pub fn tab(race: &Race, ui: &Ui) {
    let mut measured = MEASURED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ui.button("Measure the car") || measured.is_none() {
        *measured = measure(race);
    }
    ui.same_line();
    ui.text_disabled("the last car simulated, full throttle; measure again after a change of car");
    let Some(m) = measured.as_ref() else {
        ui.text_disabled("No car loaded: open this in a race.");
        return;
    };
    ui.text(format!(
        "{} gears, limit {:.0}; shift up at {:.2}, clutch below {:.2}, down below {:.2} of it",
        m.gears.len(),
        m.limit,
        m.points[0],
        m.points[1],
        m.points[2],
    ));
    for (zone, text) in [
        (Zone::ShiftUp, "over shift up"),
        (Zone::Normal, "normal"),
        (Zone::Clutch, "clutch"),
        (Zone::ShiftDown, "shift down"),
    ] {
        ui.text_colored(zone.colour(), text);
        ui.same_line();
    }
    ui.text_colored(DRAG, "drag");
    ui.same_line();
    ui.text_colored(NET, "best drive less drag");
    let live = cursor(race);
    let high = m
        .gears
        .iter()
        .flatten()
        .map(|p| p.1)
        .fold(0.0_f32, f32::max);
    let low = m.net.iter().map(|p| p.1).fold(0.0_f32, f32::min);
    let width = ui.content_region_avail_width();
    let height = (ui.content_region_avail()[1] - 150.0).max(160.0);
    let plot = Plot::new(ui, [width, height], [0.0, TOP_SPEED], [low, high * 1.05]);
    plot.grid(ui, 10.0, (high / 5.0).max(0.001));
    for gear in &m.gears {
        for pair in gear.windows(2) {
            plot.line(
                ui,
                (pair[0].0, pair[0].1),
                (pair[1].0, pair[1].1),
                pair[0].2.colour(),
            );
        }
    }
    plot.path(ui, &m.drag, DRAG);
    plot.path(ui, &m.net, NET);
    if let Some((speed, drive, _, _)) = live {
        plot.cross(ui, (speed, drive));
    }
    match live {
        Some((speed, drive, part, gear)) => ui.text(format!(
            "Now: speed {speed:.1}, gear {gear}, {:.0}% of the limit, full throttle's drive {drive:.2}",
            part * 100.0
        )),
        None => ui.text_disabled("Now: neutral, reverse or over the limit"),
    }
    let samples: Vec<(f32, f32, usize)> = GEAR
        .test_speeds
        .iter()
        .zip(&GEAR.test_gears)
        .enumerate()
        .map(|(n, (s, g))| {
            (
                n as f32,
                f32::from_bits(s.bits()),
                (g.get() as usize).min(6),
            )
        })
        .collect();
    let top = samples.iter().map(|s| s.1).fold(0.0_f32, f32::max);
    if top > 0.0 {
        ui.text("The acceleration test: speed by time (one sample in twenty steps), by gear");
        let plot = Plot::new(ui, [width, 110.0], [0.0, 255.0], [0.0, top * 1.05]);
        plot.grid(ui, 50.0, (top / 4.0).max(0.001));
        for pair in samples.windows(2) {
            plot.line(
                ui,
                (pair[0].0, pair[0].1),
                (pair[1].0, pair[1].1),
                GEAR_COLOURS[pair[1].2],
            );
        }
    } else {
        ui.text_disabled("The acceleration test has not run for this car.");
    }
}

const DRAG: [f32; 4] = [0.9, 0.5, 0.9, 1.0];
const NET: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::owner::race;

    #[test]
    fn measuring_puts_back_all_it_writes() {
        let shared = &race().shared;
        let _lock = crate::store::test_lock();
        let was_limit = shared.car.engine_speed_limit.bits();
        let was_count = shared.physics_tables.gear_count.get();
        let was_ratio = shared.physics_tables.gear_ratios[2].bits();
        shared.car.engine_speed_limit.set_bits(7000.0_f32.to_bits());
        shared.physics_tables.gear_count.set(1);
        shared.physics_tables.gear_ratios[2].set_bits(0.01_f32.to_bits());
        let before = places(shared).map(crate::session::Cell32::bits);
        let m = measure(race()).expect("a car");
        assert_eq!(places(shared).map(crate::session::Cell32::bits), before);
        assert_eq!(m.gears.len(), 1);
        assert!(!m.gears[0].is_empty());
        shared.car.engine_speed_limit.set_bits(was_limit);
        shared.physics_tables.gear_count.set(was_count);
        shared.physics_tables.gear_ratios[2].set_bits(was_ratio);
    }

    #[test]
    fn no_limit_no_measuring() {
        let shared = &race().shared;
        let _lock = crate::store::test_lock();
        let was = shared.car.engine_speed_limit.bits();
        shared.car.engine_speed_limit.set_bits(0);
        assert!(measure(race()).is_none());
        shared.car.engine_speed_limit.set_bits(was);
    }
}
