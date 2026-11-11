//! The smaller views of `StepInputs` that one family of steps reads, each built by a method
//! of `StepInputs` and handed down in its place: the sound cues of a contact, the ground
//! search and probe, and the contact steps (which hold the other two).

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
use crate::race::owner_shared::RaceShared;
use crate::race::scratch_store::FrameScratch;
use crate::race::sound_spots_store::SoundSpotBlock;
use crate::race::tables_store::RaceTables;
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

impl<'a> StepInputs<'a> {
    pub fn camera(&self) -> CameraInputs<'a> {
        CameraInputs {
            car: self.car,
            course: self.course,
            numbers: self.numbers,
            tables: self.tables,
            camera: &self.physics.camera,
            split_tables: &self.physics.split_tables,
            race_tables: self.race_tables,
            shared: self.shared,
            frame_work: &self.frame.frame_work,
            view_records: &self.frame.view_and_records,
            lights_data: &self.course.lights_data,
            replay_frames: self.replay_frames,
            ground: self.camera_ground(),
        }
    }

    pub fn camera_ground(&self) -> CameraGroundInputs<'a> {
        CameraGroundInputs {
            car: self.car,
            course: self.course,
            camera: &self.physics.camera,
            ground: &self.physics.ground,
            probe: &self.physics.ground_probe,
        }
    }

    pub fn sound(&self) -> SoundInputs<'a> {
        SoundInputs {
            car: self.car,
            numbers: self.numbers,
            work: self.work,
            frame_work: &self.frame.frame_work,
            matrices: &self.frame.matrices,
            spots: &self.frame.sound_spots,
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
            ground: &self.physics.ground,
            probe: &self.physics.ground_probe,
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
            scratch: &self.frame.scratch,
            shared: self.shared,
            ground: self.ground(),
            sound: self.sound(),
        }
    }
}
