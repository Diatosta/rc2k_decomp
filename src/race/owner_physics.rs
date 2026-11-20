//! The race owner's physics group (`race::owner::Race::physics`): the physics work: camera,
//! contact, gear, ground, pace, split tables, damage, car specs, checkpoints. The gear figures and
//! the checkpoints stay views over their group statics (`groups_store::LightsAndGear`, `BUFFERS`).

use crate::memory::session::{DrawShared, GameThread};
use crate::physics::camera_store::CameraWork;
use crate::physics::contact_store::ContactWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::physics::pace_store::PaceNotes;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::store::DamageModel;
use crate::race::car_specs_store::CarSpecs;

/// What only the game's thread reaches: the camera, the ground, the pace notes, the split
/// tables, the damage model and the car specs (no drawing part, audio, key or network thread
/// touches them; the drawing's searches have their own copies).
struct PhysicsMain {
    pub camera: CameraWork,
    pub ground: GroundWork,
    pub pace: PaceNotes,
    pub split_tables: SplitTables,
    pub damage: DamageModel,
    pub car_specs: CarSpecs,
}

impl PhysicsMain {
    pub const fn new() -> Self {
        PhysicsMain {
            camera: CameraWork::new(),
            ground: GroundWork::new(),
            pace: PaceNotes::new(),
            split_tables: SplitTables::new(),
            damage: DamageModel::new(),
            car_specs: CarSpecs::new(),
        }
    }
}

/// The race's physics state.
pub struct Physics {
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the drawing's searches have
    /// their own and read only the lit faces.
    pub contact: DrawShared<ContactWork>,
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the drawing's searches
    /// have their own fields and read only its mark.
    pub ground_probe: DrawShared<GroundProbe>,
    main: GameThread<PhysicsMain>,
}

impl Physics {
    pub const fn new() -> Self {
        Physics {
            contact: DrawShared::new(ContactWork::new()),
            ground_probe: DrawShared::new(GroundProbe::new()),
            main: GameThread::new(PhysicsMain::new()),
        }
    }

    pub fn camera(&self) -> &CameraWork {
        &self.main.get().camera
    }

    pub fn ground(&self) -> &GroundWork {
        &self.main.get().ground
    }

    pub fn pace(&self) -> &PaceNotes {
        &self.main.get().pace
    }

    pub fn split_tables(&self) -> &SplitTables {
        &self.main.get().split_tables
    }

    pub fn damage(&self) -> &DamageModel {
        &self.main.get().damage
    }

    pub fn car_specs(&self) -> &CarSpecs {
        &self.main.get().car_specs
    }
}
