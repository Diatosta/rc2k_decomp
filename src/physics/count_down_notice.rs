//! While a notice is shown (`notice_shown` set) and its time `notice_time` is not 0, the time
//! counts down by 1; at 0 the notice is cleared. `physics::take_notice` sets them.
//!
//! Returns the time before (when `notice_shown` is set).

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use rc2k_core::physics::notices::Notice;

pub(super) fn count_down_notice(car: &Car) {
    let was = Notice {
        shown: car.notice_shown.get(),
        time: car.notice_time.get(),
        ..Notice::default()
    };
    let mut n = was;
    n.count_down();
    set_changed(&car.notice_time, was.time, n.time);
    set_changed(&car.notice_shown, was.shown, n.shown);
}
