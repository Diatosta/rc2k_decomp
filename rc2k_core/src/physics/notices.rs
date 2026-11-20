//! The car's notices: the one shown and the queue of up to 0x20 waiting, among them damage
//! warnings by the gauges' levels.

use crate::Invalid;

/// The notice shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Notice {
    /// The notice's id (0: none).
    pub id: u32,
    /// The notice's counter.
    pub w6f8: u32,
    /// The last notice shown.
    pub last: u32,
    /// Non-zero while a notice is shown.
    pub shown: u32,
    /// Its time, counted down while shown.
    pub time: u32,
    /// A second time (0x32 when taken from the queue).
    pub w708: u32,
}

/// The notice raised by `value_notice`.
pub(crate) const VALUE_NOTICE: u32 = 0xCE;
const VALUE_THRESHOLD: u32 = 300;

impl Notice {
    /// Notice 0xCE by a figure (signed) against 300: at or past it, with none shown
    /// and 0xCE not the last, it is set with time 0; below it with 0xCE the last, cleared.
    pub fn value_notice(&mut self, figure: u32) {
        if (figure as i32) >= VALUE_THRESHOLD as i32 {
            if self.shown == 0 && self.last != VALUE_NOTICE {
                self.id = VALUE_NOTICE;
                self.time = 0;
            }
        } else if self.last == VALUE_NOTICE {
            self.shown = 0;
            self.last = 0;
        }
    }

    /// Counts the shown notice's time down; at 0 it is no longer shown. Returns the time
    /// before, when one is shown.
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

    /// With none shown and the queue not empty, shows the first queued and moves the queue
    /// down one. Returns its id when one was taken.
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

/// The notices waiting: the count, then the ids, values and levels (0x20 each).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoticeQueue {
    pub count: u32,
    pub ids: [u32; 0x20],
    pub values: [u32; 0x20],
    pub levels: [u32; 0x20],
}

impl NoticeQueue {
    /// Queues notice `id` at `level` (1 .. 4) for `value`, when the car's flags have bit 2 and
    /// the queue is not full. With a level, an entry of the same value is kept to the larger
    /// level (unsigned); else a new entry. Returns the entry's index, `None` when nothing was
    /// done. A count past the room is refused.
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
mod tests;
