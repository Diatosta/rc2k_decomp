//! `effects::flare_frame` (0x4C4EA2): `scene::set_texture` with `KindTextureTables.kind_first_slots[202]`,
//! `scene::depth_test_off`, and when the settings' +0x2754 (the flare's strength) is set
//! `effects::draw_lens_flare`.
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::run_frame` (0x4D1645). Refuses null settings, before anything.

use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};

/// `scene::set_texture`, `scene::depth_test_off`, and
/// with the settings' +0x2754 `draw_lens_flare`.
pub fn flare_frame(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let part = crate::workers::game_scratch();
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.get().kind_first_slots[202].get(),
        );
        crate::scene::depth_test_off::depth_test_off(part);
        let marks = effects.settings()?;
        if marks.listener_figures[0].get() != 0 {
            crate::effects::draw_lens_flare::draw_lens_flare(
                engine_tables,
                buffer,
                paths,
                scene,
                effects,
            )?;
        }
    }
    Ok(())
}
