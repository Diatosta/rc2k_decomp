//! Module scene's owner: the scene's state in one struct, handed by reference to the code that runs
//! it in place of global stores. The file numbers of the object loaders, the draw switches, the
//! light tables and the car icon upload join it first; the other stores follow one by one.

use crate::memory::session::{DrawPlain, GameThread};
use crate::scene::draw_store::{DrawWork, SceneDraw};
use crate::scene::draw_switches_store::DrawSwitches;
use crate::scene::file_numbers_store::{ObjectFileNumbers, PathText};
use crate::scene::kind_texture_store::{KindSlots, KindTextures};
use crate::scene::light_tables_store::LightTables;
use crate::scene::light_work_store::LightWork;
use crate::scene::line_shake_store::LineAndShake;
use crate::scene::object_slots_store::{LoadedSlots, SlotShapes};
use crate::scene::object_store::ObjectWork;
use crate::scene::object_tables_store::{ObjectKinds, TextureLevels};
use crate::scene::texture_places_store::{PlaceSlots, TexturePlaces};
use std::cell::{Cell, Ref, RefCell, RefMut};

/// The scene's state.
pub struct Scene {
    /// Written between frames, read by the drawing parts through their reads
    /// (`scene::workers::DrawReads`).
    pub light_tables: DrawPlain<LightTables>,
    pub drawing: Drawing,
    /// The scratch every drawing part and view works on (`scene::scratch_store::DrawingScratch`).
    pub scratch: crate::scene::scratch_store::DrawingScratch,
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
    /// The slots' tables the drawing parts do not read: borrowed for one statement at a time
    /// (`Scene::loaded_slots`).
    loaded_slots: RefCell<LoadedSlots>,
    /// The slots' kinds and the texture level tables: borrowed for one statement at a time
    /// (`Scene::texture_levels`).
    texture_levels: RefCell<TextureLevels>,
    /// The work words: borrowed for one statement at a time (`Scene::draw_work`).
    draw_work: RefCell<DrawWork>,
    /// The best-line place and the camera shake: borrowed for one statement at a time
    /// (`Scene::line_and_shake`).
    line_and_shake: RefCell<LineAndShake>,
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

    /// The best-line place and the camera shake, read: no write borrow may be held.
    pub fn line_and_shake(&self) -> Ref<'_, LineAndShake> {
        self.main.get().line_and_shake.borrow()
    }

    /// The best-line place and the camera shake, to write: no other borrow may be held.
    pub fn line_and_shake_mut(&self) -> RefMut<'_, LineAndShake> {
        self.main.get().line_and_shake.borrow_mut()
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

    /// The slots' tables the drawing does not read (but `Objects::slots`), read: no write borrow
    /// may be held.
    pub fn loaded_slots(&self) -> Ref<'_, LoadedSlots> {
        self.main.get().loaded_slots.borrow()
    }

    /// The slots' tables, to write: no other borrow may be held.
    pub fn loaded_slots_mut(&self) -> RefMut<'_, LoadedSlots> {
        self.main.get().loaded_slots.borrow_mut()
    }

    /// The slots' kinds and the texture level tables, read: no write borrow may be held.
    pub fn texture_levels(&self) -> Ref<'_, TextureLevels> {
        self.main.get().texture_levels.borrow()
    }

    /// The slots' kinds and the texture level tables, to write: no other borrow may be held.
    pub fn texture_levels_mut(&self) -> RefMut<'_, TextureLevels> {
        self.main.get().texture_levels.borrow_mut()
    }

    /// The work words (the shared scratch dword, the physics' work floats, the turns' sines), read:
    /// no write borrow may be held.
    pub fn draw_work(&self) -> Ref<'_, DrawWork> {
        self.main.get().draw_work.borrow()
    }

    /// The work words, to write: no other borrow may be held.
    pub fn draw_work_mut(&self) -> RefMut<'_, DrawWork> {
        self.main.get().draw_work.borrow_mut()
    }

    /// The shared scratch dword as a cell, put back when it is dropped: no borrow of the work words
    /// may be held while it lives.
    pub fn scratch_cell(&self) -> crate::scene::draw_store::ScratchCell<'_> {
        crate::scene::draw_store::ScratchCell::new(self)
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
    /// The scene's draw variables: written between frames, read by the drawing parts through
    /// their reads (`scene::workers::DrawReads`).
    pub scene: DrawPlain<SceneDraw>,
    /// The `v3driverfix` setting (read on the game's thread).
    pub v3_driver_fix: GameThread<Cell<u32>>,
}

/// The object tables: the slots and the kinds' tables.
pub struct Objects {
    /// The object slots (`OBJECT_SLOTS`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub slots: DrawPlain<SlotShapes>,
    /// The object tables (`OBJECT_TABLES`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub tables: DrawPlain<ObjectKinds>,
}

/// The texture places the drawing reads and the kinds' texture tables.
pub struct Textures {
    /// Each place's slot and the video textures' count: written between frames, read by the
    /// drawing parts through their reads (`scene::workers::DrawReads`).
    pub place_slots: DrawPlain<PlaceSlots>,
    /// The kinds' texture tables: written between frames, read by the drawing
    /// parts through their reads (`scene::workers::DrawReads`).
    pub kinds: DrawPlain<KindSlots>,
    /// Each kind's first texture: borrowed for one statement at a time (`Textures::first_textures`).
    first_textures: GameThread<RefCell<KindTextures>>,
}

impl Textures {
    /// The kinds' first textures, read: no write borrow may be held.
    pub fn first_textures(&self) -> Ref<'_, KindTextures> {
        self.first_textures.get().borrow()
    }

    /// The kinds' first textures, to write: no other borrow may be held.
    pub fn first_textures_mut(&self) -> RefMut<'_, KindTextures> {
        self.first_textures.get().borrow_mut()
    }
}

impl Scene {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            light_tables: DrawPlain::new(LightTables::new()),
            scratch: crate::scene::scratch_store::DrawingScratch::new(),
            drawing: Drawing {
                scene: DrawPlain::new(SceneDraw::new()),
                v3_driver_fix: GameThread::new(Cell::new(0)),
            },
            objects: Objects {
                slots: DrawPlain::new(SlotShapes::new()),
                tables: DrawPlain::new(ObjectKinds::new()),
            },
            textures: Textures {
                place_slots: DrawPlain::new(PlaceSlots::new()),
                kinds: DrawPlain::new(KindSlots::new()),
                first_textures: GameThread::new(RefCell::new([0; 1024])),
            },
            main: GameThread::new(SceneMain {
                file_numbers: RefCell::new(ObjectFileNumbers::new()),
                object_work: RefCell::new(ObjectWork::new()),
                draw_switches: RefCell::new(DrawSwitches::new()),
                light_work: RefCell::new(LightWork::new()),
                texture_places: RefCell::new(TexturePlaces::new()),
                loaded_slots: RefCell::new(LoadedSlots::new()),
                texture_levels: RefCell::new(TextureLevels::new()),
                draw_work: RefCell::new(DrawWork::new()),
                line_and_shake: RefCell::new(LineAndShake::new()),
            }),
        }
    }
}
