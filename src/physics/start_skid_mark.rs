//! When either of the two qwords `wheel_squeeze`, `Car.wheel_squeeze[2]` is not 0 and
//! `camera_view_return` is: `camera_blend_view` = it less 1, `camera_switch_active` 1,
//! `camera_switch_share` 0, `camera_switch_length` 10, `camera_switch_clock` the step count
//! `step_clock`; then `camera_view_return` 0. This starts a skid mark (the start's place, a count
//! of 10, the step it began).

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use rc2k_core::physics::ground::SkidStart;

// Core: `rc2k_core::physics::ground::SkidStart::start`.
pub(super) fn start_skid_mark(car: &Car) {
    // The squeezes are tested as two qwords: any bit set counts (a -0.0 too).
    let wheels = car.wheel_squeeze.each_ref().map(|w| w.get().to_bits());
    let was = SkidStart {
        pending: car.camera_view_return.get(),
        from: car.camera_blend_view.get(),
        on: car.camera_switch_active.get(),
        switch_share: car.camera_switch_share.get().to_bits(),
        switch_length: car.camera_switch_length.get(),
        step: car.camera_switch_clock.get(),
    };
    let mut s = was;
    s.start(wheels, || car.step_clock.get());
    set_changed(&car.camera_blend_view, was.from, s.from);
    set_changed(&car.camera_switch_active, was.on, s.on);
    if s.switch_share != was.switch_share {
        car.camera_switch_share.set(f32::from_bits(s.switch_share));
    }
    set_changed(
        &car.camera_switch_length,
        was.switch_length,
        s.switch_length,
    );
    set_changed(&car.camera_switch_clock, was.step, s.step);
    set_changed(&car.camera_view_return, was.pending, s.pending);
}
