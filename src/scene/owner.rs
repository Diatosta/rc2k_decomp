//! Module scene's owner (step 11, owned state): the scene's state in one struct, handed by
//! reference to the code that runs it in place of global stores. The file numbers of the
//! object loaders, the draw switches, the light tables and the car icon upload join it first;
//! the other stores follow one by one.
//!
//! `scene()` is the boundary: the callers outside the module that do not take the owner fetch
//! it there. It rises as their callers take the owner (`notes/memory_layer_plan.md`, step 11).

use crate::scene::draw_store::SceneDraw;
use crate::scene::draw_switches_store::DrawSwitches;
use crate::scene::file_numbers_store::ObjectFileNumbers;
use crate::scene::kind_texture_store::KindTextureTables;
use crate::scene::light_store::LightState;
use crate::scene::light_tables_store::LightTables;
use crate::scene::light_work_store::LightWork;
use crate::scene::object_slots_store::ObjectTables as ObjectSlots;
use crate::scene::object_store::ObjectWork;
use crate::scene::object_tables_store::ObjectTables;
use crate::scene::texture_places_store::TexturePlaces;
use crate::scene::upload_car_icon::IconUploadCell;

/// The scene's state.
pub struct Scene {
    pub file_numbers: ObjectFileNumbers,
    pub draw_switches: DrawSwitches,
    pub light_tables: LightTables,
    pub icon_upload: IconUploadCell,
    pub drawing: Drawing,
    pub lights: Lights,
    pub objects: Objects,
    pub textures: Textures,
}

/// The drawing state: the draw variables and the scene objects' work.
pub struct Drawing {
    /// The scene's draw variables (`DRAW`).
    /// One copy per drawing thread (`workers`).
    pub scene: crate::workers::PerWorker<SceneDraw>,
    /// The scene objects' work variables (`OBJECT_WORK`).
    /// One copy per drawing thread (`workers`).
    pub object_work: crate::workers::PerWorker<ObjectWork>,
}

/// The light state and the lights' work variables.
pub struct Lights {
    /// The light state (`LIGHT_STATE`).
    /// One copy per drawing thread (`workers`).
    pub state: crate::workers::PerWorker<LightState>,
    /// The lights' work variables (`LIGHT_WORK`).
    /// One copy per drawing thread (`workers`).
    pub work: crate::workers::PerWorker<LightWork>,
}

/// The object tables: the slots and the kinds' tables.
pub struct Objects {
    /// The object slots (`OBJECT_SLOTS`).
    pub slots: ObjectSlots,
    /// The object tables (`OBJECT_TABLES`).
    pub tables: ObjectTables,
}

/// The texture places and the kinds' texture tables.
pub struct Textures {
    /// The texture places (`TEXTURE_PLACES`).
    pub places: TexturePlaces,
    /// The kinds' texture tables (`KIND_TEXTURES`).
    pub kinds: KindTextureTables,
}

/// The one instance. Reached through `scene()`; `moved` registers its stores' places.
pub static SCENE: Scene = Scene {
    file_numbers: ObjectFileNumbers::new(),
    draw_switches: DrawSwitches::new(),
    light_tables: LightTables::new(),
    icon_upload: IconUploadCell::new(),
    drawing: Drawing {
        scene: crate::workers::PerWorker::new([const { SceneDraw::new() }; crate::workers::COUNT]),
        object_work: crate::workers::PerWorker::new(
            [const { ObjectWork::new() }; crate::workers::COUNT],
        ),
    },
    lights: Lights {
        state: crate::workers::PerWorker::new([const { LightState::new() }; crate::workers::COUNT]),
        work: crate::workers::PerWorker::new([const { LightWork::new() }; crate::workers::COUNT]),
    },
    objects: Objects {
        slots: ObjectSlots::new(),
        tables: ObjectTables::new(),
    },
    textures: Textures {
        places: TexturePlaces::new(),
        kinds: KindTextureTables::new(),
    },
};

/// The scene's state, for a caller that does not take it as a parameter yet.
pub fn scene() -> &'static Scene {
    &SCENE
}
