//! A replay of more than two cars is heard with every recorded car (each has a sound object of
//! its own). Watched in one view, the sounds are placed from the followed car, so the cars far
//! from it are quiet; in a split screen each car keeps its own place, as in the race.

use crate::physics::car_store::Car;
use crate::race::owner_shared::RaceShared;
use std::sync::atomic::Ordering::Relaxed;

/// Whether the cars of the replay now viewed are all heard (set with the sound objects).
pub fn set(shared: &RaceShared, on: bool) {
    shared.every_car_heard.store(on, Relaxed);
}

/// Whether every car of a viewed replay is heard.
pub fn is_on(shared: &RaceShared) -> bool {
    shared.every_car_heard.load(Relaxed)
}

/// The car whose camera `car`'s sounds are placed against: the followed car (the first view's)
/// while every car is heard in one view, else the car's own.
pub fn listener<'a>(shared: &'a RaceShared, car: &'a Car) -> &'a Car {
    if is_on(shared) && shared.view.get().views.get() == 1 {
        shared
            .car_blocks()
            .of(shared.view.get().view_blocks[0].get())
            .unwrap_or(car)
    } else {
        car
    }
}
