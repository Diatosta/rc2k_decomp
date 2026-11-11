//! `drive::set_listener` (0x4565B7): the state the sounds are placed by, from the viewed
//! car (`g_carBlocks[`drawn_view`]`), car block 0 (0x70F410 ..) and the view: its three
//! whole numbers +0x134 .. +0x13C as floats (0x5C5630 ..), `listener_near` = (+0xABC >=
//! 0x10); the view's place negated and its turn's nine (0x5C2FA0 .. 0x5C2FCC); three of
//! 0x6415D0 .. (the first 0 without `settings_words`) into 0x5C56D8 ..; six whole numbers of
//! the car's light record (`g_lightRecords` +0x40 .., by `race::car_file_number` of
//! `CarsAndReplay.car_blocks.cars[0].car_kind`) times 65536 over 1000 (two less `ground_drop`) into 0x608C60 ..; car
//! block 0's matrix (0x70FC2C) through `scene::multiply_matrices` with 0x70FC6C and the
//! view matrix into `g_madeObjectMatrix` (and a copy at 0x5EA0CC); the two points of
//! 0x608C60 .. turned by it, each with its x and then -x (`scene::rotate_point`,
//! `fistp`'d into 0x5C568C .. 0x5C56B8); the switches of 0x5C56BC .. 0x5C56E8 by
//! `mirror_wanted`, `CarsAndReplay.car_blocks.cars[0].surface_noise_a`, `CarsAndReplay.car_blocks.cars[0].surface_noise_b`; `spline_gear_changed` from `CarsAndReplay.car_blocks.cars[0].gear_change_pending`
//! (then cleared), `spline_handbrake`; `listener_scale` = `CarsAndReplay.car_blocks.cars[0].steer_angle_visual` / 2^29 + `camera_shake_listener`;
//! two floats copied; with `still` set, `spline_gear_changed` and `spline_handbrake` 0. By
//! the look of it the listener for the 3D sounds (a reading).
//!
//! Caller: `drive::share_all_cars` (0x456484), second. Refuses (before anything) a viewed
//! car past the eight blocks or with a null block, a car file number above 0x21, and a
//! light record figure whose times 65536 over 1000 leaves 32 bits (the original's `idiv`
//! traps).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::{ext_to_int, int_80};
use crate::math::x87_float as bh;
use crate::race::effects_settings_store::EFFECTS_SETTINGS;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::scene::matrix::multiply_matrices;
use crate::scene::rotate_point::rotate_point;
use crate::session::{Dword, Float};
use rc2k_core::drive::listener;

/// The car file's six figures times 65536 over 1000, or the refusal (before anything).
fn figures(shared: &RaceShared, viewed: u32) -> Result<[i32; 6], Refused> {
    listener::check_viewed(viewed, || shared.work.has_car_block(viewed as usize))?;
    let n = car_file_number(crate::race::blocks_store::own_car().car_kind.get());
    rc2k_core::drive::sharing::check_light_record(n)?;
    let mut out = [0i32; 6];
    for (k, o) in out.iter_mut().enumerate() {
        // `g_lightRecords + 0x40` of the car file's record: `axles[5 + k]`.
        *o = listener::light_figure(LIGHT_RECORDS.records[n as usize].axles[5 + k].get())?;
    }
    Ok(out)
}

/// The view the listener is placed by: its number, its matrix, its turn and its place (the
/// original's `g_drawnView`, `g_viewMatrix`, `g_viewTurnMatrix`, `g_viewPlace`, as the last
/// view drawn left them).
#[derive(Clone, Copy)]
pub struct ListenerView<'a> {
    /// Which car's view (0 or 1).
    pub drawn_view: u32,
    pub view_matrix: &'a [Float; 16],
    pub view_turn_matrix: &'a [Float; 16],
    pub view_place: &'a [Dword; 3],
    /// The view's sun (its place and the view's value, `ViewSun`).
    pub sun: &'a crate::scene::polygon_work::ViewSun,
}

impl<'a> ListenerView<'a> {
    /// A view's own, from its overlays (`race::run_view_marks`).
    pub fn of(view: &'a crate::race::view_scratch::ViewScratch, race: &Race) -> Self {
        Self {
            drawn_view: view.drawn_view(race),
            view_matrix: &view.matrices.view_matrix,
            view_turn_matrix: &view.matrices.view_turn_matrix,
            view_place: &view.state.view_place,
            sun: &view.polygon.sun,
        }
    }

    /// Before any view of the race (`drive::race_init`'s settling steps): what the last view
    /// drawn left, as the original read; the sun the first player's view's (the original's
    /// was the last view drawn's: all 0 before a session's first race either way).
    pub fn before_any_view(race: &'a Race) -> Self {
        let frame = &race.frame;
        Self {
            drawn_view: race.shared.view.drawn_view.get(),
            view_matrix: &frame.matrices.view_matrix,
            view_turn_matrix: &frame.matrices.view_turn_matrix,
            view_place: &frame.view_and_records.view_vars.view_place,
            sun: &crate::race::view_scratch::main_view(0).polygon.sun,
        }
    }
}

