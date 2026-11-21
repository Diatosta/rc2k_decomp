//! The smaller views of `StepInputs` that one family of steps reads, each built by a method of
//! `StepInputs` and handed down in its place: the sound cues of a contact, the ground search and
//! probe, the contact steps (which hold the other two), the camera, the reset chain and the
//! clear-place search, the checkpoints, the lighting and the moving slots.

use crate::memory::session::DrawPlain;
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::physics::contact_store::{ContactWork, ProbeLists};
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::GroundSearch;
use crate::physics::step_inputs::StepInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::frame_store::FrameWorkStore;
use crate::race::groups_store::ViewAndRecords;
use crate::race::lights_store::LightsData;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::RaceTables;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::work_store::RaceWork;
use std::cell::RefCell;

/// What a contact's sound cue reads: the car, the numbers, the sound bands, the
/// matrices that turn the place, the contact's speed, the current car.
#[derive(Clone, Copy)]
pub struct SoundInputs<'a> {
    pub car: &'a Car,
    /// The car whose camera the sounds are placed against (`race::heard_cars::listener`).
    pub listener: &'a Car,
    pub numbers: &'a Numbers,
    pub work: &'a RaceWork,
    pub frame_work: &'a FrameWorkStore,
    pub matrices: &'a DrawPlain<SceneMatrices>,
    pub view_turn: crate::scene::matrix::Matrix,
    pub heard_turn: &'a crate::race::heard_turn::HeardTurn,
    pub polygon_tables: &'a crate::scene::polygon_store::PolygonTables,
    pub contact: &'a RefCell<ContactWork>,
    /// The sound owner, whose objects the cues are placed on.
    pub owner: &'a crate::sound::owner::Sound,
}

/// What the ground search and the probe points read.
#[derive(Clone, Copy)]
pub struct GroundInputs<'a> {
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub contact: &'a RefCell<ContactWork>,
    pub probes: &'a RefCell<ProbeLists>,
    pub search: &'a RefCell<GroundSearch>,
    pub probe: &'a RefCell<GroundProbe>,
    pub engine_tables: &'a crate::math::engine_tables::EngineTables,
}

/// What the camera's ground checks read: the car's camera place, the ground search and
/// probe, the camera variables.
#[derive(Clone, Copy)]
pub struct CameraGroundInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub camera: &'a RefCell<CameraWork>,
    pub search: &'a RefCell<GroundSearch>,
    pub probe: &'a RefCell<GroundProbe>,
}

/// What the camera steps read: the camera variables, the car's place and the views, the
/// light and view tables, the replay's frames, and the camera's ground checks.
#[derive(Clone, Copy)]
pub struct CameraInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub camera: &'a RefCell<CameraWork>,
    /// The replay still flag (`SplitTables.still`), as it is when the inputs are built.
    pub still: u32,
    pub race_tables: &'a RaceTables,
    pub shared: &'a RaceShared,
    pub frame_work: &'a FrameWorkStore,
    pub view_records: &'a ViewAndRecords,
    pub lights_data: &'a LightsData,
    pub step_first: u32,
    pub checkpoints: crate::race::checkpoints_store::CheckpointsRef<'a>,
    pub kind_tables: &'a crate::race::kind_store::KindTables,
    pub light_records: &'a crate::race::light_records_store::LightRecords,
    pub ground: CameraGroundInputs<'a>,
}

/// What the contact, wall and ground-touch steps read.
#[derive(Clone, Copy)]
pub struct ContactInputs<'a> {
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub contact: &'a RefCell<ContactWork>,
    pub lit_faces: &'a crate::physics::lit_faces_store::LitFacesStore,
    pub work: &'a RaceWork,
    pub frame: &'a crate::race::owner_frame::Frame,
    pub shared: &'a RaceShared,
    pub ground: GroundInputs<'a>,
    pub sound: SoundInputs<'a>,
}

