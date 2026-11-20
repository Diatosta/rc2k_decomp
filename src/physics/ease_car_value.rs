//! Unless `scene_switch` is set, `eased_value` follows `eased_target` << 8: set to it when
//! `value_jump_steps` is set, else stepped by 4 towards it. What the values are is not known.
//!
//! Returns `eased_target` << 8 (when `scene_switch` is 0).

use crate::physics::car_store::Car;
use crate::physics::cells::set_changed;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::easing::ease_value;

pub(super) fn ease_car_value(car: &Car, vars: &RaceVars) {
    let was = car.eased_value.get();
    let mut value = was;
    ease_value(
        vars.scene_switch.get(),
        car.value_jump_steps.get(),
        car.eased_target.get(),
        &mut value,
    );
    set_changed(&car.eased_value, was, value);
}
