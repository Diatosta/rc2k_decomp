//! The work chosen in the service area carried out, on leaving it (converted 2026-10-21
//! from `service::apply_repairs` 0x46C14D): each chosen repair shown being done, a frame
//! at a time, then the chosen setup installed.
//!
//! A chosen item is one whose flag (`dword_759B6E`) is exactly 1. The car's dwords by
//! offset: +0xCC + 4 x item the parts' damage (the tyres' at +0x108), +0x10C the tyre,
//! +0x114 the ride height, +0x118 the stiffness, +0x120 the gear ratio, +0x17C .. +0x1F4
//! the parts' wear, +0x24C eight dwords of the body's state, +0x278 the state bits.

/// The car's damage of item 0 (+4 an item).
pub const DAMAGE: u32 = 0xCC;
pub const BODY: u32 = 3;
pub const GEARBOX: u32 = 0xE;
pub const TYRES: u32 = 0xF;
/// The car's state bits, and the body's eight dwords of state.
pub const STATE_BITS: u32 = 0x278;
pub const BODY_STATE: u32 = 0x24C;
/// The state bits a repaired part sets: the tyres', the body's (all the others).
pub const TYRE_BITS: u32 = 0x1E;
pub const BODY_BITS: u32 = 0xFFFF_FFE1;

/// (item, first offset in the car, dwords) zeroed before the repairs are shown.
pub const RESET_RUNS: [(u32, u32, u32); 5] = [
    (GEARBOX, 0x17C, 10),
    (BODY, 0x1A4, 6),
    (7, 0x1BC, 7),
    (0xB, 0x1D8, 5),
    (0xD, 0x1EC, 3),
];

/// The order the repairs are shown in.
pub const REPAIR_ORDER: [u32; 14] = [2, 4, 6, 8, 0xA, 0xC, GEARBOX, 3, 5, 7, 9, 0xB, 0xD, TYRES];

/// Each step of the shown repair: this many milliseconds, 2 off the damage.
pub const STEP_MS: i32 = 5;
pub const STEP: i32 = 2;
/// The frames after each piece of work, in milliseconds.
pub const PAUSE_MS: i32 = 500;
/// The wait at the end, in milliseconds.
pub const END_SLEEP_MS: u32 = 1000;

/// The setup values the work installs (the chosen one made the installed one and the
/// car's), with their items and the car's offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    GearRatio,
    Tyre,
    RideHeight,
    Stiffness,
}

impl Setting {
    /// Where the car keeps it.
    pub fn offset(self) -> u32 {
        match self {
            Setting::GearRatio => 0x120,
            Setting::Tyre => 0x10C,
            Setting::RideHeight => 0x114,
            Setting::Stiffness => 0x118,
        }
    }
}

/// The setup items installed after the repairs: (item, setting).
pub const SETUPS: [(u32, Setting); 2] = [(0x12, Setting::RideHeight), (0x13, Setting::Stiffness)];

/// The time-keeping variables (`dword_75AB48`, `dword_75AB4C`, `dword_75AB54`).
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
    /// An item's flag.
    fn flag(&mut self, item: u32) -> u32;
    fn set_flag(&mut self, item: u32, value: u32);
    /// The car's dword at `offset`.
    fn car(&mut self, offset: u32) -> u32;
    fn set_car(&mut self, offset: u32, value: u32);
    /// `dword_75AB38` = 1.
    fn set_working(&mut self);
    fn timer(&mut self, t: Timer) -> u32;
    fn set_timer(&mut self, t: Timer, value: u32);
    /// `service::reset_tyre_wear`.
    fn reset_tyre_wear(&mut self) -> Result<(), Self::Error>;
    /// The chosen value made the installed one; it is returned.
    fn install(&mut self, s: Setting) -> u32;
    fn clock(&mut self) -> u32;
    /// A frame of the service area.
    fn frame(&mut self);
    fn sleep(&mut self, milliseconds: u32);
}

/// Frames for `PAUSE_MS`.
fn pause<W: Workshop>(w: &mut W) {
    let now = w.clock();
    w.set_timer(Timer::Start, now);
    loop {
        w.frame();
        let elapsed = w.clock().wrapping_sub(w.timer(Timer::Start));
        if elapsed as i32 >= PAUSE_MS {
            return;
        }
    }
}

/// `service::apply_repairs`.
pub fn apply_repairs<W: Workshop>(w: &mut W) -> Result<(), W::Error> {
    w.set_working();
    for (item, offset, dwords) in RESET_RUNS {
        if w.flag(item) == 1 {
            for i in 0..dwords {
                w.set_car(offset + 4 * i, 0);
            }
        }
    }
    let tyre_damage = DAMAGE + 4 * TYRES;
    if w.flag(TYRES) == 1 {
        let kept = w.car(tyre_damage);
        w.reset_tyre_wear()?;
        w.set_car(tyre_damage, kept);
    }

    for item in REPAIR_ORDER {
        if w.flag(item) != 1 {
            continue;
        }
        let damage = DAMAGE + 4 * item;
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
            let left = w.car(damage) as i32;
            if left <= 0 || left - STEP <= 0 {
                break;
            }
            w.set_car(damage, (left - STEP) as u32);
        }
        w.set_car(damage, 0);
        w.set_flag(item, 0);
        let installed = match item {
            GEARBOX => Some(Setting::GearRatio),
            TYRES => Some(Setting::Tyre),
            _ => None,
        };
        if let Some(s) = installed {
            let value = w.install(s);
            w.set_car(s.offset(), value);
        }
        pause(w);
    }

    for (item, s) in SETUPS {
        if w.flag(item) != 1 {
            continue;
        }
        w.set_flag(item, 0);
        let value = w.install(s);
        w.set_car(s.offset(), value);
        pause(w);
    }

    if w.car(tyre_damage) == 0 {
        let bits = w.car(STATE_BITS);
        w.set_car(STATE_BITS, bits | TYRE_BITS);
    }
    if w.car(DAMAGE + 4 * BODY) == 0 {
        let bits = w.car(STATE_BITS);
        w.set_car(STATE_BITS, bits | BODY_BITS);
        for i in 0..8 {
            w.set_car(BODY_STATE + 4 * i, 0);
        }
    }
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
        car: HashMap<u32, u32>,
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
        fn car(&mut self, offset: u32) -> u32 {
            *self.car.get(&offset).unwrap_or(&0)
        }
        fn set_car(&mut self, offset: u32, value: u32) {
            self.car.insert(offset, value);
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
            self.car.insert(DAMAGE + 4 * TYRES, 77);
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
        s.car.insert(DAMAGE + 4 * 3, 7);
        s.car.insert(DAMAGE + 4 * 0xF, 9);
        s.car.insert(DAMAGE + 4 * 5, 9);
        s.car.insert(0x17C, 5);
        apply_repairs(&mut s).unwrap();
        assert!(s.working);
        assert_eq!(s.car[&0x17C], 0);
        assert_eq!(s.car[&(DAMAGE + 12)], 0);
        assert_eq!(s.car[&(DAMAGE + 4 * 5)], 9);
        assert_eq!(s.flags[&5], 2);
        assert_eq!(
            s.installed,
            [Setting::GearRatio, Setting::Tyre, Setting::RideHeight]
        );
        assert_eq!(s.car[&0x120], 40);
        assert_eq!(s.car[&STATE_BITS], TYRE_BITS | BODY_BITS);
        assert_eq!(s.slept, END_SLEEP_MS);
        assert!(s.frames > 0);
    }
}
