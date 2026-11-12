//! `effects::run_frame` (0x4D1549): the effects' frame drawn: `effects::swap_mask`,
//! `effects::tick_clock`; render states 0x15 = 4, 0x1C = 0, 0x13 = 5, 0x14 = 6, 0x1B = 1,
//! 3 = 1; `effects::run_rain`, `effects::run_snow`; `effects::draw_frame`;
//! `effects::ring_frame`; render state 0x14 = 2; `effects::flare_frame`; render state 3 = 1;
//! `effects::draw_mark_slots`, `effects::run_particles`; `effects::stage_record`,
//! `effects::draw_glow`, `scene::depth_buffer_on`, `scene::depth_test_on`. Three shares of the
//! frame's time (`rdtsc` over the frame's cycles, times three constants) into the draw's, the
//! ring's and the whole's shares.//!
//! Caller: the race's frame (`race::run_view_marks`). Its calls are in
//! the order above.

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner::Race;
use crate::session::PlainFloat;
/// Where the share goes (its factor is `FRAME_K.times`): the draw, the ring, the whole.
fn shares(effects: &Effects) -> [&PlainFloat; 3] {
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

fn rdtsc() -> u32 {
    crate::arch::rdtsc() as u32
}

fn share(at: usize, start: u32, effects: &crate::effects::owner::Effects) {
    let to = shares(effects)[at];
    let took = rdtsc().wrapping_sub(start) as i32;
    to.set(x87::int_over_int_times(
        took,
        effects.setup.frame.frame_cycles.bits() as i32,
        FRAME_K.times[at],
    ));
}

/// The view the effects are drawn for (a player's main view, from its overlays): its fog, its
/// matrix (the original's `g_viewMatrix`) and its light (`g_viewLight`), which the original
/// read from what the last view drawn left; and the race's sine tables the sprites turn by.
#[derive(Clone, Copy)]
pub struct EffectsView<'a> {
    pub fog: &'a crate::scene::fog::Fog,
    pub view_matrix: &'a [crate::session::Float; 16],
    pub light: u32,
    pub engine_tables: &'a crate::scene::engine_tables_store::EngineTables,
}

impl<'a> EffectsView<'a> {
    /// `view`'s own, in race `race`.
    pub fn of(race: &'a Race, view: &'a crate::race::view_scratch::ViewScratch) -> Self {
        Self {
            fog: &view.fog,
            view_matrix: &view.matrices.view_matrix,
            light: view.light.view_light.bits(),
            engine_tables: race.engine_tables(),
        }
    }

    /// Before any view of the race (`drive::race_init`'s settling steps): the matrix the last
    /// view drawn left, as the original read; the front end's fog and light (frame plan F4; the
    /// snow's fades worked out from the light are worked out again at every frame before they
    /// are drawn).
    pub fn before_any_view(race: &'a Race) -> Self {
        let front_end = crate::race::view_scratch::front_end();
        Self {
            fog: &front_end.fog,
            view_matrix: &race.frame.matrices.view_matrix,
            light: front_end.light.view_light.bits(),
            engine_tables: race.engine_tables(),
        }
    }
}

/// For `view`.
pub fn run_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    sound: &crate::sound::owner::Sound,
    view: EffectsView<'_>,
) -> Result<(), Refused> {
    let part = crate::workers::game_scratch();
    let gfx = crate::gfx::owner::gfx();
    use crate::effects::*;
    use crate::scene::set_render_state::set_render_state;
    let (weather, particles, ring) = (&effects.weather, &effects.particles, &effects.ring);
    let paths = &effects.paths;
    {
        swap_mask::swap_mask(&effects.masks, scene, gfx, effects)?;
        setup::tick_clock(&race.shared, effects)?;
        let whole = rdtsc();
        for (value, state) in STATES {
            set_render_state(part, value, state);
        }
        run_rain::run_rain(buffer, race, weather, scene, effects)?;
        run_snow::run_snow(buffer, race, weather, scene, effects, view)?;
        let start = rdtsc();
        share(0, start, effects);
        draw_frame::draw_frame(buffer, race, scene, effects)?;
        let start = rdtsc();
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

/// The frame's time constants (IDA 0x93C8C0 .. 0x93C8D0).
#[repr(C)]
pub struct FrameK {
    /// 0x93C8C0: `flt_93C8C0`: the marks' share factor (`frame_marks`)
    pub share_k: f32,
    /// 0x93C8C4: `flt_93C8C4`, `flt_93C8C8`, `flt_93C8CC`: the frame's time factors (`run_frame`)
    pub times: [f32; 3],
}

/// FrameK's first values.
pub static FRAME_K: FrameK = FrameK {
    share_k: 100.0_f32,
    times: [100.0, 100.0, 100.0],
};

#[cfg(test)]
impl FrameK {
    pub const LEN: usize = 0x10;
}

#[cfg(test)]
mod frame_k_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn frame_k_starts_as_the_games() {
        assert_eq!(size_of::<FrameK>(), FrameK::LEN);
        assert!(
            bytes(&FRAME_K)
                == crate::test_support::first_values::bytes("effects.run_frame.FRAME_K"),
            "FRAME_K starts differently from the image"
        );
        assert_eq!(offset_of!(FrameK, share_k), 0x0);
        assert_eq!(offset_of!(FrameK, times), 0x4);
    }
}
