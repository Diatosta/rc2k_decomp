//! The race owner's physics group (`race::owner::Race::physics`): the physics work: camera, contact, gear, ground, pace, split tables, damage, car specs, checkpoints.
//! `GEAR` and `CHECKPOINTS` stay views over their group statics (`LIGHTS_AND_GEAR`, `BUFFERS`).

use crate::physics::camera_store::CameraWork;
use crate::physics::contact_store::ContactWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::physics::pace_store::PaceNotes;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::store::DamageModel;
use crate::race::car_specs_store::CarSpecs;

/// The race's physics state (step 11, owned state: the stores move in as fields).
pub struct Physics {
    /// One copy: no drawing part writes it (the write watch's carried census, frame plan F7).
    pub camera: CameraWork,
    /// One copy: the drawing's searches have their own (frame plan F4).
    pub contact: ContactWork,
    /// One copy: the drawing's searches have their own (frame plan F4).
    pub ground_probe: GroundProbe,
    /// One copy per drawing thread (`workers`).
    pub ground: crate::workers::PerWorker<GroundWork>,
    pub pace: PaceNotes,
    pub split_tables: SplitTables,
    pub damage: DamageModel,
    pub car_specs: CarSpecs,
}

impl Physics {
    pub const fn new() -> Self {
        Physics {
            camera: CameraWork::new(),
            contact: ContactWork::new(),
            ground_probe: GroundProbe::new(),
            ground: crate::workers::PerWorker::new(
                [const { GroundWork::new() }; crate::workers::COPIES],
            ),
            pace: PaceNotes::new(),
            split_tables: SplitTables::new(),
            damage: DamageModel::new(),
            car_specs: CarSpecs::new(),
        }
    }
}