/// What the reset chain reads (`reset_car`, `reset_car_on_line`, `reset_when_stuck`,
/// `false_start`, and the cars' starts `start_car`, `settle_car`, `reset_car_motion`): the car
/// and its tables, the course and the nearest-checkpoint search, the start place's flags, the
/// matrices the settling turns, and the ground probe the settling uses.
#[derive(Clone, Copy)]
pub struct ResetInputs<'a> {
    pub physics: &'a Physics,
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub work: &'a RaceWork,
    pub shared: &'a RaceShared,
    pub vars: &'a RaceVars,
    pub view: &'a RaceView,
    /// The finish state's "no place" switch.
    pub no_place: u32,
    pub car_blocks: &'a crate::race::blocks_store::CarBlocks,
    pub matrices: &'a DrawPlain<SceneMatrices>,
    pub ground: GroundInputs<'a>,
}

/// What `car_collisions` reads: the contact steps' inputs (the car, the other cars' work, the
/// corner hits), the matrices the places are turned by, the race mode and the cars' count.
#[derive(Clone, Copy)]
pub struct CollisionInputs<'a> {
    pub contact: ContactInputs<'a>,
    pub matrices: &'a DrawPlain<SceneMatrices>,
    pub vars: &'a RaceVars,
    pub view: &'a RaceView,
}

/// What the clear-place search reads: the reset chain it ends in and the collision steps it
/// tries (which hold the contact steps).
#[derive(Clone, Copy)]
pub struct ClearPlaceInputs<'a> {
    pub reset: ResetInputs<'a>,
    pub collisions: CollisionInputs<'a>,
}

/// What the checkpoint steps read: the checkpoint tracking, the pace notes and the split
/// points. `physics` holds the ground work, the split tables and the pace notes they search.
#[derive(Clone, Copy)]
pub struct CheckpointInputs<'a> {
    pub physics: &'a Physics,
    pub course: &'a Course,
    pub numbers: &'a Numbers,
    pub shared: &'a RaceShared,
    pub car: &'a Car,
    pub tables: &'a PhysicsTables,
    pub work: &'a RaceWork,
    pub vars: &'a RaceVars,
    pub frame_work: &'a FrameWorkStore,
    /// The finish state's "no place" switch.
    pub no_place: u32,
    pub checkpoints: crate::race::checkpoints_store::CheckpointsRef<'a>,
}

/// What the car's lighting reads: the car, the course's faces, the ground search and probe,
/// and the scene switch.
#[derive(Clone, Copy)]
pub struct LightingInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub search: &'a RefCell<GroundSearch>,
    pub probe: &'a RefCell<GroundProbe>,
    pub vars: &'a RaceVars,
    pub polygon_tables: &'a crate::scene::polygon_store::PolygonTables,
}

/// What the wheels' surfaces read: the car and the surface tables, the probe points' shares,
/// the surface kinds and the mud scale. `shared` is for the x87 steps, which take the whole.
#[derive(Clone, Copy)]
pub struct WheelInputs<'a> {
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub shared: &'a RaceShared,
    pub probes: &'a RefCell<ProbeLists>,
    pub view: &'a RaceView,
    pub vars: &'a RaceVars,
}

/// What the moving slots read: the slots and their scratch places, the physics numbers, the
/// course's turn lists, the matrices and the ground search.
#[derive(Clone, Copy)]
pub struct SlotInputs<'a> {
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub work: &'a RaceWork,
    pub matrices: &'a DrawPlain<SceneMatrices>,
    pub search: &'a RefCell<GroundSearch>,
    pub probe: &'a RefCell<GroundProbe>,
}

