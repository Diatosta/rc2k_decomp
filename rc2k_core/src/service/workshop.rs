//! The work chosen in the service area carried out, on leaving it: each chosen repair shown being
//! done, a frame at a time, then the chosen setup installed.
//!
//! A chosen item is one whose flag is exactly 1. Items 2 .. 0xF are the
//! car's parts, their damage `Player::damage[item - 2]` (the tyres' the last).

use crate::menu::player::Player;
use crate::service::car::Part;

pub const BODY: u32 = 3;
pub(crate) const GEARBOX: u32 = 0xE;
pub(crate) const TYRES: u32 = 0xF;
/// The state bits a repaired part sets in `state_flags`: the tyres', the body's (all the
/// others).
pub(crate) const TYRE_BITS: u32 = 0x1E;
pub(crate) const BODY_BITS: u32 = 0xFFFF_FFE1;

/// The parts whose wear is zeroed before the repairs are shown, by item.
pub(crate) const RESET_RUNS: [(u32, Part); 5] = [
    (GEARBOX, Part::Gearbox),
    (BODY, Part::Body),
    (7, Part::Lights),
    (0xB, Part::Suspension),
    (0xD, Part::Brakes),
];

fn damage(car: &mut Player, item: u32) -> &mut u32 {
    &mut car.damage[item as usize - 2]
}

/// The order the repairs are shown in.
pub(crate) const REPAIR_ORDER: [u32; 14] =
    [2, 4, 6, 8, 0xA, 0xC, GEARBOX, 3, 5, 7, 9, 0xB, 0xD, TYRES];

/// Each step of the shown repair: this many milliseconds, 2 off the damage.
pub(crate) const STEP_MS: i32 = 5;
pub const STEP: i32 = 2;
/// The frames after each piece of work, in milliseconds.
pub(crate) const PAUSE_MS: i32 = 500;
/// The wait at the end, in milliseconds.
pub(crate) const END_SLEEP_MS: u32 = 1000;

/// The setup values the work installs (the chosen one made the installed one and the
/// car's), with their items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    GearRatio,
    Tyre,
    RideHeight,
    Stiffness,
}

impl Setting {
    /// Where the car keeps it.
    pub fn of(self, car: &mut Player) -> &mut u32 {
        match self {
            Setting::GearRatio => &mut car.gear_ratio,
            Setting::Tyre => &mut car.tyre,
            Setting::RideHeight => &mut car.ride_height,
            Setting::Stiffness => &mut car.suspension_stiffness,
        }
    }
}

/// The setup items installed after the repairs: (item, setting).
pub(crate) const SETUPS: [(u32, Setting); 2] =
    [(0x12, Setting::RideHeight), (0x13, Setting::Stiffness)];

/// The time-keeping variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    /// The clock when a piece of work started.
    Start,
    Elapsed,
    /// `Elapsed` at the last step.
    LastStep,
}

/// What the work is done with.
pub trait Workshop {
    type Error;
    fn flag(&mut self, item: u32) -> u32;
    fn set_flag(&mut self, item: u32, value: u32);
    /// The car (the current player's record).
    fn car(&mut self) -> Player;
    fn update_car(&mut self, change: impl FnOnce(&mut Player));
    /// `MenuState.menus.service_flag` = 1.
    fn set_working(&mut self);
    fn timer(&mut self, t: Timer) -> u32;
    fn set_timer(&mut self, t: Timer, value: u32);
    fn reset_tyre_wear(&mut self) -> Result<(), Self::Error>;
    /// The chosen value made the installed one; it is returned.
    fn install(&mut self, s: Setting) -> u32;
    fn clock(&mut self) -> u32;
    /// A frame of the service area.
    fn frame(&mut self);
    /// The wait at the end.
    fn sleep(&mut self, milliseconds: u32);
    /// Whether a key was pressed since the last asking (it is taken): the frames shown
    /// after a piece of work end early then.
    fn key_pressed(&mut self) -> bool {
        false
    }
}

/// Waits frames for `PAUSE_MS`, or until a key.
fn pause<W: Workshop>(w: &mut W) {
    let now = w.clock();
    w.set_timer(Timer::Start, now);
    loop {
        w.frame();
        let elapsed = w.clock().wrapping_sub(w.timer(Timer::Start));
        if elapsed as i32 >= PAUSE_MS || w.key_pressed() {
            return;
        }
    }
}

