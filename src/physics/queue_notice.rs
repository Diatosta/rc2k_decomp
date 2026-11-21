//! Queues a damage notice (`notice` id, 0xCF on; `rank`, its level 1 .. 4; `value`, a gauge value)
//! in the work car's queue of up to 0x20 entries (`notice_count`, `notice_ids`, `notice_values`,
//! `notice_ranks`), when `option_bits` has bit 2 and the queue is not full. With a rank other than
//! 0, an entry of the same value and a non-zero rank is kept at the largest rank (its id with it),
//! unsigned; otherwise (rank 0, or no such entry) a new entry is added. `physics::take_notice`
//! takes entries off; `race::start_car` empties the queue. The only caller is
//! `physics::damage_notices` (76 calls), run every physics step from `physics::car_physics_step`;
//! each call is behind a state variable whose value passes a threshold of a car figure, so a call
//! comes only when such a figure rises to a new level: damage warnings. Notices are the messages
//! shown for a car (GO, reset, finish, false start, the quit question, and these warnings: the id
//! picks the text, `notice_id`). A stage driven cleanly gives none.
//!
//! Refuses (before writing) a count above 0x20.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::cells::{set_changed, set_values_changed, values};
use rc2k_core::physics::notices::NoticeQueue;

/// The work car's notice queue.
pub fn load(car: &Car) -> NoticeQueue {
    NoticeQueue {
        count: car.notice_count.get(),
        ids: values(&car.notice_ids),
        values: values(&car.notice_values),
        levels: values(&car.notice_ranks),
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(car: &Car, was: &NoticeQueue, now: &NoticeQueue) {
    set_values_changed(&car.notice_ids, &was.ids, &now.ids);
    set_values_changed(&car.notice_values, &was.values, &now.values);
    set_values_changed(&car.notice_ranks, &was.levels, &now.levels);
    set_changed(&car.notice_count, was.count, now.count);
}

pub(super) fn queue_notice(car: &Car, notice: u32, rank: u32, value: u32) -> Result<(), Refused> {
    let was = load(car);
    let mut q = was;
    q.queue(car.option_bits.get(), notice, rank, value)?;
    store(car, &was, &q);
    Ok(())
}
