//! `effects::ring_frame`: the emitters' ring drawn: render states 0x15 = 4, 0x13 = 5, 0x14 = 6,
//! 0x1C = 1, 0x1B = 1, `scene::set_texture` (the ring's texture), `scene::depth_test_off`; the
//! ring's colour = the first car's `tag`'s low 24 bits, the kind-1 choice = the rain's drop count;
//! `effects::run_ring`; render state 0x14 = 2; `effects::run_ribbons`.
//!
//! Screen: the dust and the water splashes (confirmed: both gone with the ring's sprites cut).
//!
//! Works on the effects settings.
//!
//! Refuses null settings, before anything.

use crate::effects::mask_store::Masks;
use crate::effects::particles_store::Particles;
use crate::effects::ring_store::Ring;
use crate::errors::Refused;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `scene::depth_test_off`, `run_ring`, `run_ribbons`.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
pub(super) fn ring_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    weather: &crate::effects::rain_store::Weather,
    particles: &Particles,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    let part = scene.scratch.game_scratch();
    use crate::scene::set_render_state::set_render_state;
    {
        let marks = effects.settings()?;
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.borrow().kind_first_slots[211],
        );
        crate::scene::depth_test_off::depth_test_off(part);
        // SurfaceTables.lives_by_level[8]: the ninth of `lives_by_level`.
        // The ring's colour: the first car record's tag.
        masks.surface.borrow_mut().lives_by_level[8] = marks.cars[0].tag.get() & 0xFF_FFFF;
        let drops = weather.rain.borrow().drops;
        particles.kind1_choice.set(drops);
        crate::effects::run_ring::run_ring(buffer, masks, ring, scene, effects, view)?;
        set_render_state(part, 2, 0x14);
        crate::effects::run_ribbons::run_ribbons(buffer, ring, scene, effects, view)?;
    }
    Ok(())
}
