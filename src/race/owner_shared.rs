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
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub work: RaceWork,
    /// `RACE_STATE` (`race`).
    pub state: RaceState,
    /// `RACE_TABLES` (`race`).
    pub tables: RaceTables,
    /// `BUFFERS` (`race`).
    pub buffers: Buffers,
    /// `TYRE_CHANGE` (`race`).
    pub tyre_change: TyreChange,
    /// `RACE_VARS` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub vars: RaceVars,
    /// `RACE_VIEW` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub view: RaceView,
    /// `FRAME_SWITCHES` (`race`).
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub switches: FrameSwitches,
    /// `REPLAY_FILE` (`race`).
    pub replay_file: ReplayFileBlock,
    /// `TEMPLATES` (`race`).
    pub replay_templates: Templates,
    /// `REPLAYS` (`race`).
    pub replays: Replays,
    /// `PHYSICS` (`physics`).
    /// One copy: the drawing only reads it (frame plan F4).
    pub physics_tables: PhysicsTables,
    /// `CAR` (`physics`): the work car. One copy: the drawing no longer reads it (frame plan
    /// F3), so the drawing threads need none of their own.
    pub car: Car,
}

impl Shared {
    pub const fn new() -> Self {
        Shared {
            work: RaceWork::new(),
            state: RaceState::new(),
            tables: RaceTables::new(),
            buffers: Buffers::new(),
            tyre_change: TyreChange::new(),
            vars: RaceVars::new(),
            view: RaceView::new(),
            switches: FrameSwitches::new(),
            replay_file: ReplayFileBlock::new(),
            replay_templates: Templates::new(),
            replays: Replays::new(),
            physics_tables: PhysicsTables::new(),
            car: Car::new(),
        }
    }
}
