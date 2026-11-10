//! `physics::take_notice` (0x4B1C56): when no notice is shown (`dword_71A700` and
//! `dword_71A6F4` both 0) and the queue (`physics::queue_notice`) is not empty: the first
//! entry's id into `dword_71A6F4`, `dword_71A708` = 0x32, `dword_71A704` = 0x12C (by the
//! look of them, times to show it), the count less 1, and the three lists moved down one
//! (0x1F dwords each, `rep movsd`).
//!
//! Caller: `physics::car_physics_step` (0x43B961), every physics step, right after
//! `physics::damage_notices`.

use crate::physics::cells::set_changed;
use crate::physics::queue_notice::{load, store};
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::notices::Notice;

pub fn take_notice(shared: &RaceShared) {
    let was = Notice {
        id: shared.car.notice_id.get(),
        shown: shared.car.notice_shown.get(),
        time: shared.car.notice_time.get(),
        w708: shared.car.notice_time_b.get(),
        ..Notice::default()
    };
    let queue = load(shared);
    let (mut notice, mut now) = (was, queue);
    if notice.take(&mut now).is_none() {
        return;
    }
    set_changed(&shared.car.notice_id, was.id, notice.id);
    set_changed(&shared.car.notice_time_b, was.w708, notice.w708);
    set_changed(&shared.car.notice_time, was.time, notice.time);
    store(shared, &queue, &now);
}
