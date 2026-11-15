//! The race's owner: the race's state in one struct, in groups by part, handed by reference to the
//! code that runs a race. `race()` is the boundary: code that does not take the owner yet fetches
//! it there; the climb from `drive::play_race` down passes `&Race` instead.

use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::owner_physics::Physics;
use crate::race::owner_replay::Replay;
use crate::race::owner_shared::RaceShared;

/// The race's state.
pub struct Race {
    pub physics: Physics,
    pub course: Course,
    pub frame: Frame,
    pub replay: Replay,
    /// The stores the menus, the network and the audio side read too.
    pub shared: RaceShared,
}

impl Race {
    /// A race with every store in its start-up state.
    pub const fn new() -> Self {
        Race {
            physics: Physics::new(),
            course: Course::new(),
            frame: Frame::new(),
            replay: Replay::new(),
            shared: RaceShared::new(),
        }
    }
}

/// The one instance. Reached through `race()`.
pub static RACE: Race = Race::new();

/// The race's state, for a caller that does not take it as a parameter yet.
pub fn race() -> &'static Race {
    &RACE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_race_holds_its_own_car_blocks() {
        // The value is large: it is built and kept on the heap by a thread with room for it.
        let size = std::mem::size_of::<Race>();
        let built = std::thread::Builder::new()
            .stack_size(size * 3 + (1 << 20))
            .spawn(|| {
                let second = Box::new(Race::new());
                let first_car = &second.shared.car_blocks().cars[0];
                assert!(!std::ptr::eq(
                    first_car,
                    core::ptr::from_ref(&RACE.shared.car_blocks().cars[0])
                ));
                first_car.step_clock.set(0x5EC0_4D01);
                assert_ne!(
                    RACE.shared.car_blocks().cars[0].step_clock.get(),
                    0x5EC0_4D01
                );
                assert!(std::ptr::eq(
                    second.car_blocks(),
                    second.shared.car_blocks()
                ));
            })
            .unwrap();
        built.join().unwrap();
    }
}
