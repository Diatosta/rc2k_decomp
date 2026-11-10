//! `effects::flare_frame` (0x4C4EA2): `scene::set_texture` with `dword_5D1E10`,
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
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if effects.setup.settings.is_set.get() == 0 {
            return Err(BadInput("no marks settings").into());
        }
        crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[202].bits());
        crate::scene::depth_test_off::depth_test_off();
        let marks = crate::effects::setup::effects_settings()?;
        if marks.listener_figures[0].get() != 0 {
            crate::effects::draw_lens_flare::draw_lens_flare(paths, effects)?;
        }
    }
    Ok(())
}
