//! The dyno graph (the "Dyno" tab). For each forward gear, speeds from 0 to 71: the driven
//! speeds set, `physics::revs_from_wheels`, and below the engine's limit full throttle's drive;
//! the colours say where each point lies against the gearbox's shift points. Then the drag, the
//! best gear's drive less it, and the acceleration test's samples (`physics::acceleration_run`).
//! The curves are measured once for the car, and every variable the measuring writes is put
//! back.

use crate::debug::ui_plot::Plot;
use crate::math::x87::{mul_div2, sum_times_int_over_minus};
use crate::memory::plain_floats::FloatRowMut;
use crate::physics::drive_torque_share::drive_torque_share;
use crate::physics::engine_torque::engine_torque;
use crate::physics::revs_from_wheels::revs_from_wheels;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::owner::Scene;
use dear_imgui_rs::Ui;
use std::sync::Mutex;

/// The speed at the graph's right edge.
const TOP_SPEED: f32 = 71.0;
const POINTS: usize = 400;

#[derive(Clone, Copy, PartialEq)]
enum Zone {
    /// At or above the shift-up point.
    ShiftUp,
    /// Between the points.
    Normal,
    /// Below the shift-down clutch point.
    Clutch,
    /// Below the shift-down point.
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

/// What the gear of a sample of the test drive is drawn in.
const GEAR_COLOURS: [[f32; 4]; 7] = [
    [0.7, 0.7, 0.7, 1.0],
    [1.0, 0.35, 0.3, 1.0],
    [1.0, 0.6, 0.2, 1.0],
    [1.0, 0.9, 0.3, 1.0],
    [0.35, 0.9, 0.4, 1.0],
    [0.4, 0.7, 1.0, 1.0],
    [0.8, 0.5, 1.0, 1.0],
];

struct Measured {
    /// Each gear's points: (speed, drive, zone).
    gears: Vec<Vec<(f32, f32, Zone)>>,
    /// At each point's speed: (speed, drag).
    drag: Vec<(f32, f32)>,
    /// At each point's speed: the best gear's drive less the drag.
    net: Vec<(f32, f32)>,
    /// The gear count, the limit and the three shift points when measured.
    limit: f32,
    points: [f32; 3],
}

static MEASURED: Mutex<Option<Measured>> = Mutex::new(None);

/// The physics tables' variables the measuring writes: the drive torque, the drive ratio, the
/// engine's force and torque and the torque's share.
fn table_places(shared: &RaceShared) -> [f32; 5] {
    let tables = shared.physics_tables();
    [
        tables.drive_torque,
        tables.drive_ratio,
        tables.engine_force,
        tables.engine_torque,
        tables.torque_share,
    ]
}

/// `table_places` put back.
fn put_table_places(shared: &RaceShared, saved: [f32; 5]) {
    let mut tables = shared.physics_tables_mut();
    [
        tables.drive_torque,
        tables.drive_ratio,
        tables.engine_force,
        tables.engine_torque,
        tables.torque_share,
    ] = saved;
}

/// `f` run, the variables it writes put back after it.
fn kept<T>(shared: &RaceShared, scene: &Scene, f: impl FnOnce() -> T) -> T {
    let saved_tables = table_places(shared);
    let car = {
        let car = shared.car();
        (
            car.gear,
            car.throttle,
            car.engine_speed,
            car.local_velocity[2],
            car.drive_speed_a,
            car.drive_speed_b,
        )
    };
    let work = scene.draw_work().clone();
    let result = f();
    *scene.draw_work_mut() = work;
    put_table_places(shared, saved_tables);
    let mut now = shared.car_mut();
    now.gear = car.0;
    now.throttle = car.1;
    now.engine_speed = car.2;
    now.local_velocity[2] = car.3;
    now.drive_speed_a = car.4;
    now.drive_speed_b = car.5;
    result
}

/// The drive at the engine's speed and the car's throttle.
fn torque(race: &Race, scene: &Scene) {
    let shared = &race.shared;
    let course = &race.course;
    engine_torque(&race.course, &race.shared, scene);
    {
        let mut tables = shared.physics_tables_mut();
        tables.drive_ratio = sum_times_int_over_minus(
            tables.engine_force,
            tables.engine_torque,
            shared.car().throttle as i32,
            course.numbers.k256,
        );
        let gear = shared.car().gear;
        tables.drive_torque = mul_div2(
            tables.drive_ratio,
            &race.gear().drive_torque_figure,
            tables.gear_ratios[(gear + 1) as usize],
            course.numbers.k10000,
        );
        if shared.car().drive_cut_steps != 0 {
            tables.drive_torque = 0.0;
        }
    }
    drive_torque_share(
        &race.shared.car(),
        &mut race.shared.physics_tables_mut(),
        &race.course.numbers,
        scene,
    );
}

/// The drive at `speed`, full throttle: None at or above the limit, or where the engine's
/// speed is no number (a ratio of 0).
fn drive_at(race: &Race, scene: &Scene, speed: f32) -> Option<(f32, f32)> {
    let shared = &race.shared;
    shared.car_mut().drive_speed_a = speed;
    shared.car_mut().drive_speed_b = speed;
    shared.car_mut().local_velocity.set_at(2, speed);
    revs_from_wheels(
        &mut race.shared.car_mut(),
        &race.shared.physics_tables(),
        &race.course.numbers,
    )
    .ok()?;
    let revs = shared.car().engine_speed;
    let limit = shared.car().engine_speed_limit;
    if revs.is_nan() || revs >= limit || revs < 0.0 {
        return None;
    }
    shared.car_mut().throttle = 0x100;
    torque(race, scene);
    Some((shared.physics_tables().drive_torque, revs / limit))
}

/// The measuring, without its drawing.
fn measure(race: &Race, scene: &Scene) -> Option<Measured> {
    let shared = &race.shared;
    let course = &race.course;
    let limit = shared.car().engine_speed_limit;
    let count = shared.physics_tables().gear_count.min(7) as usize;
    if limit.is_nan() || limit <= 0.0 || count == 0 {
        return None;
    }
    let figure = |f: &dyn crate::memory::session::FloatLike| f.get();
    let up = figure(&race.gear().shift_up_point);
    let clutch = figure(&race.gear().shift_down_clutch_point);
    let down = figure(&race.gear().shift_down_point);
    let (air, rolling) = (
        course.numbers.air_drag.get(),
        course.numbers.rolling_drag.get(),
    );
    let k = course.numbers.k10000.get();
    Some(kept(shared, scene, || {
        let mut gears = Vec::new();
        let mut best = [f32::MIN; POINTS];
        for gear in 1..=count {
            shared.car_mut().gear = gear as i32;
            let mut curve = Vec::new();
            for (n, best) in best.iter_mut().enumerate() {
                let speed = n as f32 / POINTS as f32 * TOP_SPEED;
                let Some((drive, part)) = drive_at(race, scene, speed) else {
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

/// The car's speed and its drive right now, full throttle, in a forward gear
/// below the limit.
fn cursor(race: &Race, scene: &Scene) -> Option<(f32, f32, f32, i32)> {
    let shared = &race.shared;
    let gear = shared.car().gear;
    let speed = shared.car().drive_speed_a;
    if !(1..=7).contains(&gear)
        || !speed.is_finite()
        || shared.car().engine_speed_limit.is_nan()
        || shared.car().engine_speed_limit <= 0.0
    {
        return None;
    }
    let (drive, part) = kept(&race.shared, scene, || drive_at(race, scene, speed))?;
    Some((speed, drive, part, gear))
}

/// The tab.
pub fn tab(race: &Race, scene: &Scene, ui: &Ui) {
    let mut measured = MEASURED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ui.button("Measure the car") || measured.is_none() {
        *measured = measure(race, scene);
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
    let live = cursor(race, scene);
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
    let samples: Vec<(f32, f32, usize)> = race
        .gear()
        .test_speeds
        .iter()
        .zip(&race.gear().test_gears)
        .enumerate()
        .map(|(n, (s, g))| (n as f32, s.get(), (g.get() as usize).min(6)))
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

    #[test]
    fn measuring_puts_back_all_it_writes() {
        let shared = &crate::test_support::own::race().shared;
        let _lock = crate::test_support::test_lock();
        let was_limit = shared.car().engine_speed_limit;
        let was_count = shared.physics_tables().gear_count;
        let was_ratio = shared.physics_tables().gear_ratios[2];
        shared.car_mut().engine_speed_limit = 7000.0_f32;
        shared.physics_tables_mut().gear_count = 1;
        shared.physics_tables_mut().gear_ratios[2] = 0.01_f32;
        let snapshot = || {
            let car = shared.car();
            (
                table_places(shared).map(f32::to_bits),
                (
                    car.gear,
                    car.throttle,
                    car.engine_speed.to_bits(),
                    car.local_velocity[2].to_bits(),
                    car.drive_speed_a.to_bits(),
                    car.drive_speed_b.to_bits(),
                ),
                format!("{:?}", crate::test_support::own::scene().draw_work()),
            )
        };
        let before = snapshot();
        let m = measure(
            crate::test_support::own::race(),
            crate::test_support::own::scene(),
        )
        .expect("a car");
        assert_eq!(snapshot(), before);
        assert_eq!(m.gears.len(), 1);
        assert!(!m.gears[0].is_empty());
        shared.car_mut().engine_speed_limit = was_limit;
        shared.physics_tables_mut().gear_count = was_count;
        shared.physics_tables_mut().gear_ratios[2] = was_ratio;
    }

    #[test]
    fn no_limit_no_measuring() {
        let shared = &crate::test_support::own::race().shared;
        let _lock = crate::test_support::test_lock();
        let was = shared.car().engine_speed_limit;
        shared.car_mut().engine_speed_limit = 0.0;
        assert!(
            measure(
                crate::test_support::own::race(),
                crate::test_support::own::scene()
            )
            .is_none()
        );
        shared.car_mut().engine_speed_limit = was;
    }
}
