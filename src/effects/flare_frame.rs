//! `effects::flare_frame`: `scene::set_texture` with `KindSlots.kind_first_slots[202]`,
//! `scene::depth_test_off`, and when the settings' `listener_figures[0]` (the flare's strength) is
//! set `effects::draw_lens_flare`.
//!
//! Refuses null settings, before anything.

use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};

/// `scene::set_texture`, `scene::depth_test_off`, and with the settings' flare strength
/// `draw_lens_flare`.
pub(super) fn flare_frame(
    engine_tables: &crate::math::engine_tables::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let part = scene.scratch.game_scratch();
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.borrow().kind_first_slots[202],
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
