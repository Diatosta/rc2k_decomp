//! Module scene's owner: the scene's state in one struct, handed by reference to the code that runs
//! it in place of global stores. The file numbers of the object loaders, the draw switches, the
//! light tables and the car icon upload join it first; the other stores follow one by one.

use crate::memory::session::{DrawShared, GameThread};
use crate::scene::draw_store::SceneDraw;
use crate::scene::draw_switches_store::DrawSwitches;
use crate::scene::file_numbers_store::{ObjectFileNumbers, PathText};
use crate::scene::kind_texture_store::KindTextureTables;
use crate::scene::light_store::LightState;
use crate::scene::light_tables_store::LightTables;
use crate::scene::light_work_store::LightWork;
use crate::scene::object_slots_store::ObjectTables as ObjectSlots;
use crate::scene::object_store::ObjectWork;
use crate::scene::object_tables_store::ObjectTables;
use crate::scene::texture_places_store::{PlaceSlots, TexturePlaces};
use std::cell::{Ref, RefCell, RefMut};

/// The scene's state.
pub struct Scene {
    /// Written between frames, read by the drawing parts through their reads
    /// (`scene::workers::DrawReads`).
    pub light_tables: DrawShared<LightTables>,
    pub drawing: Drawing,
    /// The scratch every drawing part and view works on (`scene::scratch_store::DrawingScratch`).
    pub scratch: crate::scene::scratch_store::DrawingScratch,
    pub lights: Lights,
    pub objects: Objects,
    pub textures: Textures,
    main: GameThread<SceneMain>,
}

/// What the game's thread alone reaches (no drawing part or other thread does): the object
/// loaders' file numbers and path texts and the scene objects' work variables.
struct SceneMain {
    /// The loaders' file numbers and path texts: borrowed for one statement at a time.
    file_numbers: RefCell<ObjectFileNumbers>,
    /// The scene objects' work variables: borrowed for one statement at a time (`Scene::object_work`,
    /// `Scene::object_work_mut`); `Scene` is shared by reference, so the writes go through the cell.
    object_work: RefCell<ObjectWork>,
    /// The draw switches: borrowed for one statement at a time (`Scene::draw_switches`); the
    /// drawing parts read the copy their scope was opened with.
    draw_switches: RefCell<DrawSwitches>,
    /// The lights' work variables: borrowed for one statement at a time (`Scene::light_work`); the
    /// drawing parts read the copy their scope was opened with.
    light_work: RefCell<LightWork>,
    /// The texture places the drawing parts do not read: borrowed for one statement at a time
    /// (`Scene::texture_places`).
    texture_places: RefCell<TexturePlaces>,
}

impl Scene {
    /// The loaders' file numbers and path texts, read: no write borrow may be held.
    pub fn file_numbers(&self) -> Ref<'_, ObjectFileNumbers> {
        self.main.get().file_numbers.borrow()
    }

    /// The loaders' file numbers and path texts, to write: no other borrow may be held.
    pub fn file_numbers_mut(&self) -> RefMut<'_, ObjectFileNumbers> {
        self.main.get().file_numbers.borrow_mut()
    }

    /// The path text that starts at byte `from` of the file numbers' texts (`texts`); each of its
    /// reads and writes borrows the store itself.
    pub fn path_text(&self, from: usize) -> PathText<'_> {
        PathText::new(&self.main.get().file_numbers, from)
    }

    /// The draw switches, read: no write borrow may be held.
    pub fn draw_switches(&self) -> Ref<'_, DrawSwitches> {
        self.main.get().draw_switches.borrow()
    }

    /// The draw switches, to write: no other borrow may be held.
    pub fn draw_switches_mut(&self) -> RefMut<'_, DrawSwitches> {
        self.main.get().draw_switches.borrow_mut()
    }

    /// The lights' work variables, read: no write borrow may be held.
    pub fn light_work(&self) -> Ref<'_, LightWork> {
        self.main.get().light_work.borrow()
    }

    /// The lights' work variables, to write: no other borrow may be held.
    pub fn light_work_mut(&self) -> RefMut<'_, LightWork> {
        self.main.get().light_work.borrow_mut()
    }

    /// The texture places (but the part the drawing reads, `Textures::place_slots`), read: no
    /// write borrow may be held.
    pub fn texture_places(&self) -> Ref<'_, TexturePlaces> {
        self.main.get().texture_places.borrow()
    }

    /// The texture places, to write: no other borrow may be held.
    pub fn texture_places_mut(&self) -> RefMut<'_, TexturePlaces> {
        self.main.get().texture_places.borrow_mut()
    }

    /// The scene objects' work variables, read: no write borrow may be held.
    pub fn object_work(&self) -> Ref<'_, ObjectWork> {
        self.main.get().object_work.borrow()
    }

    /// The scene objects' work variables, to write: no other borrow may be held.
    pub fn object_work_mut(&self) -> RefMut<'_, ObjectWork> {
        self.main.get().object_work.borrow_mut()
    }
}

/// The drawing state: the draw variables and the scene objects' work.
pub struct Drawing {
    /// The scene's draw variables (`DRAW`).
    /// Written between frames (mostly the physics' work dwords), read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub scene: DrawShared<SceneDraw>,
}

/// The light state and the lights' work variables.
pub struct Lights {
    /// The light state (`LIGHT_STATE`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub state: DrawShared<LightState>,
}

/// The object tables: the slots and the kinds' tables.
pub struct Objects {
    /// The object slots (`OBJECT_SLOTS`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub slots: DrawShared<ObjectSlots>,
    /// The object tables (`OBJECT_TABLES`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub tables: DrawShared<ObjectTables>,
}

/// The texture places the drawing reads and the kinds' texture tables.
pub struct Textures {
    /// Each place's slot and the video textures' count: written between frames, read by the
    /// drawing parts through their reads (`scene::workers::DrawReads`).
    pub place_slots: DrawShared<PlaceSlots>,
    /// The kinds' texture tables: written between frames, read by the drawing
    /// parts through their reads (`scene::workers::DrawReads`).
    pub kinds: DrawShared<KindTextureTables>,
}

impl Scene {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            light_tables: DrawShared::new(LightTables::new()),
            scratch: crate::scene::scratch_store::DrawingScratch::new(),
            drawing: Drawing {
                scene: DrawShared::new(SceneDraw::new()),
            },
            lights: Lights {
                state: DrawShared::new(LightState::new()),
            },
            objects: Objects {
                slots: DrawShared::new(ObjectSlots::new()),
                tables: DrawShared::new(ObjectTables::new()),
            },
            textures: Textures {
                place_slots: DrawShared::new(PlaceSlots::new()),
                kinds: DrawShared::new(KindTextureTables::new()),
            },
            main: GameThread::new(SceneMain {
                file_numbers: RefCell::new(ObjectFileNumbers::new()),
                object_work: RefCell::new(ObjectWork::new()),
                draw_switches: RefCell::new(DrawSwitches::new()),
                light_work: RefCell::new(LightWork::new()),
                texture_places: RefCell::new(TexturePlaces::new()),
            }),
        }
    }
}