/// Applies the repairs the item flags ask for, as `service::apply_repairs`.
pub fn apply_repairs<W: Workshop>(w: &mut W) -> Result<(), W::Error> {
    w.set_working();
    for (item, part) in RESET_RUNS {
        if w.flag(item) == 1 {
            w.update_car(|car| car.clear_wear(part));
        }
    }
    if w.flag(TYRES) == 1 {
        let kept = *damage(&mut w.car(), TYRES);
        w.reset_tyre_wear()?;
        w.update_car(|car| *damage(car, TYRES) = kept);
    }

    for item in REPAIR_ORDER {
        if w.flag(item) != 1 {
            continue;
        }
        let now = w.clock();
        w.set_timer(Timer::Start, now);
        w.set_timer(Timer::LastStep, 0);
        loop {
            w.frame();
            let elapsed = w.clock().wrapping_sub(w.timer(Timer::Start));
            w.set_timer(Timer::Elapsed, elapsed);
            if (elapsed.wrapping_sub(w.timer(Timer::LastStep)) as i32) < STEP_MS {
                continue;
            }
            w.set_timer(Timer::LastStep, elapsed);
            let left = *damage(&mut w.car(), item) as i32;
            if left <= 0 || left - STEP <= 0 {
                break;
            }
            w.update_car(|car| *damage(car, item) = (left - STEP) as u32);
        }
        w.update_car(|car| *damage(car, item) = 0);
        w.set_flag(item, 0);
        let installed = match item {
            GEARBOX => Some(Setting::GearRatio),
            TYRES => Some(Setting::Tyre),
            _ => None,
        };
        if let Some(s) = installed {
            let value = w.install(s);
            w.update_car(|car| *s.of(car) = value);
        }
        pause(w);
    }

    for (item, s) in SETUPS {
        if w.flag(item) != 1 {
            continue;
        }
        w.set_flag(item, 0);
        let value = w.install(s);
        w.update_car(|car| *s.of(car) = value);
        pause(w);
    }

    w.update_car(|car| {
        if *damage(car, TYRES) == 0 {
            car.state_flags |= TYRE_BITS;
        }
        if *damage(car, BODY) == 0 {
            car.state_flags |= BODY_BITS;
            car.impact_marks = [0; 8];
        }
    });
    w.sleep(END_SLEEP_MS);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct Shop {
        flags: HashMap<u32, u32>,
        car: Option<Player>,
        timers: HashMap<u8, u32>,
        clock: u32,
        frames: u32,
        slept: u32,
        working: bool,
        installed: Vec<Setting>,
    }

    impl Workshop for Shop {
        type Error = ();
        fn flag(&mut self, item: u32) -> u32 {
            *self.flags.get(&item).unwrap_or(&0)
        }
        fn set_flag(&mut self, item: u32, value: u32) {
            self.flags.insert(item, value);
        }
        fn car(&mut self) -> Player {
            *self
                .car
                .get_or_insert_with(|| Player::from_le_bytes([0; 0x28c]))
        }
        fn update_car(&mut self, change: impl FnOnce(&mut Player)) {
            let mut car = self.car();
            change(&mut car);
            self.car = Some(car);
        }
        fn set_working(&mut self) {
            self.working = true;
        }
        fn timer(&mut self, t: Timer) -> u32 {
            *self.timers.get(&(t as u8)).unwrap_or(&0)
        }
        fn set_timer(&mut self, t: Timer, value: u32) {
            self.timers.insert(t as u8, value);
        }
        fn reset_tyre_wear(&mut self) -> Result<(), ()> {
            self.update_car(|car| *damage(car, TYRES) = 77);
            Ok(())
        }
        fn install(&mut self, s: Setting) -> u32 {
            self.installed.push(s);
            40 + s as u32
        }
        fn clock(&mut self) -> u32 {
            self.clock += 3;
            self.clock
        }
        fn frame(&mut self) {
            self.frames += 1;
        }
        fn sleep(&mut self, milliseconds: u32) {
            self.slept += milliseconds;
        }
    }

    #[test]
    fn work() {
        let mut s = Shop::default();
        for item in [3, 0xE, 0xF, 0x12] {
            s.flags.insert(item, 1);
        }
        s.flags.insert(5, 2);
        s.update_car(|car| {
            *damage(car, 3) = 7;
            *damage(car, 0xF) = 9;
            *damage(car, 5) = 9;
            car.parts_wear[0] = 5;
        });
        apply_repairs(&mut s).unwrap();
        assert!(s.working);
        let car = s.car();
        let at = |o: usize| car.word(o);
        assert_eq!(at(0x17C), 0);
        assert_eq!(at(0xCC + 12), 0);
        assert_eq!(at(0xCC + 4 * 5), 9);
        assert_eq!(s.flags[&5], 2);
        assert_eq!(
            s.installed,
            [Setting::GearRatio, Setting::Tyre, Setting::RideHeight]
        );
        assert_eq!(at(0x120), 40);
        assert_eq!(at(0x278), TYRE_BITS | BODY_BITS);
        assert_eq!(s.slept, END_SLEEP_MS);
        assert!(s.frames > 0);
    }
}
