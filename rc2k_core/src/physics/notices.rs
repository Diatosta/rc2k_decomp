//! The car's notices (converted 2026-10-21 from `physics::queue_notice` 0x4B1BD0,
//! `take_notice` 0x4B1C56, `count_down_notice` 0x440F08 and `value_notice` 0x45B357):
//! the one shown (the work copy's +0x71C .. +0x730) and the queue of up to 0x20 waiting
//! (+0x428 ..: damage notices by the look of their levels, a reading).

use crate::Invalid;

/// The notice shown (`dword_71A6F4` .. `dword_71A708`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Notice {
    /// `dword_71A6F4`: the notice's id (0: none).
    pub id: u32,
    /// `dword_71A6F8`.
    pub w6f8: u32,
    /// `dword_71A6FC`: the last one shown.
    pub last: u32,
    /// `dword_71A700`: one is shown.
    pub shown: u32,
    /// `dword_71A704`: its time, counted down while shown.
    pub time: u32,
    /// `dword_71A708`: a second time (0x32 when taken from the queue).
    pub w708: u32,
}

/// `value_notice`'s notice and threshold.
pub const VALUE_NOTICE: u32 = 0xCE;
const VALUE_THRESHOLD: u32 = 300;

impl Notice {
    /// `physics::value_notice`: notice 0xCE by a figure (`dword_71A24C`) against 300 (by
    /// the sign of the difference): at or past it, with none shown and 0xCE not the last,
    /// 0xCE with time 0; below it with 0xCE the last, neither shown nor last.
    pub fn value_notice(&mut self, figure: u32) {
        if (figure.wrapping_sub(VALUE_THRESHOLD) as i32) >= 0 {
            if self.shown == 0 && self.last != VALUE_NOTICE {
                self.id = VALUE_NOTICE;
                self.time = 0;
            }
        } else if self.last == VALUE_NOTICE {
            self.shown = 0;
            self.last = 0;
        }
    }

    /// `physics::count_down_notice`: while one is shown and its time not 0, the time less
    /// 1, at 0 no longer shown. The time before (the original's eax) when one is shown.
    pub fn count_down(&mut self) -> Option<u32> {
        if self.shown == 0 {
            return None;
        }
        let before = self.time;
        if before != 0 {
            self.time = before - 1;
            if self.time == 0 {
                self.shown = 0;
            }
        }
        Some(before)
    }

    /// `physics::take_notice`: with none shown (`shown` and `id` both 0) and the queue not
    /// empty, the first queued is shown (times 0x32 and 0x12C) and the queue moves down
    /// one. Its id when one was taken.
    pub fn take(&mut self, queue: &mut NoticeQueue) -> Option<u32> {
        if self.shown != 0 || self.id != 0 || queue.count == 0 {
            return None;
        }
        let id = queue.ids[0];
        self.id = id;
        self.w708 = 0x32;
        self.time = 0x12C;
        queue.count = queue.count.wrapping_sub(1);
        for list in [&mut queue.ids, &mut queue.values, &mut queue.levels] {
            list.copy_within(1.., 0);
        }
        Some(id)
    }
}

/// The queue's room.
pub const QUEUE_ROOM: u32 = 0x20;

/// The notices waiting (`dword_71A450` the count, then three lists of 0x20 dwords: the
/// ids at `dword_71A454`, the values at `dword_71A4D4`, the levels at `dword_71A554`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoticeQueue {
    pub count: u32,
    pub ids: [u32; 0x20],
    pub values: [u32; 0x20],
    pub levels: [u32; 0x20],
}

impl NoticeQueue {
    /// `physics::queue_notice`: notice `id` at `level` (1 .. 4) for `value`, when the car's
    /// flags (`dword_71A1F8`) have bit 2 and the queue is not full. With a level, an entry
    /// of the same value and a level is kept to the larger level (unsigned, its id with
    /// it); else a new entry. The entry's index (the original's edx), `None` when nothing
    /// was done. A count past the room is refused (the original would write past the
    /// lists).
    pub fn queue(
        &mut self,
        flags: u32,
        id: u32,
        level: u32,
        value: u32,
    ) -> Result<Option<u32>, Invalid> {
        let n = self.count;
        if flags & 4 == 0 || n == QUEUE_ROOM {
            return Ok(None);
        }
        if n > QUEUE_ROOM {
            return Err(Invalid("a notice queue of more than 0x20 entries"));
        }
        if level != 0 {
            for k in 0..n as usize {
                let old = self.levels[k];
                if old != 0 && self.values[k] == value {
                    if level > old {
                        self.ids[k] = id;
                        self.levels[k] = level;
                    }
                    return Ok(Some(k as u32));
                }
            }
        }
        let k = n as usize;
        self.ids[k] = id;
        self.values[k] = value;
        self.levels[k] = level;
        self.count = n + 1;
        Ok(Some(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_and_count_down() {
        let mut n = Notice::default();
        n.value_notice(300);
        assert_eq!((n.id, n.time), (VALUE_NOTICE, 0));
        let mut m = Notice {
            shown: 1,
            last: VALUE_NOTICE,
            ..Default::default()
        };
        m.value_notice(299);
        assert_eq!((m.shown, m.last), (0, 0));
        let mut big = Notice::default();
        big.value_notice(0x8000_0000);
        assert_eq!(big.id, VALUE_NOTICE); // the difference's sign wraps
        let mut c = Notice {
            shown: 1,
            time: 2,
            ..Default::default()
        };
        assert_eq!(c.count_down(), Some(2));
        assert_eq!(c.count_down(), Some(1));
        assert_eq!((c.shown, c.time), (0, 0));
        assert_eq!(c.count_down(), None);
        let mut z = Notice {
            shown: 1,
            ..Default::default()
        };
        assert_eq!(z.count_down(), Some(0));
        assert_eq!(z.shown, 1);
    }

    #[test]
    fn queue_and_take() {
        let mut q = NoticeQueue::default();
        assert_eq!(q.queue(0, 1, 1, 1), Ok(None));
        assert_eq!(q.queue(4, 0xCF, 1, 7), Ok(Some(0)));
        assert_eq!(q.queue(4, 0xD0, 3, 7), Ok(Some(0)));
        assert_eq!((q.ids[0], q.levels[0], q.count), (0xD0, 3, 1));
        assert_eq!(q.queue(4, 0xD1, 2, 7), Ok(Some(0)));
        assert_eq!(q.ids[0], 0xD0);
        assert_eq!(q.queue(4, 0xD2, 0, 7), Ok(Some(1)));
        let mut n = Notice::default();
        assert_eq!(n.take(&mut q), Some(0xD0));
        assert_eq!(
            (n.id, n.time, n.w708, q.count, q.ids[0]),
            (0xD0, 0x12C, 0x32, 1, 0xD2)
        );
        assert_eq!(n.take(&mut q), None);
        q.count = QUEUE_ROOM;
        assert_eq!(q.queue(4, 1, 1, 1), Ok(None));
        q.count = QUEUE_ROOM + 1;
        assert!(q.queue(4, 1, 1, 1).is_err());
    }
}