/// The registers the original leaves are read by no caller.
// Converted: `rc2k_core::drive::listener` (the matrices, turns and x87 sums stay).
pub fn set_listener(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    view: ListenerView<'_>,
) -> Result<(), Refused> {
    let physics = &race.physics;
    let f = figures(&race.shared, view.drawn_view)?;
    let car = crate::race::blocks_store::own_car();
    let s = race
        .shared
        .work
        .block_car(view.drawn_view as usize, "a car's block")?;
    let place: [u32; 3] = crate::session::read_all(&s.position);
    let strength: u32 = s.marks_count.get();
    EFFECTS_SETTINGS.listener_car_place[2].set(x87::from_int(place[2] as i32));
    EFFECTS_SETTINGS.listener_car_place[1].set(x87::from_int(place[1] as i32));
    EFFECTS_SETTINGS.listener_car_place[0].set(x87::from_int(place[0] as i32));
    EFFECTS_SETTINGS
        .listener_near
        .set(u32::from(listener::car_ahead(strength)));
    for k in 0..3 {
        EFFECTS_SETTINGS.listener_place[k].set(view.view_place[k].get().wrapping_neg());
    }
    for (k, o) in listener::TURN_OFFSETS.into_iter().enumerate() {
        EFFECTS_SETTINGS.listener_turn[k].set(view.view_turn_matrix[o as usize / 4].get());
    }
    EFFECTS_SETTINGS.listener_figures[0].set(listener::first_figure(
        race.shared.vars.settings_words[0].get(),
        || view.sun.view_out.get(),
    ));
    EFFECTS_SETTINGS.listener_figures[1].set(view.sun.place[0].get());
    EFFECTS_SETTINGS.listener_figures[2].set(view.sun.place[1].get());
    let f = listener::listener_points(f, race.shared.physics_tables.ground_drop.get());
    for (k, &v) in f.iter().enumerate() {
        physics.camera.listener_points[k].set(v as u32);
    }
    // The car block 0's body matrix (0x70FC2C).
    for k in 0..16 {
        race.frame.matrices.turned_matrix[k].set(car.body_matrix[k].get());
    }
    multiply_matrices(
        &race.frame.matrices.turned_matrix,
        &car.place_matrix,
        &race.frame.matrices.lit_matrix,
    );
    multiply_matrices(
        &race.frame.matrices.lit_matrix,
        view.view_matrix,
        &race.frame.matrices.made_object_matrix,
    );
    for k in 0..16 {
        race.frame.matrices.listener_matrix[k].set(race.frame.matrices.made_object_matrix[k].get());
    }
    let made = &race.frame.matrices.made_object_matrix;
    for (first, out, negate) in [
        (0, &EFFECTS_SETTINGS.light_point_a_negated, true),
        (0, &EFFECTS_SETTINGS.light_point_a, false),
        (3, &EFFECTS_SETTINGS.light_point_b_negated, true),
        (3, &EFFECTS_SETTINGS.light_point_b, false),
    ] {
        let p = [
            int_80(f[first], negate),
            int_80(f[first + 1], false),
            int_80(f[first + 2], false),
        ];
        let r = rotate_point(made, p);
        for (cell, e) in out.iter().zip(r.iter()) {
            cell.set(ext_to_int(*e) as u32);
        }
    }
    let switches = listener::switches(
        race.frame.frame_work.mirror_wanted.get() != 0,
        car.surface_noise_a.get(),
        car.surface_noise_b.get(),
    );
    for (cell, v) in [
        &EFFECTS_SETTINGS.windscreen_drops_rate_factor,
        &EFFECTS_SETTINGS.rain_drops,
        &EFFECTS_SETTINGS.windscreen_drops_wanted,
        &EFFECTS_SETTINGS.windscreen_flakes_rate_factor,
        &EFFECTS_SETTINGS.snow_flakes,
        &EFFECTS_SETTINGS.mask_rate_switch,
        &EFFECTS_SETTINGS.windscreen_wet,
        &EFFECTS_SETTINGS.mask_number,
    ]
    .into_iter()
    .zip(switches)
    {
        cell.set(v);
    }
    let old = car.gear_change_pending.get();
    if old != 0 {
        EFFECTS_SETTINGS.spline_gear_changed.set(old);
    }
    car.gear_change_pending.set(0);
    EFFECTS_SETTINGS
        .spline_handbrake
        .set(u32::from(car.handbrake.get() != 0));
    EFFECTS_SETTINGS.listener_scale.set(bh::stf(
        bh::int(car.steer_angle_visual.get() as i32) / bh::int(0x2000_0000)
            + bh::ld(scene.lights.state.camera_shake_listener.get()),
    ));
    EFFECTS_SETTINGS
        .engine_speed_copy
        .set(x87::loaded(&car.engine_speed));
    EFFECTS_SETTINGS
        .engine_speed_scaled_copy
        .set(x87::loaded(&car.engine_speed_scaled));
    if physics.split_tables.still.get() != 0 {
        EFFECTS_SETTINGS.spline_handbrake.set(0);
        EFFECTS_SETTINGS.spline_gear_changed.set(0);
    }
    Ok(())
}
