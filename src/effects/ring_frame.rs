//! `effects::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (the ring's texture),
//! `scene::depth_test_off`; the ring's colour = the settings' +0x7C's low 24 bits, the kind-1
//! choice = the rain's drop count; `effects::run_ring`; render state 0x14 = 2;
//! `effects::run_ribbons`.//!
//! Screen: the dust and the water splashes (confirmed: both gone with the ring's sprites cut).
//!
//! Works on the effects settings.
//!
//! Caller: `effects::run_frame` Refuses null settings, before anything.

use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::Refused;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `scene::depth_test_off`, `run_ring`, `run_ribbons`.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
pub fn ring_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    weather: &crate::effects::rain::Weather,
    particles: &Particles,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    let part = crate::workers::game_scratch();
    use crate::scene::set_render_state::set_render_state;
    {
        let marks = effects.settings()?;
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.get().kind_first_slots[211].get(),
        );
        crate::scene::depth_test_off::depth_test_off(part);
        // SurfaceTables.lives_by_level[8]: the ninth of `lives_by_level`.
        // The settings' +0x7C: the first car record's +0x1C.
        masks.surface.lives_by_level[8].set(marks.cars[0].tag.get() & 0xFF_FFFF);
        particles.kind1_choice.choice.set(weather.rain.drops.get());
        crate::effects::run_ring::run_ring(buffer, masks, ring, scene, effects, view)?;
        set_render_state(part, 2, 0x14);
        crate::effects::run_ribbons::run_ribbons(buffer, ring, scene, effects, view)?;
    }
    Ok(())
}
