//! Module scene's owner: the scene's state in one struct, handed by reference to the code that runs
//! it in place of global stores. The file numbers of the object loaders, the draw switches, the
//! light tables and the car icon upload join it first; the other stores follow one by one.

use crate::memory::session::{DrawShared, GameThread};
use crate::scene::draw_store::SceneDraw;
use crate::scene::draw_switches_store::DrawSwitches;
use crate::scene::file_numbers_store::ObjectFileNumbers;
use crate::scene::icon_upload_store::IconUploadCell;
use crate::scene::kind_texture_store::KindTextureTables;
use crate::scene::light_store::LightState;
use crate::scene::light_tables_store::LightTables;
use crate::scene::light_work_store::LightWork;
use crate::scene::object_slots_store::ObjectTables as ObjectSlots;
use crate::scene::object_store::ObjectWork;
use crate::scene::object_tables_store::ObjectTables;
use crate::scene::texture_places_store::TexturePlaces;

/// The scene's state.
pub struct Scene {
    /// Written between frames, read by the drawing parts through their reads
    /// (`scene::workers::DrawReads`).
    pub draw_switches: DrawShared<DrawSwitches>,
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
/// loaders' file numbers and path texts, the car icon upload's work and the scene objects'
/// work variables.
struct SceneMain {
    file_numbers: ObjectFileNumbers,
    icon_upload: IconUploadCell,
    /// The scene objects' work variables (`OBJECT_WORK`).
    object_work: ObjectWork,
}

impl Scene {
    pub fn file_numbers(&self) -> &ObjectFileNumbers {
        &self.main.get().file_numbers
    }

    pub fn icon_upload(&self) -> &IconUploadCell {
        &self.main.get().icon_upload
    }

    pub fn object_work(&self) -> &ObjectWork {
        &self.main.get().object_work
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
    /// The lights' work variables (`LIGHT_WORK`): written between frames, read by the drawing
    /// parts through their reads (`scene::workers::DrawReads`).
    pub work: DrawShared<LightWork>,
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

/// The texture places and the kinds' texture tables.
pub struct Textures {
    /// The texture places (`TEXTURE_PLACES`): written between frames, read by the drawing parts
    /// through their reads (`scene::workers::DrawReads`).
    pub places: DrawShared<TexturePlaces>,
    /// The kinds' texture tables: written between frames, read by the drawing
    /// parts through their reads (`scene::workers::DrawReads`).
    pub kinds: DrawShared<KindTextureTables>,
}

impl Scene {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            draw_switches: DrawShared::new(DrawSwitches::new()),
            light_tables: DrawShared::new(LightTables::new()),
            scratch: crate::scene::scratch_store::DrawingScratch::new(),
            drawing: Drawing {
                scene: DrawShared::new(SceneDraw::new()),
            },
            lights: Lights {
                state: DrawShared::new(LightState::new()),
                work: DrawShared::new(LightWork::new()),
            },
            objects: Objects {
                slots: DrawShared::new(ObjectSlots::new()),
                tables: DrawShared::new(ObjectTables::new()),
            },
            textures: Textures {
                places: DrawShared::new(TexturePlaces::new()),
                kinds: DrawShared::new(KindTextureTables::new()),
            },
            main: GameThread::new(SceneMain {
                file_numbers: ObjectFileNumbers::new(),
                icon_upload: IconUploadCell::new(),
                object_work: ObjectWork::new(),
            }),
        }
    }
}
