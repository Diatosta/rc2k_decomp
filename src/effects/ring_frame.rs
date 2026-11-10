//! `effects::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (`dword_5D1E34`),
//! `scene::depth_test_off`; `dword_939BA6` the settings' +0x7C's low 24 bits,
//! `g_marksKind1Choice` `dword_8BC50D` (the rain's drops); `effects::run_ring`; render state
//! 0x14 = 2; `effects::run_ribbons`.
//!
//! Screen: the dust and the water splashes (owner, 2026-10-29: both gone with the ring's sprites cut).
//!
//! Works on `g_marksSettings`.
//!
//! Caller: `effects::run_frame` (0x4D160B). Refuses null settings, before anything.

use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::run_particles::Particles;
use crate::errors::Refused;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `scene::depth_test_off`, `run_ring`, `run_ribbons`.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
pub fn ring_frame(
    masks: &Masks,
    weather: &crate::effects::rain::Weather,
    particles: &Particles,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    {
        let marks = crate::effects::setup::marks()?;
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[211].bits());
        crate::scene::depth_test_off::depth_test_off();
        // dword_939BA6: the ninth of `lives_by_level`.
        // The settings' +0x7C: the first car record's +0x1C.
        masks.surface.lives_by_level[8].set(marks.cars[0].tag.get() & 0xFF_FFFF);
        particles.kind1_choice.choice.set(weather.rain.drops.get());
        crate::effects::run_ring::run_ring(masks, ring, effects, view)?;
        set_render_state(2, 0x14);
        crate::effects::run_ribbons::run_ribbons(ring, effects, view)?;
    }
    Ok(())
}
