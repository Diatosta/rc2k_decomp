//! `physics::value_notice` (0x45B357): notice 0xCE by `dword_71A24C` against 300 (the
//! sign of the difference): at 300 or more, with no notice shown (`dword_71A700` 0) and
//! 0xCE not the last (`dword_71A6FC`), `dword_71A6F4` = 0xCE and `dword_71A704` = 0;
//! below 300 with 0xCE the last, `dword_71A700` and `dword_71A6FC` 0. What the figure is
//! is not known.
//!
//! Caller: `physics::car_physics_step` (0x43B9AE), its last call, every physics step.

use crate::physics::car_store::Car;
use crate::session::Cell32;
use rc2k_core::physics::notices::Notice;

// Converted: `rc2k_core::physics::notices::Notice::value_notice`.
fn core(v: &[&dyn Cell32]) {
    let was = Notice {
        shown: v[1].bits(),
        last: v[2].bits(),
        id: v[3].bits(),
        time: v[4].bits(),
        ..Notice::default()
    };
    let mut n = was;
    n.value_notice(v[0].bits());
    crate::physics::cells::set_changed(v[3], was.id, n.id);
    crate::physics::cells::set_changed(v[4], was.time, n.time);
    crate::physics::cells::set_changed(v[1], was.shown, n.shown);
    crate::physics::cells::set_changed(v[2], was.last, n.last);
}

fn game(car: &Car) -> Vec<&dyn Cell32> {
    vec![
        &car.turned_steps as &dyn Cell32,
        &car.notice_shown,
        &car.notice_last,
        &car.notice_id,
        &car.notice_time,
    ]
}

pub fn value_notice(car: &Car) {
    core(&game(car));
}
