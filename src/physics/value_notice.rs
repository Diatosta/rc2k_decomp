//! Notice 0xCE by `turned_steps` against 300: at 300 or more, with no notice shown (`notice_shown`
//! 0) and 0xCE not the last (`notice_last`), `notice_id` = 0xCE and `notice_time` = 0; below 300
//! with 0xCE the last, `notice_shown` and `notice_last` = 0. What the figure is is not known.

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use rc2k_core::physics::notices::Notice;

// Core: `rc2k_core::physics::notices::Notice::value_notice`.
pub fn value_notice(car: &mut Car) {
    let was = Notice {
        shown: car.notice_shown,
        last: car.notice_last,
        id: car.notice_id,
        time: car.notice_time,
        ..Notice::default()
    };
    let mut n = was;
    n.value_notice(car.turned_steps);
    set_changed(&mut car.notice_id, was.id, n.id);
    set_changed(&mut car.notice_time, was.time, n.time);
    set_changed(&mut car.notice_shown, was.shown, n.shown);
    set_changed(&mut car.notice_last, was.last, n.last);
}
