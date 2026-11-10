//! The race's owner (step 11, owned state): the race's state in one struct, in groups by
//! part, handed by reference to the code that runs a race in place of global stores.
//! `race()` is the boundary: code that does not take the owner yet fetches it there; the
//! climb from `drive::play_race` down passes `&Race` instead (`notes/memory_layer_plan.md`,
//! step 11, the race owner's design).

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

/// The one instance. Reached through `race()`.
pub static RACE: Race = Race {
    physics: Physics::new(),
    course: Course::new(),
    frame: Frame::new(),
    replay: Replay::new(),
    shared: RaceShared::new(),
};

/// The race's state, for a caller that does not take it as a parameter yet.
pub fn race() -> &'static Race {
    &RACE
}