impl<'a> StepInputs<'a> {
    pub fn camera(&self) -> CameraInputs<'a> {
        CameraInputs {
            car: self.car,
            course: self.course,
            numbers: self.numbers,
            tables: self.tables,
            camera: self.physics.camera_cell(),
            still: self.physics.split_tables().still,
            race_tables: self.race_tables,
            shared: self.shared,
            frame_work: &self.frame.frame_work,
            view_records: self.frame.view_and_records.get(),
            lights_data: self.course.lights_data(),
            step_first: self.step_first,
            checkpoints: self.shared.checkpoints(),
            kind_tables: &crate::race::kind_store::KIND_TABLES,
            light_records: &self.frame.lights_and_gear.get().light_records,
            ground: self.camera_ground(),
        }
    }

    pub fn camera_ground(&self) -> CameraGroundInputs<'a> {
        CameraGroundInputs {
            car: self.car,
            course: self.course,
            camera: self.physics.camera_cell(),
            search: self.physics.ground_search(),
            probe: self.physics.ground_probe(),
        }
    }

    pub fn sound(&self, owner: &'a crate::sound::owner::Sound) -> SoundInputs<'a> {
        SoundInputs {
            car: self.car,
            listener: crate::race::heard_cars::listener(self.shared, self.car),
            numbers: self.numbers,
            work: self.work,
            frame_work: &self.frame.frame_work,
            matrices: &self.frame.matrices,
            view_turn: self.frame.view_kept.borrow().view_turn_matrix,
            heard_turn: self.shared.heard_turn.get(),
            polygon_tables: &self.shared.buffers.reads.get().polygon_tables,
            contact: self.physics.contact(),
            owner,
        }
    }

    pub fn ground(&self) -> GroundInputs<'a> {
        GroundInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            contact: self.physics.contact(),
            probes: self.physics.probes(),
            search: self.physics.ground_search(),
            probe: self.physics.ground_probe(),
            engine_tables: &self.shared.buffers.reads.get().engine_tables,
        }
    }

    pub fn contact(&self, sound: &'a crate::sound::owner::Sound) -> ContactInputs<'a> {
        ContactInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            contact: self.physics.contact(),
            lit_faces: &self.physics.lit_faces,
            work: self.work,
            frame: self.frame,
            shared: self.shared,
            ground: self.ground(),
            sound: self.sound(sound),
        }
    }

    pub fn reset(&self) -> ResetInputs<'a> {
        ResetInputs {
            physics: self.physics,
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            work: self.work,
            shared: self.shared,
            vars: self.vars,
            view: self.view,
            no_place: self.replay.no_place(),
            car_blocks: self.shared.car_blocks.get(),
            matrices: &self.frame.matrices,
            ground: self.ground(),
        }
    }

    pub fn collisions(&self, sound: &'a crate::sound::owner::Sound) -> CollisionInputs<'a> {
        CollisionInputs {
            contact: self.contact(sound),
            matrices: &self.frame.matrices,
            vars: self.vars,
            view: self.view,
        }
    }

    pub fn clear_place(&self, sound: &'a crate::sound::owner::Sound) -> ClearPlaceInputs<'a> {
        ClearPlaceInputs {
            reset: self.reset(),
            collisions: self.collisions(sound),
        }
    }

    pub fn checkpoints(&self) -> CheckpointInputs<'a> {
        CheckpointInputs {
            physics: self.physics,
            course: self.course,
            numbers: self.numbers,
            shared: self.shared,
            car: self.car,
            tables: self.tables,
            work: self.work,
            vars: self.vars,
            frame_work: &self.frame.frame_work,
            no_place: self.replay.no_place(),
            checkpoints: self.shared.checkpoints(),
        }
    }

    pub fn lighting(&self) -> LightingInputs<'a> {
        LightingInputs {
            car: self.car,
            course: self.course,
            search: self.physics.ground_search(),
            probe: self.physics.ground_probe(),
            vars: self.vars,
            polygon_tables: &self.shared.buffers.reads.get().polygon_tables,
        }
    }

    pub fn wheels(&self) -> WheelInputs<'a> {
        WheelInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            shared: self.shared,
            probes: self.physics.probes(),
            view: self.view,
            vars: self.vars,
        }
    }

    pub fn slots(&self) -> SlotInputs<'a> {
        SlotInputs {
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            work: self.work,
            matrices: &self.frame.matrices,
            search: self.physics.ground_search(),
            probe: self.physics.ground_probe(),
        }
    }
}
