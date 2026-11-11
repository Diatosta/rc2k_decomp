//! The race's effects: the weather (rain, snow), the wheels' spray, dust and mud, the drops
//! on the windscreen, the lens flare and the surface textures they need (files\m\: masks,
//! edges, snow, splines, surface data). `effects::init_marks` (0x4D12F7) sets them up as a
//! stage loads, `effects::run_frame` (0x4D1549) runs them each frame of the race.
//!
//! The game's own name for the step is
//! "init mark" (its loading log), and its data is `g_marks*` in IDA; the functions'
//! IDA names are `effects::*` like this module's.

pub mod add_drift_particle;
pub mod add_mark_slot;
pub mod adjugate3;
pub mod aim_vector;
#[cfg(all(test, target_arch = "x86"))]
mod asm_test;
pub mod begin_quads;
pub mod bend_points;
pub mod blend_corner;
pub mod blend_points;
pub mod blocks;
pub mod burst_particles;
pub mod cars_particles;
pub mod cell_x87;
pub mod clip;
pub mod clip_quad;
pub mod copy_first_point;
pub mod corners;
pub mod draw_cell_sprite;
pub mod draw_flare_part;
pub mod draw_frame;
pub mod draw_glow;
pub mod draw_lens_flare;
pub mod draw_mark_slots;
pub mod draw_overlays;
pub mod draw_particle;
pub mod draw_plane;
pub mod draw_pong_box;
pub mod draw_rain_streak;
pub mod draw_ribbon;
pub mod draw_shadow;
pub mod draw_snow;
pub mod draw_subcell_sprite;
pub mod draw_turned_sprite;
pub mod draw_windscreen_flake;
pub mod drift_offsets;
pub mod edges;
pub mod emit_body_contact_particles;
pub mod emit_body_particles;
pub mod emit_bursts;
pub mod emit_bursts_kind_3;
pub mod emit_car_particles;
pub mod emit_exhaust_smoke;
pub mod emit_rolling_particles;
pub mod emit_rolling_particles_rated;
pub mod emit_slip_spray;
pub mod emit_slip_spray_long;
pub mod emit_twin_particle_first;
pub mod emit_twin_particle_second;
pub mod emit_wheel_fade_particle;
pub mod emit_wheel_particles;
pub mod emitters;
pub mod flare_frame;
pub mod frame_marks;
pub mod frame_state;
pub mod init_mark_step;
pub mod init_marks;
pub mod item_walk;
pub mod jitter_point;
pub mod keep_vectors;
pub mod kept_turns;
pub mod kind_store;
pub mod kind_values;
pub mod lines;
pub mod load_mask_pictures;
pub mod make_fade_colours;
pub mod mark_slots;
pub mod mask_constants;
pub mod mask_picks;
pub mod mask_store;
pub mod mask_vars;
pub mod masks;
pub mod morph_object_points;
pub mod move_pong_ball;
pub mod move_pong_bat;
pub mod owner;
pub mod owner_groups;
pub mod paths;
pub mod pick_shapes;
pub mod pixels;
pub mod place_graded_points;
pub mod place_rain_view;
pub mod place_snow_view;
pub mod place_spline;
pub mod plain;
pub mod pong;
pub mod pong_ball_hits;
pub mod pong_bat_back;
pub mod pong_cell;
pub mod pong_digit;
pub mod pong_score;
pub mod pong_setup;
pub mod profiles;
pub mod rain;
pub mod ring_entries;
pub mod ring_frame;
pub mod ring_store;
pub mod rings;
pub mod run_frame;
pub mod run_particles;
pub mod run_pong;
pub mod run_rain;
pub mod run_ribbons;
pub mod run_ring;
pub mod run_shapes;
pub mod run_snow;
pub mod run_splines;
pub mod run_windscreen_flakes;
pub mod scale_axes;
pub mod scale_kept_vectors;
pub mod scale_setting;
pub mod scale_vectors_by;
pub mod set_quad_uvs;
pub mod set_up_surface;
pub mod setup;
pub mod shape_points;
pub mod shape_work;
pub mod shapes;
pub mod shapes_frame;
pub mod snow;
pub mod spans;
pub mod splines;
pub mod spread_point;
pub mod stage_record;
pub mod store;
pub mod store_data;
pub mod surface_store;
pub mod swap_mask;
pub mod tidy_marks;
pub mod tilt_shape;
pub mod trail_particles;
pub mod tuning;
pub mod turns;
#[cfg(all(test, target_arch = "x86"))]
mod typed_sync;
pub mod view_frame;
pub mod view_heading;
pub mod view_motion;
pub mod view_step;
pub mod view_vectors;
pub mod waves;
pub mod windscreen_drop_state;
pub mod windscreen_drop_vars;
pub mod windscreen_drops;
pub mod windscreen_drops_drift;
pub mod windscreen_drops_frame;
pub mod windscreen_drops_setup;
pub mod windscreen_drops_spawn;
pub mod windscreen_drops_top;
pub mod windscreen_flakes;
pub mod windscreen_flakes_frame;
pub mod windscreen_flakes_setup;
pub mod windscreen_flakes_spawn;
pub mod windscreen_flakes_store;
pub mod work_store;

/// The first refusal met inside a callback that cannot return one (a closure handed to a
/// core that asks for plain numbers): the callback answers the type's default after it, and
/// the caller asks `done` once the core has returned, so the refusal is not lost.
#[derive(Default)]
pub struct Sticky(std::cell::Cell<Option<crate::errors::BadInput>>);

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
pub fn try_from_fn<T: Default, const N: usize>(
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
pub fn try_any<I: Iterator>(
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
