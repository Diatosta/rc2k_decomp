//! The work car's start (from `physics::count_start_second`
//! 0x436AC4, `mark_car_started` 0x436AA7, `false_start` 0x43B9B4, `count_down_start`
//! 0x436AFB and `reset_when_stuck` 0x43EFBA): the time counted once the start is behind,
//! the car marked as got going, a jump start's penalty, the start's count down and the
//! stuck car put back.

use super::notices::Notice;

/// The work copy's start fields (`g_car` +0x14 ..; what they are is a reading).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarStart {
    /// +0x14 (`run_ticks`): the start timer.
    pub timer: u32,
    /// +0x18 (`start_in_gear`): set once the car is in gear.
    pub held: u32,
    /// +0x1C (`false_start_count`): the jump start's count down.
    pub count: u32,
    /// +0x88 (`stage_time`): the car's time.
    pub time: u32,
    /// +0x8C (`time_left`): the start's count down.
    pub countdown: u32,
    /// +0xA0 (`started_stand_timer`): the car has got going.
    pub started: u32,
    /// +0xA8 (`finish_flag`): finished.
    pub finished: u32,
    /// +0x26C (`tyre_change_state`): the grades' mode (0 or -1 while racing).
    pub state: u32,
    /// +0x29C (`damage_failed`).
    pub moving: u32,
}

/// When the start timer is past the start (0x320).
const START: u32 = 0x320;
/// The jump start's count down (500) and its penalty (1000 on the time).
const PENALTY_COUNT: u32 = 500;
const PENALTY: u32 = 1000;
/// The notices shown: the jump start, and the car put back.
pub const JUMP_START_NOTICE: u32 = 0xCC;
pub const PUT_BACK_NOTICE: u32 = 0xCA;

/// What `false_start` did: the timer when it was compared and
/// whether the car is to be put back at its start (`race::start_car`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FalseStart {
    pub timer: Option<u32>,
    pub put_back: bool,
}

/// What `count_down_start` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartCount {
    /// Finished: nothing.
    Stopped,
    /// The timer less 0x320: before the start, or no count left.
    Waiting(u32),
    /// The count down went one down: its hundreds and the rest (`div` by 100), and
    /// whether a beep is due (a whole hundred, 8 or less).
    Counted {
        hundreds: u32,
        rest: u32,
        beep: bool,
    },
}

/// The stuck count at which the car is put back on the line.
pub const STUCK_LIMIT: u32 = 700;

/// `physics::reset_when_stuck`: with the grades' mode (`driver_kind`) 0 or 2 the stuck
/// count (`g_car` +0x1E8) is 0; else it goes up, and at 700 or more (unsigned) the car is
/// to be put back on the line (`physics::reset_car`).
pub fn stuck_tick(mode: u32, stuck: &mut u32) -> bool {
    if mode == 2 || mode == 0 {
        *stuck = 0;
        return false;
    }
    *stuck = stuck.wrapping_add(1);
    *stuck >= STUCK_LIMIT
}

impl CarStart {
    /// `physics::count_down_start`: unless finished, once the timer is past the start
    /// (by the sign of the difference) the count down, when not 0, goes one down.
    pub fn count_down_start(&mut self) -> StartCount {
        if self.finished != 0 {
            return StartCount::Stopped;
        }
        let past = self.timer.wrapping_sub(START);
        if (past as i32) < 0 || self.countdown == 0 {
            return StartCount::Waiting(past);
        }
        self.countdown -= 1;
        let (hundreds, rest) = (self.countdown / 100, self.countdown % 100);
        StartCount::Counted {
            hundreds,
            rest,
            beep: hundreds <= 8 && rest == 0,
        }
    }

    /// `physics::count_start_second`: with the state 0 or -1 and no count down, the
    /// timer less 0x320; when that is not below 0 (by its sign) and
    /// the car is not finished, the time up by one.
    pub fn count_start_second(&mut self) -> Option<u32> {
        if (self.state != u32::MAX && self.state != 0) || self.count != 0 {
            return None;
        }
        let past = self.timer.wrapping_sub(START);
        if (past as i32) >= 0 && self.finished == 0 {
            self.time = self.time.wrapping_add(1);
        }
        Some(past)
    }

