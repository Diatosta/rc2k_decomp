//! `drive::set_listener` (0x4565B7): the state the sounds are placed by, from the viewed
//! car (`g_carBlocks[dword_63DB9C]`), car block 0 (0x70F410 ..) and the view: its three
//! whole numbers +0x134 .. +0x13C as floats (0x5C5630 ..), `dword_5C2FD4` = (+0xABC >=
//! 0x10); the view's place negated and its turn's nine (0x5C2FA0 .. 0x5C2FCC); three of
//! 0x6415D0 .. (the first 0 without `dword_6238A8`) into 0x5C56D8 ..; six whole numbers of
//! the car's light record (`g_lightRecords` +0x40 .., by `race::car_file_number` of
//! `dword_70F410`) times 65536 over 1000 (two less `dword_613FC4`) into 0x608C60 ..; car
//! block 0's matrix (0x70FC2C) through `scene::multiply_matrices` with 0x70FC6C and the
//! view matrix into `g_madeObjectMatrix` (and a copy at 0x5EA0CC); the two points of
//! 0x608C60 .. turned by it, each with its x and then -x (`scene::rotate_point`,
//! `fistp`'d into 0x5C568C .. 0x5C56B8); the switches of 0x5C56BC .. 0x5C56E8 by
//! `dword_6415E0`, `dword_70FCEC`, `dword_70FCF0`; `dword_5C5680` from `dword_70F628`
//! (then cleared), `dword_5C5684`; `flt_5C5660` = `dword_70FA3C` / 2^29 + `flt_637644`;
//! two floats copied; with `dword_601024` set, `dword_5C5680` and `dword_5C5684` 0. By
//! the look of it the listener for the 3D sounds (a reading).
//!
//! Caller: `drive::share_all_cars` (0x456484), second. Refuses (before anything) a viewed
//! car past the eight blocks or with a null block, a car file number above 0x21, and a
//! light record figure whose times 65536 over 1000 leaves 32 bits (the original's `idiv`
//! traps).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::{ext_to_int, int_80, int_over_int_plus};
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::scene::matrix::multiply_matrices;
use crate::scene::rotate_point::rotate_point;
use crate::session::{Dword, Float};
use rc2k_core::drive::listener;

/// The car file's six figures times 65536 over 1000, or the refusal (before anything).
fn figures(shared: &Shared, viewed: u32) -> Result<[i32; 6], Refused> {
    listener::check_viewed(viewed, || shared.work.car_block(viewed as usize) != 0)?;
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

/// The Rust version. The registers it leaves are read by no caller (liveness: eax and
/// ebx after `share_all_cars`' call have no read point).
// Converted: `rc2k_core::drive::listener` (the matrices, turns and x87 sums stay).
pub fn set_listener(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    view: ListenerView<'_>,
) -> Result<(), Refused> {
    let physics = &race.physics;
    let f = figures(&race.shared, view.drawn_view)?;
    let car = crate::race::blocks_store::own_car();
    let s = crate::race::blocks_store::block_at(
        race.shared.work.car_block(view.drawn_view as usize),
        "a car's block",
    )?;
    let place: [u32; 3] = crate::session::read_all(&s.position);
    let strength: u32 = s.marks_count.get();
    MARKS_SETTINGS.listener_car_place[2].set_bits(x87::from_int(place[2] as i32));
    MARKS_SETTINGS.listener_car_place[1].set_bits(x87::from_int(place[1] as i32));
    MARKS_SETTINGS.listener_car_place[0].set_bits(x87::from_int(place[0] as i32));
    MARKS_SETTINGS
        .listener_near
        .set(u32::from(listener::car_ahead(strength)));
    for k in 0..3 {
        MARKS_SETTINGS.listener_place[k].set(view.view_place[k].get().wrapping_neg());
    }
    for (k, o) in listener::TURN_OFFSETS.into_iter().enumerate() {
        MARKS_SETTINGS.listener_turn[k].set_bits(view.view_turn_matrix[o as usize / 4].bits());
    }
    MARKS_SETTINGS.listener_figures[0].set(listener::first_figure(
        race.shared.vars.settings_words[0].get(),
        || view.sun.view_out.get(),
    ));
    MARKS_SETTINGS.listener_figures[1].set(view.sun.place[0].get());
    MARKS_SETTINGS.listener_figures[2].set(view.sun.place[1].get());
    let f = listener::listener_points(f, race.shared.physics_tables.ground_drop.get());
    for (k, &v) in f.iter().enumerate() {
        physics.camera.listener_points[k].set(v as u32);
    }
    // The car block 0's body matrix (0x70FC2C).
    for k in 0..16 {
        race.frame.matrices.turned_matrix[k].set_bits(car.body_matrix[k].bits());
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
        race.frame.matrices.listener_matrix[k]
            .set_bits(race.frame.matrices.made_object_matrix[k].bits());
    }
    let made = &race.frame.matrices.made_object_matrix;
    for (first, out, negate) in [
        (0, &MARKS_SETTINGS.listener_point_a_negated, true),
        (0, &MARKS_SETTINGS.listener_point_a, false),
        (3, &MARKS_SETTINGS.listener_point_b_negated, true),
        (3, &MARKS_SETTINGS.listener_point_b, false),
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
        &MARKS_SETTINGS.listener_switch_0,
        &MARKS_SETTINGS.listener_switch_1,
        &MARKS_SETTINGS.listener_switch_2,
        &MARKS_SETTINGS.listener_switch_3,
        &MARKS_SETTINGS.listener_switch_4,
        &MARKS_SETTINGS.listener_switch_5,
        &MARKS_SETTINGS.listener_switch_6,
        &MARKS_SETTINGS.listener_switch_7,
    ]
    .into_iter()
    .zip(switches)
    {
        cell.set(v);
    }
    let old = car.gear_changed_2.get();
    if old != 0 {
        MARKS_SETTINGS.listener_gear_changed.set(old);
    }
    car.gear_changed_2.set(0);
    MARKS_SETTINGS
        .listener_flag
        .set(u32::from(car.handbrake.get() != 0));
    MARKS_SETTINGS.listener_scale.set_bits(int_over_int_plus(
        car.steer_angle_visual.get() as i32,
        0x2000_0000,
        scene.lights.state.listener_camera.bits(),
    ));
    MARKS_SETTINGS
        .listener_float_a
        .set_bits(x87::loaded(car.engine_speed.bits()));
    MARKS_SETTINGS
        .listener_float_b
        .set_bits(x87::loaded(car.engine_speed_scaled.bits()));
    if physics.split_tables.still.get() != 0 {
        MARKS_SETTINGS.listener_flag.set(0);
        MARKS_SETTINGS.listener_gear_changed.set(0);
    }
    Ok(())
}
