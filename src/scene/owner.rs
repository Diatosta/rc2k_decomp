//! Module scene's owner (step 11, owned state): the scene's state in one struct, handed by
//! reference to the code that runs it in place of global stores. The file numbers of the
//! object loaders, the draw switches, the light tables and the car icon upload join it first;
//! the other stores follow one by one.
//!
//! `scene()` is the boundary: the callers outside the module that do not take the owner fetch
//! it there. It rises as their callers take the owner (`notes/memory_layer_plan.md`, step 11).

use crate::scene::draw_switches_store::DrawSwitches;
use crate::scene::file_numbers_store::ObjectFileNumbers;
use crate::scene::light_tables_store::LightTables;
use crate::scene::upload_car_icon::IconUploadCell;

/// The scene's state.
pub struct Scene {
    pub file_numbers: ObjectFileNumbers,
    pub draw_switches: DrawSwitches,
    pub light_tables: LightTables,
    pub icon_upload: IconUploadCell,
}

/// The one instance. Reached through `scene()`; `moved` registers its stores' places.
pub static SCENE: Scene = Scene {
    file_numbers: ObjectFileNumbers::new(),
    draw_switches: DrawSwitches::new(),
    light_tables: LightTables::new(),
    icon_upload: IconUploadCell::new(),
};

/// The scene's state, for a caller that does not take it as a parameter yet.
pub fn scene() -> &'static Scene {
    &SCENE
}
