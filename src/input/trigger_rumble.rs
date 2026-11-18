//! The car's feel handed to a pad's trigger motors: each time the race brings its force feedback
//! up to date, the pedals, the driven wheels' slip and the engine's revs of the device's car
//! become the two triggers' levels (`rc2k_core::input::triggers`), at the strength the device is
//! set to. Switched on or off by the advanced page (`rc2k.toml`). A device with no motors under
//! its triggers ignores them; with the switch off nothing is sent at all.

use crate::input::di_objects;
use crate::input::types::InputDevice;
use crate::physics::car_store::Car;
use rc2k_core::input::triggers::{Feel, Levels, levels};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static ON: AtomicBool = AtomicBool::new(true);

/// Whether the triggers rumble (the advanced page's switch).
pub fn on() -> bool {
    ON.load(Relaxed)
}

pub fn set_on(on: bool) {
    ON.store(on, Relaxed);
}

/// What the car does, as the triggers read it.
fn feel_of(car: &Car) -> Feel {
    let limit = car.engine_speed_limit.get();
    Feel {
        throttle: car.throttle.get(),
        brake: car.brake.get(),
        slip: car.drive_speed_a.get() - car.local_velocity[2].get(),
        forward: car.local_velocity[2].get(),
        revs: if limit > 0.0 {
            car.engine_speed.get() / limit
        } else {
            0.0
        },
        airborne: car.driven_wheels_off.is_set()
            || car.front_wheels_off.is_set()
            || car.rear_wheels_off.is_set(),
    }
}

/// The device's triggers set from its car, when the race runs and the switch is on.
pub fn feed(device: &InputDevice, car: &Car, race_on: bool) {
    if !on() || !race_on {
        return;
    }
    // The device's strength is a percent; the motors take it in hundredths of a percent.
    let gain = device.strength.get().min(100) * 100;
    let target: Levels = levels(&feel_of(car), gain);
    di_objects::set_triggers(device.device.get(), target);
}
