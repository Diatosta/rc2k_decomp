//! `physics::take_notice` (0x4B1C56): when no notice is shown (`notice_shown` and
//! `notice_id` both 0) and the queue (`physics::queue_notice`) is not empty: the first
//! entry's id into `notice_id`, `notice_time_b` = 0x32, `notice_time` = 0x12C (by the
//! look of them, times to show it), the count less 1, and the three lists moved down one
//! (0x1F dwords each, `rep movsd`).
//!
//! Caller: `physics::car_physics_step` (0x43B961), every physics step, right after
//! `physics::damage_notices`.

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use crate::physics::queue_notice::{load, store};
use rc2k_core::physics::notices::Notice;

pub fn take_notice(car: &Car) {
    let was = Notice {
        id: car.notice_id.get(),
        shown: car.notice_shown.get(),
        time: car.notice_time.get(),
        w708: car.notice_time_b.get(),
        ..Notice::default()
    };
    let queue = load(car);
    let (mut notice, mut now) = (was, queue);
    if notice.take(&mut now).is_none() {
        return;
    }
    set_changed(&car.notice_id, was.id, notice.id);
    set_changed(&car.notice_time_b, was.w708, notice.w708);
    set_changed(&car.notice_time, was.time, notice.time);
    store(car, &queue, &now);
}