    /// `physics::mark_car_started`: started once +0x29C is set.
    pub fn mark_started(&mut self) {
        if self.started == 0 && self.moving != 0 {
            self.started = 1;
        }
    }

    /// `physics::false_start`: `held` set once in gear (`gear` not 0); held with no count
    /// down and the timer before the start (by the sign of the difference) starts the
    /// count down at 500 with the jump start's notice; each call the count goes down, and
    /// on reaching 0 the time takes the penalty, the timer and `held` go, the notice says
    /// the car is put back.
    pub fn false_start(&mut self, gear: u32, notice: &mut Notice) -> FalseStart {
        let mut done = FalseStart::default();
        if self.held == 0 && gear != 0 {
            self.held = 1;
        }
        if self.held != 0 && self.count == 0 {
            done.timer = Some(self.timer);
            if (self.timer.wrapping_sub(START) as i32) < 0 {
                self.count = PENALTY_COUNT;
                notice.id = JUMP_START_NOTICE;
                notice.w708 = 0;
                notice.time = u32::MAX;
            }
        }
        if self.count != 0 {
            self.count = self.count.wrapping_sub(1);
            if self.count == 0 {
                self.time = self.time.wrapping_add(PENALTY);
                self.timer = 0;
                self.held = 0;
                notice.w708 = 0;
                notice.time = 300;
                notice.last = 0;
                notice.id = PUT_BACK_NOTICE;
                notice.w6f8 = 0;
                notice.shown = 0;
                done.put_back = true;
            }
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counted_and_started() {
        let mut c = CarStart {
            timer: START,
            ..Default::default()
        };
        assert_eq!(c.count_start_second(), Some(0));
        assert_eq!(c.time, 1);
        c.finished = 1;
        assert_eq!(c.count_start_second(), Some(0));
        assert_eq!(c.time, 1);
        c.timer = 0;
        c.finished = 0;
        assert_eq!(c.count_start_second(), Some(START.wrapping_neg()));
        assert_eq!(c.time, 1);
        c.state = 3;
        assert_eq!(c.count_start_second(), None);
        c.mark_started();
        assert_eq!(c.started, 0);
        c.moving = 1;
        c.mark_started();
        assert_eq!(c.started, 1);
    }

    #[test]
    fn count_down() {
        let mut c = CarStart {
            timer: START,
            countdown: 801,
            ..Default::default()
        };
        assert_eq!(
            c.count_down_start(),
            StartCount::Counted {
                hundreds: 8,
                rest: 0,
                beep: true
            }
        );
        assert_eq!(
            c.count_down_start(),
            StartCount::Counted {
                hundreds: 7,
                rest: 99,
                beep: false
            }
        );
        c.countdown = 0;
        assert_eq!(c.count_down_start(), StartCount::Waiting(0));
        c.finished = 1;
        assert_eq!(c.count_down_start(), StartCount::Stopped);
        let mut s = 698;
        assert!(!stuck_tick(1, &mut s));
        assert!(stuck_tick(3, &mut s));
        assert!(!stuck_tick(2, &mut s));
        assert_eq!(s, 0);
    }

    #[test]
    fn jump_start() {
        let mut c = CarStart {
            timer: 5,
            ..Default::default()
        };
        let mut n = Notice::default();
        let d = c.false_start(2, &mut n);
        assert_eq!(
            d,
            FalseStart {
                timer: Some(5),
                put_back: false
            }
        );
        assert_eq!(
            (c.held, c.count, n.id, n.time),
            (1, 499, JUMP_START_NOTICE, u32::MAX)
        );
        c.count = 1;
        let d = c.false_start(2, &mut n);
        assert_eq!(
            d,
            FalseStart {
                timer: None,
                put_back: true
            }
        );
        assert_eq!(
            (c.held, c.timer, c.time, n.id, n.time),
            (0, 0, PENALTY, PUT_BACK_NOTICE, 300)
        );
        let mut late = CarStart {
            timer: START,
            held: 1,
            ..Default::default()
        };
        let d = late.false_start(0, &mut n);
        assert_eq!(
            d,
            FalseStart {
                timer: Some(START),
                put_back: false
            }
        );
        assert_eq!(late.count, 0);
    }
}
