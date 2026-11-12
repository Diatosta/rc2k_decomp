//! `physics::value_notice` (0x45B357): notice 0xCE by `turned_steps` against 300 (the
//! sign of the difference): at 300 or more, with no notice shown (`notice_shown` 0) and
//! 0xCE not the last (`notice_last`), `notice_id` = 0xCE and `notice_time` = 0;
//! below 300 with 0xCE the last, `notice_shown` and `notice_last` 0. What the figure is
//! is not known.
//!
//! Caller: `physics::car_physics_step` (0x43B9AE), its last call, every physics step.

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use rc2k_core::physics::notices::Notice;

// Converted: `rc2k_core::physics::notices::Notice::value_notice`.
pub fn value_notice(car: &Car) {
    let was = Notice {
        shown: car.notice_shown.get(),
        last: car.notice_last.get(),
        id: car.notice_id.get(),
        time: car.notice_time.get(),
        ..Notice::default()
    };
    let mut n = was;
    n.value_notice(car.turned_steps.get());
    set_changed(&car.notice_id, was.id, n.id);
    set_changed(&car.notice_time, was.time, n.time);
    set_changed(&car.notice_shown, was.shown, n.shown);
    set_changed(&car.notice_last, was.last, n.last);
}
