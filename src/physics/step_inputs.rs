//! What the contact, wall, ground-touch, camera and checkpoint steps read of the race, borrowed
//! once at the step's entry (`car_physics_step`) and handed down in place of the whole race.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_frame::Frame;
use crate::race::owner_physics::Physics;
use crate::race::owner_replay::Replay;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::RaceTables;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::work_store::RaceWork;

/// The race's pieces the contact steps touch.
#[derive(Clone, Copy)]
pub struct StepInputs<'a> {
    pub physics: &'a Physics,
    pub course: &'a Course,
    pub numbers: &'a Numbers,
    pub frame: &'a Frame,
    pub car: &'a Car,
    pub tables: &'a PhysicsTables,
    pub work: &'a RaceWork,
    pub shared: &'a RaceShared,
    pub vars: &'a RaceVars,
    pub view: &'a RaceView,
    pub race_tables: &'a RaceTables,
    /// Set while the replay steps to a still (`ReplayFrames::step_first`).
    pub step_first: u32,
    pub replay: &'a Replay,
}

impl<'a> StepInputs<'a> {
    pub fn of(race: &'a Race) -> Self {
        StepInputs {
            physics: &race.physics,
            course: &race.course,
            numbers: &race.course.numbers,
            frame: &race.frame,
            car: race.shared.car(),
            tables: race.shared.physics_tables.get(),
            work: race.shared.work.get(),
            shared: &race.shared,
            vars: race.shared.vars.get(),
            view: race.shared.view.get(),
            race_tables: race.shared.tables(),
            step_first: race.replay.replay_frames().step_first,
            replay: &race.replay,
        }
    }
}
