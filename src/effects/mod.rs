//! The race's effects: the weather (rain, snow), the wheels' spray, dust and mud, the drops
//! on the windscreen, the lens flare and the surface textures they need (files\m\: masks,
//! edges, snow, splines, surface data). `effects::init_marks` sets them up as a
//! stage loads, `effects::run_frame` runs them each frame of the race.
//!
//! The game's own name for the step is "init mark" (its loading log).

mod add_drift_particle;
mod add_mark_slot;
mod adjugate3;
mod aim_vector;
#[cfg(test)]
mod asm_layout;
#[cfg(test)]
mod asm_test;
mod begin_quads;
mod bend_points;
mod blend_corner;
mod blend_corner_store;
mod blend_points;
mod blocks;
mod blocks_store;
mod burst_particles;
mod cars_particles;
pub(crate) mod clip_quad;
mod clip_store;
mod copy_first_point;
mod corners;
mod cycles;
mod draw_cell_sprite;
mod draw_flare_part;
mod draw_frame;
mod draw_glow;
mod draw_lens_flare;
mod draw_mark_slots;
pub(crate) mod draw_overlays;
mod draw_particle;
mod draw_plane;
mod draw_pong_box;
mod draw_rain_streak;
mod draw_ribbon;
pub(crate) mod draw_shadow;
mod draw_snow;
mod draw_subcell_sprite;
mod draw_turned_sprite;
mod draw_windscreen_flake;
mod drift_offsets;
mod edges;
mod emit_body_contact_particles;
mod emit_body_particles;
mod emit_bursts_kind_3;
mod emit_car_particles;
mod emit_exhaust_smoke;
mod emit_exhaust_smoke_store;
mod emit_rolling_dust;
mod emit_rolling_dust_store;
mod emit_rolling_particles_rated;
mod emit_rolling_particles_rated_store;
mod emit_slip_snow_cloud;
mod emit_slip_snow_cloud_store;
mod emit_slip_spray;
mod emit_slip_spray_store;
mod emit_twin_particle_first;
mod emit_twin_particle_first_store;
mod emit_twin_particle_second;
mod emit_wheel_dust_cloud;
mod emit_wheel_dust_cloud_store;
mod emit_wheel_grit_bursts;
mod emit_wheel_grit_bursts_store;
mod emit_wheel_particles;
mod emitters;
mod emitters_store;
mod flare_frame;
pub(crate) mod frame_marks;
mod frame_state_store;
pub(crate) mod init_mark_step;
mod init_marks;
mod item_walk;
mod jitter_point;
mod keep_vectors;
mod kept_turns;
mod kind_store;
mod kind_values;
mod kind_values_store;
mod lines;
mod lines_store;
mod load_mask_pictures;
mod make_fade_colours;
mod mark_slots;
mod mark_slots_store;
mod mask_constants_store;
mod mask_picks_store;
pub(crate) mod mask_store;
mod mask_vars_store;
mod masks;
mod morph_object_points;
mod morph_object_points_store;
mod move_pong_ball;
mod move_pong_bat;
pub(crate) mod owner;
mod owner_groups_store;
mod particles_store;
mod paths_store;
mod pick_shapes;
mod pixels;
mod place_graded_points;
mod place_rain_view;
mod place_snow_view;
mod place_spline;
mod plain;
mod pong;
mod pong_ball_hits;
mod pong_cell;
mod pong_digit;
mod pong_score;
mod pong_setup;
mod pong_store;
mod profiles;
mod profiles_store;
mod rain_store;
mod ring_entries;
mod ring_frame;
mod ring_store;
mod rings;
pub(crate) mod run_frame;
mod run_particles;
mod run_pong;
mod run_rain;
mod run_ribbons;
mod run_ring;
mod run_shapes;
mod run_snow;
mod run_splines;
mod run_windscreen_flakes;
mod scale_axes;
mod scale_kept_vectors;
mod scale_setting;
mod scale_vectors_by;
mod set_quad_uvs;
mod set_up_surface;
pub(crate) mod setup;
pub(crate) mod setup_store;
mod shape_points;
mod shape_points_store;
mod shape_work_store;
mod shapes;
mod shapes_frame;
mod snow_store;
mod spans;
mod spawn_gate;
mod splines;
mod splines_store;
mod spread_point;
mod spread_point_store;
mod stage_record;
mod surface_store;
mod swap_mask;
pub(crate) mod tidy_marks;
mod tilt_shape;
mod tilt_shape_store;
mod trail_particles;
mod tuning_store;
mod turns;
#[cfg(test)]
mod typed_sync;
mod view_frame;
mod view_heading;
mod view_motion;
mod view_step;
mod view_vectors_store;
mod waves_store;
mod windscreen_drop_state_store;
pub(crate) mod windscreen_drop_vars_store;
mod windscreen_drops;
mod windscreen_drops_drift;
mod windscreen_drops_frame;
mod windscreen_drops_setup;
mod windscreen_drops_spawn;
mod windscreen_drops_top;
mod windscreen_flakes;
mod windscreen_flakes_frame;
mod windscreen_flakes_setup;
mod windscreen_flakes_spawn;
mod windscreen_flakes_store;
pub(crate) mod work_store;

/// The first refusal met inside a callback that cannot return one (a closure handed to a
/// core that asks for plain numbers): the callback answers the type's default after it, and
/// the caller asks `done` once the core has returned, so the refusal is not lost.
#[derive(Default)]
struct Sticky(std::cell::Cell<Option<crate::errors::BadInput>>);

impl Sticky {
    /// `r`'s value, or the default after noting the first refusal.
    pub fn take<T: Default>(&self, r: Result<T, crate::errors::BadInput>) -> T {
        match r {
            Ok(v) => v,
            Err(e) => {
                if self.0.get().is_none() {
                    self.0.set(Some(e));
                }
                T::default()
            }
        }
    }

    /// The refusal noted by `take`, if any.
    pub fn done(&self) -> Result<(), crate::errors::BadInput> {
        match self.0.get() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// `std::array::from_fn` for a closure that can refuse: the first refusal is returned (the
/// closure goes on with the default after it, as `Sticky` does).
fn try_from_fn<T: Default, const N: usize>(
    mut f: impl FnMut(usize) -> Result<T, crate::errors::Refused>,
) -> Result<[T; N], crate::errors::Refused> {
    let mut first = None;
    let out = std::array::from_fn(|k| match f(k) {
        Ok(v) => v,
        Err(e) => {
            if first.is_none() {
                first = Some(e);
            }
            T::default()
        }
    });
    match first {
        Some(e) => Err(e),
        None => Ok(out),
    }
}

/// Whether `f` holds for some item of `items` (stopping at the first, as `Iterator::any`),
/// `f` able to refuse.
fn try_any<I: Iterator>(
    items: I,
    mut f: impl FnMut(I::Item) -> Result<bool, crate::errors::Refused>,
) -> Result<bool, crate::errors::Refused> {
    for item in items {
        if f(item)? {
            return Ok(true);
        }
    }
    Ok(false)
}
