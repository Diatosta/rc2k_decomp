//! `effects::run_frame`: the effects' frame drawn, in this order: `effects::swap_mask`,
//! `effects::tick_clock`, the render states, `effects::run_rain`, `effects::run_snow`,
//! `effects::draw_frame`, `effects::ring_frame`, `effects::flare_frame`,
//! `effects::draw_mark_slots`, `effects::run_particles`, `effects::stage_record`,
//! `effects::draw_glow`, then the depth buffer and test on. Three shares of the frame's time (the
//! cycle count over the frame's cycles, times three constants) go into the draw's, the ring's and
//! the whole's shares.

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use core::cell::Cell;
/// Where the share goes (its factor is `FRAME_K.times`): the draw, the ring, the whole.
fn shares(effects: &Effects) -> [&Cell<f32>; 3] {
    [
        &effects.setup.frame.draw_share,
        &effects.setup.frame.ring_share,
        &effects.setup.frame.whole_share,
    ]
}
const STATES: [(u32, u32); 6] = [
    (4, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (1, 3),
];

fn share(at: usize, start: Option<u32>, effects: &crate::effects::owner::Effects) {
    let Some(took) = crate::effects::cycles::since(start) else {
        return;
    };
    let to = shares(effects)[at];
    to.set(x87::int_over_int_times(
        took,
        effects.setup.frame.frame_cycles.get(),
        FRAME_K.times[at],
    ));
}

/// The view the effects are drawn for (a player's main view, from its overlays): its fog, its
/// matrix and its light; and the race's sine tables the sprites turn by.
#[derive(Clone, Copy)]
pub struct EffectsView<'a> {
    pub fog: &'a crate::scene::fog::Fog,
    pub view_matrix: crate::scene::matrix::Matrix,
    pub light: u32,
    pub engine_tables: crate::math::engine_tables::EngineTablesRef<'a>,
}

impl<'a> EffectsView<'a> {
    /// `view`'s own, in race `race`.
    pub fn of(race: &'a Race, view: &'a crate::race::view_scratch::ViewScratch) -> Self {
        Self {
            fog: &view.fog,
            view_matrix: view.matrices.borrow().view_matrix,
            light: view.light.borrow().view_light,
            engine_tables: race.engine_tables(),
        }
    }

    /// Before any view of the race (`drive::race_init`'s settling steps): the matrix the last
    /// view drawn left; the front end's fog and light (the snow's fades worked out from the
    /// light are worked out again at every frame before they are drawn).
    /// are drawn).
    pub fn before_any_view(race: &'a Race, scene: &'a crate::scene::owner::Scene) -> Self {
        let front_end = scene.scratch.front_end();
        Self {
            fog: &front_end.fog,
            view_matrix: race.frame.view_kept.borrow().view_matrix,
            light: front_end.light.borrow().view_light,
            engine_tables: race.engine_tables(),
        }
    }
}

/// For `view`.
#[allow(
    clippy::too_many_arguments,
    reason = "the owners one frame of the effects reads, as they are lent"
)]
pub fn run_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    sound: &crate::sound::owner::Sound,
    view: EffectsView<'_>,
) -> Result<(), Refused> {
    let part = scene.scratch.game_scratch();
    use crate::effects::*;
    use crate::scene::set_render_state::set_render_state;
    let (weather, particles, ring) = (&effects.weather, &effects.particles, &effects.ring);
    let paths = &effects.paths;
    {
        swap_mask::swap_mask(&effects.masks, scene, gfx, system, effects)?;
        setup::tick_clock(&race.shared, effects)?;
        let whole = crate::effects::cycles::start();
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        run_rain::run_rain(buffer, race, weather, scene, effects)?;
        run_snow::run_snow(buffer, race, weather, scene, effects, view)?;
        let start = crate::effects::cycles::start();
        share(0, start, effects);
        draw_frame::draw_frame(buffer, race, scene, effects)?;
        let start = crate::effects::cycles::start();
        ring_frame::ring_frame(
            buffer,
            &effects.masks,
            weather,
            particles,
            ring,
            scene,
            effects,
            view,
        )?;
        share(1, start, effects);
        set_render_state(part, 2, 0x14);
        flare_frame::flare_frame(view.engine_tables, buffer, paths, scene, effects)?;
        set_render_state(part, 1, 3);
        draw_mark_slots::draw_mark_slots(buffer, race, paths, scene, effects, view)?;
        run_particles::run_particles(
            buffer,
            &race.frame,
            particles,
            ring,
            scene,
            effects,
            sound,
            view,
        )?;
        share(2, whole, effects);
        stage_record::stage_record(&race.frame)?;
        draw_glow::draw_glow(view.engine_tables, buffer, scene, effects)?;
        crate::scene::depth_buffer_on::depth_buffer(part, true);
        crate::scene::depth_test_on::depth_test_on(part);
    }
    Ok(())
}

/// The frame's time constants.
pub struct FrameK {
    /// The marks' share factor (`frame_marks`)
    pub share_k: f32,
    /// The frame's time factors (`run_frame`)
    pub times: [f32; 3],
}

/// FrameK's first values.
pub(super) static FRAME_K: FrameK = FrameK {
    share_k: 100.0_f32,
    times: [100.0, 100.0, 100.0],
};
