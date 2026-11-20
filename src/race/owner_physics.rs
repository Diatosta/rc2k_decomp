//! The race owner's physics group (`race::owner::Race::physics`): the physics work: camera,
//! contact, gear, ground, pace, split tables, damage, car specs, checkpoints. The gear figures and
//! the checkpoints stay views over their group statics (`groups_store::LightsAndGear`, `BUFFERS`).

use crate::memory::session::{DrawShared, GameThread};
use crate::physics::camera_store::CameraWork;
use crate::physics::contact_store::ContactWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::GroundSearch;
use crate::physics::ground_store::GroundWork;
use crate::physics::pace_store::PaceNotes;
use crate::physics::split_tables_store::SplitTables;
use crate::physics::store::DamageModel;
use crate::race::car_specs_store::CarSpecs;
use std::cell::{Ref, RefCell, RefMut};

/// What only the game's thread reaches: the camera, the ground work and search, the pace notes, the split
/// tables, the damage model and the car specs (no drawing part, audio, key or network thread
/// touches them; the drawing's searches have their own copies).
struct PhysicsMain {
    pub camera: RefCell<CameraWork>,
    pub ground: GroundWork,
    pub ground_search: RefCell<GroundSearch>,
    pub pace: RefCell<PaceNotes>,
    pub split_tables: RefCell<SplitTables>,
    pub damage: RefCell<DamageModel>,
    /// The ground search's mark: borrowed for one statement at a time; the drawing parts read the
    /// copy their scope was opened with (`scene::workers::DrawReads::ground_probe`).
    pub ground_probe: RefCell<GroundProbe>,
    pub car_specs: CarSpecs,
}

impl PhysicsMain {
    pub const fn new() -> Self {
        PhysicsMain {
            camera: RefCell::new(CameraWork::new()),
            ground: GroundWork::new(),
            ground_search: RefCell::new(GroundSearch::new()),
            pace: RefCell::new(PaceNotes::new()),
            split_tables: RefCell::new(SplitTables::new()),
            damage: RefCell::new(DamageModel::new()),
            ground_probe: RefCell::new(GroundProbe::new()),
            car_specs: CarSpecs::new(),
        }
    }
}

/// The race's physics state.
pub struct Physics {
    /// One copy, lent to the drawing parts (`scene::workers::DrawReads`): the drawing's searches have
    /// their own and read only the lit faces.
    pub contact: DrawShared<ContactWork>,
    main: GameThread<PhysicsMain>,
}

impl Physics {
    pub const fn new() -> Self {
        Physics {
            contact: DrawShared::new(ContactWork::new()),
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

    /// The ground search's record: the place asked for, the face found, its shares and height. The
    /// search holds the borrow while it runs; nothing else reads it then.
    pub fn ground_search(&self) -> &RefCell<GroundSearch> {
        &self.main.get().ground_search
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

    /// The damage model, read: no write borrow may be held.
    pub fn damage(&self) -> Ref<'_, DamageModel> {
        self.main.get().damage.borrow()
    }

    /// The damage model, to write: no other borrow may be held.
    pub fn damage_mut(&self) -> RefMut<'_, DamageModel> {
        self.main.get().damage.borrow_mut()
    }

    /// The ground search's mark, for the searches that set it and the ones that read it.
    pub fn ground_probe(&self) -> &RefCell<GroundProbe> {
        &self.main.get().ground_probe
    }

    pub fn car_specs(&self) -> &CarSpecs {
        &self.main.get().car_specs
    }
}
