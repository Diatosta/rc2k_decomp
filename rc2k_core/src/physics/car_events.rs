//! The work car's timed events: the queue of up to 7 pace-note symbols stacked for a view,
//! aged against the scene clock, and the table of 0x20 listed events.

use crate::Invalid;

pub const ROOM: u32 = 7;
/// How long an event stays (3000 of the scene clock).
pub const LIFE: u32 = 3000;

/// The queue; the times and events have an eighth slot each, which the shift reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CarEvents {
    /// When each is due.
    pub times: [u32; 8],
    pub events: [u32; 8],
    pub count: u32,
    /// The scroll target: 0 for the first entry, 0x20 less for each further one, 0x20 more
    /// for each expired.
    pub level: u32,
    /// The scroll position drawn, eased 4 a step towards `level`.
    pub shown: u32,
}

/// What `expire` did: whether anything was shifted out, and `shown` after.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expired {
    pub shifted: bool,
    pub shown: u32,
}

impl CarEvents {
    fn check(&self) -> Result<(), Invalid> {
        if self.count > ROOM {
            return Err(Invalid("a car event count above 7"));
        }
        Ok(())
    }

    /// Queues `event` due `delay` after `clock` unless the queue is full; the first resets
    /// `level` and `shown`, each further one takes 0x20 off `level`. Returns the time.
    pub fn queue(&mut self, clock: u32, delay: u32, event: u32) -> Result<u32, Invalid> {
        self.check()?;
        let time = clock.wrapping_add(delay);
        let n = self.count;
        if n == ROOM {
            return Ok(time);
        }
        self.times[n as usize] = time;
        self.events[n as usize] = event;
        self.count = n + 1;
        if n == 0 {
            self.level = 0;
            self.shown = 0;
        } else {
            self.level = self.level.wrapping_sub(0x20);
        }
        Ok(time)
    }

    /// Drops each first event 3000 or more behind `clock` (by sign): the lists shift down,
    /// `level` goes up 0x20 and `shown` 0x40. Then `shown` comes 4 towards `level`, not past it.
    pub fn expire(&mut self, clock: u32) -> Result<Expired, Invalid> {
        self.check()?;
        let mut shifted = false;
        while self.count != 0 {
            if (clock.wrapping_sub(self.times[0]).wrapping_sub(LIFE) as i32) < 0 {
                break;
            }
            self.times.copy_within(1.., 0);
            self.events.copy_within(1.., 0);
            self.count -= 1;
            self.level = self.level.wrapping_add(0x20);
            self.shown = self.shown.wrapping_add(0x40);
            shifted = true;
        }
        let target = self.level;
        let mut s = self.shown;
        if s != target && (s.wrapping_sub(target) as i32) >= 0 {
            s = s.wrapping_sub(4);
            if (s.wrapping_sub(target) as i32) < 0 {
                s = target;
            }
        }
        self.shown = s;
        Ok(Expired { shifted, shown: s })
    }
}

pub const LISTED_EVENTS: u32 = 0x20;
/// No event.
pub const NO_EVENT: u32 = u32::MAX;

/// For a number in the table, its entry and whether it is queued (not -1); None past the table.
pub fn listed_event(n: u32, entry: impl FnOnce(u32) -> u32) -> Option<(u32, bool)> {
    (n < LISTED_EVENTS).then(|| {
        let e = entry(n);
        (e, e != NO_EVENT)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_and_expired() {
        let mut q = CarEvents {
            level: 9,
            shown: 9,
            ..Default::default()
        };
        assert_eq!(q.queue(100, 5, 0xA), Ok(105));
        assert_eq!((q.count, q.level, q.shown), (1, 0, 0));
        assert_eq!(q.queue(200, 5, 0xB), Ok(205));
        assert_eq!((q.count, q.level), (2, 0u32.wrapping_sub(0x20)));
        assert_eq!(
            q.expire(3104),
            Ok(Expired {
                shifted: false,
                shown: 0u32.wrapping_sub(4)
            })
        );
        assert_eq!(
            q.expire(3105),
            Ok(Expired {
                shifted: true,
                shown: 0x38
            })
        );
        assert_eq!(
            (q.count, q.events[0], q.times[0], q.level),
            (1, 0xB, 205, 0)
        );
        q.count = ROOM;
        assert_eq!(q.queue(0, 1, 1), Ok(1));
        assert_eq!(q.count, ROOM);
        q.count = 8;
        assert!(q.queue(0, 1, 1).is_err());
        assert!(q.expire(0).is_err());
    }

    #[test]
    fn listed_events() {
        assert_eq!(listed_event(0x20, |_| unreachable!()), None);
        assert_eq!(listed_event(3, |n| n * 2), Some((6, true)));
        assert_eq!(listed_event(0x1F, |_| NO_EVENT), Some((NO_EVENT, false)));
    }
}
