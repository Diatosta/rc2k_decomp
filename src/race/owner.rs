//! The race's owner: the race's state in one struct, in groups by part, handed by reference to the
//! code that runs a race.

use crate::effects::owner::Effects;
use crate::hud::owner::Hud;
use crate::memory::session::GameThread;
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
    /// The HUD's variables, drawn over the race.
    pub hud: GameThread<Hud>,
    /// The effects (marks, rain, snow, windscreen drops, particles) drawn over the race.
    pub effects: GameThread<Effects>,
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
            hud: GameThread::new(Hud::new()),
            effects: GameThread::new(Effects::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_race_holds_its_own_car_blocks() {
        // The value is large: it is built and kept on the heap by a thread with room for it.
        let first = crate::test_support::own::race();
        let first_car = core::ptr::from_ref(&first.shared.car_blocks().cars[0]).addr();
        let second =
            crate::owners::on_big_stack(3 * size_of::<Race>() + (1 << 20), || -> &'static Race {
                Box::leak(Box::new(Race::new()))
            });
        let second_car = &second.shared.car_blocks().cars[0];
        assert_ne!(core::ptr::from_ref(second_car).addr(), first_car);
        second_car.step_clock.set(0x5EC0_4D01);
        assert_ne!(
            first.shared.car_blocks().cars[0].step_clock.get(),
            0x5EC0_4D01
        );
        assert!(std::ptr::eq(
            second.car_blocks(),
            second.shared.car_blocks()
        ));
    }
}
