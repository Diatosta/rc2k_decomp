//! The race owner's shared group (`race::owner::Race::shared`): the race's stores the menus, the network and the audio side read too.

use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::buffers_store::Buffers;
use crate::race::replay::Replays;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::replay_path::Templates;
use crate::race::state_store::RaceState;
use crate::race::switches_store::FrameSwitches;
use crate::race::tables_store::RaceTables;
use crate::race::tyre_store::TyreChange;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::work_store::RaceWork;

/// The race's stores the menus, the network and the audio side read too.
pub struct Shared {
    /// `RACE_WORK` (`race`).
    /// One copy per drawing thread (`workers`).
    pub work: crate::workers::PerWorker<RaceWork>,
    /// `RACE_STATE` (`race`).
    pub state: RaceState,
    /// `RACE_TABLES` (`race`).
    pub tables: RaceTables,
    /// `BUFFERS` (`race`).
    pub buffers: Buffers,
    /// `TYRE_CHANGE` (`race`).
    pub tyre_change: TyreChange,
    /// `RACE_VARS` (`race`).
    /// One copy per drawing thread (`workers`).
    pub vars: crate::workers::PerWorker<RaceVars>,
    /// `RACE_VIEW` (`race`).
    /// One copy per drawing thread (`workers`).
    pub view: crate::workers::PerWorker<RaceView>,
    /// `FRAME_SWITCHES` (`race`).
    /// One copy per drawing thread (`workers`).
    pub switches: crate::workers::PerWorker<FrameSwitches>,
    /// `REPLAY_FILE` (`race`).
    pub replay_file: ReplayFileBlock,
    /// `TEMPLATES` (`race`).
    pub replay_templates: Templates,
    /// `REPLAYS` (`race`).
    pub replays: Replays,
    /// `PHYSICS` (`physics`).
    /// One copy per drawing thread (`workers`).
    pub physics_tables: crate::workers::PerWorker<PhysicsTables>,
    /// `CAR` (`physics`).
    /// One copy per drawing thread (`workers`).
    pub car: crate::workers::PerWorker<Car>,
}

impl Shared {
    pub const fn new() -> Self {
        Shared {
            work: crate::workers::PerWorker::new(
                [const { RaceWork::new() }; crate::workers::COPIES],
            ),
            state: RaceState::new(),
            tables: RaceTables::new(),
            buffers: Buffers::new(),
            tyre_change: TyreChange::new(),
            vars: crate::workers::PerWorker::new(
                [const { RaceVars::new() }; crate::workers::COPIES],
            ),
            view: crate::workers::PerWorker::new(
                [const { RaceView::new() }; crate::workers::COPIES],
            ),
            switches: crate::workers::PerWorker::new(
                [const { FrameSwitches::new() }; crate::workers::COPIES],
            ),
            replay_file: ReplayFileBlock::new(),
            replay_templates: Templates::new(),
            replays: Replays::new(),
            physics_tables: crate::workers::PerWorker::new(
                [const { PhysicsTables::new() }; crate::workers::COPIES],
            ),
            car: crate::workers::PerWorker::new([const { Car::new() }; crate::workers::COPIES]),
        }
    }
}
