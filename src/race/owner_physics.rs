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
use std::cell::{Ref, RefCell, RefMut};

/// What only the game's thread reaches: the camera, the ground, the pace notes, the split
/// tables, the damage model and the car specs (no drawing part, audio, key or network thread
/// touches them; the drawing's searches have their own copies).
struct PhysicsMain {
    pub camera: RefCell<CameraWork>,
    pub ground: GroundWork,
    pub pace: RefCell<PaceNotes>,
    pub split_tables: RefCell<SplitTables>,
    pub damage: DamageModel,
    pub car_specs: CarSpecs,
}

impl PhysicsMain {
    pub const fn new() -> Self {
        PhysicsMain {
            camera: RefCell::new(CameraWork::new()),
            ground: GroundWork::new(),
            pace: RefCell::new(PaceNotes::new()),
            split_tables: RefCell::new(SplitTables::new()),
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

    /// The camera work, read: no write borrow may be held.
    pub fn camera(&self) -> Ref<'_, CameraWork> {
        self.main.get().camera.borrow()
    }

    /// The camera work, to write: no other borrow may be held.
    pub fn camera_mut(&self) -> RefMut<'_, CameraWork> {
        self.main.get().camera.borrow_mut()
    }

    /// The camera work's cell, for the step inputs that hold it across a step.
    pub fn camera_cell(&self) -> &RefCell<CameraWork> {
        &self.main.get().camera
    }

    pub fn ground(&self) -> &GroundWork {
        &self.main.get().ground
    }

    /// The pace notes, read: no write borrow may be held.
    pub fn pace(&self) -> Ref<'_, PaceNotes> {
        self.main.get().pace.borrow()
    }

    /// The pace notes, to write: no other borrow may be held.
    pub fn pace_mut(&self) -> RefMut<'_, PaceNotes> {
        self.main.get().pace.borrow_mut()
    }

    /// The split tables, read: no write borrow may be held.
    pub fn split_tables(&self) -> Ref<'_, SplitTables> {
        self.main.get().split_tables.borrow()
    }

    /// The split tables, to write: no other borrow may be held.
    pub fn split_tables_mut(&self) -> RefMut<'_, SplitTables> {
        self.main.get().split_tables.borrow_mut()
    }

    pub fn damage(&self) -> &DamageModel {
        &self.main.get().damage
    }

    pub fn car_specs(&self) -> &CarSpecs {
        &self.main.get().car_specs
    }
}
