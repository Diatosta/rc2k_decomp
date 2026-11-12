//! The smaller views of `StepInputs` that one family of steps reads, each built by a method
//! of `StepInputs` and handed down in its place: the sound cues of a contact, the ground
//! search and probe, the contact steps (which hold the other two), the camera, the reset
//! chain and the clear-place search, the checkpoints, the lighting and the moving slots.

use crate::drive::finish_store::Finish;
use crate::drive::replay_frames_store::ReplayFrames;
use crate::physics::camera_store::CameraWork;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::step_inputs::StepInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::frame_store::FrameWork;
use crate::race::groups_store::ViewAndRecords;
use crate::race::lights_store::LightsData;
use crate::race::matrices_store::SceneMatrices;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use crate::race::scratch_store::FrameScratch;
use crate::race::sound_spots_store::SoundSpotBlock;
use crate::race::tables_store::RaceTables;
use crate::race::vars_store::RaceVars;
use crate::race::view_store::RaceView;
use crate::race::work_store::RaceWork;

/// What a contact's sound cue reads: the car, the numbers, the sound spots and bands, the
/// matrices that turn the place, the contact's speed, the current car.
#[derive(Clone, Copy)]
pub struct SoundInputs<'a> {
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub work: &'a RaceWork,
    pub frame_work: &'a FrameWork,
    pub matrices: &'a SceneMatrices,
    pub spots: &'a SoundSpotBlock,
    pub polygon_tables: &'a crate::scene::polygon_store::PolygonTables,
    pub contact: &'a ContactWork,
}

/// What the ground search and the probe points read.
#[derive(Clone, Copy)]
pub struct GroundInputs<'a> {
    pub car: &'a Car,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub course: &'a Course,
    pub contact: &'a ContactWork,
    pub ground: &'a GroundWork,
    pub probe: &'a GroundProbe,
    pub engine_tables: &'a crate::scene::engine_tables_store::EngineTables,
}

/// What the camera's ground checks read: the car's camera place, the ground search and
/// probe, the camera variables.
#[derive(Clone, Copy)]
pub struct CameraGroundInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub camera: &'a CameraWork,
    pub ground: &'a GroundWork,
    pub probe: &'a GroundProbe,
}

/// What the camera steps read: the camera variables, the car's place and the views, the
/// light and view tables, the replay's frames, and the camera's ground checks.
#[derive(Clone, Copy)]
pub struct CameraInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub numbers: &'a Numbers,
    pub tables: &'a PhysicsTables,
    pub camera: &'a CameraWork,
    pub split_tables: &'a SplitTables,
    pub race_tables: &'a RaceTables,
    pub shared: &'a RaceShared,
    pub frame_work: &'a FrameWork,
    pub view_records: &'a ViewAndRecords,
    pub lights_data: &'a LightsData,
    pub replay_frames: &'a ReplayFrames,
    pub checkpoints: &'a crate::race::checkpoints_store::Checkpoints,
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
    pub contact: &'a ContactWork,
    pub work: &'a RaceWork,
    pub scratch: &'a FrameScratch,
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
    pub finish: &'a Finish,
    pub car_blocks: &'a crate::race::blocks_store::CarBlocks,
    pub matrices: &'a SceneMatrices,
    pub ground: GroundInputs<'a>,
}

/// What `car_collisions` reads: the contact steps' inputs (the car, the other cars' work, the
/// corner hits), the matrices the places are turned by, the race mode and the cars' count.
#[derive(Clone, Copy)]
pub struct CollisionInputs<'a> {
    pub contact: ContactInputs<'a>,
    pub matrices: &'a SceneMatrices,
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
    pub frame_work: &'a FrameWork,
    pub finish: &'a Finish,
    pub checkpoints: &'a crate::race::checkpoints_store::Checkpoints,
}

/// What the car's lighting reads: the car, the course's faces, the ground search and probe,
/// and the scene switch.
#[derive(Clone, Copy)]
pub struct LightingInputs<'a> {
    pub car: &'a Car,
    pub course: &'a Course,
    pub ground: &'a GroundWork,
    pub probe: &'a GroundProbe,
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
    pub contact: &'a ContactWork,
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
    pub matrices: &'a SceneMatrices,
    pub ground: &'a GroundWork,
    pub probe: &'a GroundProbe,
}

impl<'a> StepInputs<'a> {
    pub fn camera(&self) -> CameraInputs<'a> {
        CameraInputs {
            car: self.car,
            course: self.course,
            numbers: self.numbers,
            tables: self.tables,
            camera: self.physics.camera(),
            split_tables: self.physics.split_tables(),
            race_tables: self.race_tables,
            shared: self.shared,
            frame_work: &self.frame.frame_work,
            view_records: &self.frame.view_and_records,
            lights_data: self.course.lights_data(),
            replay_frames: self.replay_frames,
            checkpoints: &self.shared.buffers.checkpoints,
            kind_tables: &self.replay.viewer_and_kinds.kind_tables,
            light_records: &self.frame.lights_and_gear.get().light_records,
            ground: self.camera_ground(),
        }
    }

    pub fn camera_ground(&self) -> CameraGroundInputs<'a> {
        CameraGroundInputs {
            car: self.car,
            course: self.course,
            camera: self.physics.camera(),
            ground: self.physics.ground(),
            probe: self.physics.ground_probe.get(),
        }
    }

    pub fn sound(&self) -> SoundInputs<'a> {
        SoundInputs {
            car: self.car,
            numbers: self.numbers,
            work: self.work,
            frame_work: &self.frame.frame_work,
            matrices: &self.frame.matrices,
            spots: self.frame.sound_spots(),
            polygon_tables: &self.shared.buffers.polygon_tables,
            contact: &self.physics.contact,
        }
    }

    pub fn ground(&self) -> GroundInputs<'a> {
        GroundInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            contact: &self.physics.contact,
            ground: self.physics.ground(),
            probe: self.physics.ground_probe.get(),
            engine_tables: &self.shared.buffers.engine_tables,
        }
    }

    pub fn contact(&self) -> ContactInputs<'a> {
        ContactInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            contact: &self.physics.contact,
            work: self.work,
            scratch: self.frame.scratch(),
            shared: self.shared,
            ground: self.ground(),
            sound: self.sound(),
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
            finish: self.replay.finish.get(),
            car_blocks: &self.replay.cars_and_replay.car_blocks,
            matrices: &self.frame.matrices,
            ground: self.ground(),
        }
    }

    pub fn collisions(&self) -> CollisionInputs<'a> {
        CollisionInputs {
            contact: self.contact(),
            matrices: &self.frame.matrices,
            vars: self.vars,
            view: self.view,
        }
    }

    pub fn clear_place(&self) -> ClearPlaceInputs<'a> {
        ClearPlaceInputs {
            reset: self.reset(),
            collisions: self.collisions(),
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
            finish: self.replay.finish.get(),
            checkpoints: &self.shared.buffers.checkpoints,
        }
    }

    pub fn lighting(&self) -> LightingInputs<'a> {
        LightingInputs {
            car: self.car,
            course: self.course,
            ground: self.physics.ground(),
            probe: self.physics.ground_probe.get(),
            vars: self.vars,
            polygon_tables: &self.shared.buffers.polygon_tables,
        }
    }

    pub fn wheels(&self) -> WheelInputs<'a> {
        WheelInputs {
            car: self.car,
            numbers: self.numbers,
            tables: self.tables,
            course: self.course,
            shared: self.shared,
            contact: &self.physics.contact,
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
            ground: self.physics.ground(),
            probe: self.physics.ground_probe.get(),
        }
    }
}
