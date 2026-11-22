//! The state the sounds are placed by (by the look of it the listener for the 3D sounds, a
//! reading), from the viewed car, car block 0 and the view: the car's position, the view's place
//! and turn, the car's light record points turned by its matrix and the view's, the sound
//! switches from the mirror and surface noise, `listener_scale` from the visual steer angle plus
//! `camera_shake_listener`; with `still` set, the gear-change and handbrake sounds are off.
//! Refuses (before anything) a viewed car past the eight blocks or with a null block, a car file
//! number above 0x21, and a light record figure whose times 65536 over 1000 leaves 32 bits.

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87::{ext_to_int, int_80};
use crate::math::x87_float as bh;
use crate::physics::figures::GROUND_DROP;
use crate::race::mud::car_file_number;
use crate::race::owner::Race;
use crate::scene::matrix::{Matrix, product};
use rc2k_core::drive::listener;
use rc2k_core::net::net_race::NetRace;

/// The car file's six figures times 65536 over 1000, or the refusal (before anything).
fn figures(race: &Race, net_race: Option<NetRace>, viewed: u32) -> Result<[i32; 6], Refused> {
    let shared = &race.shared;
    listener::check_viewed(viewed, || shared.work.get().has_car_block(viewed as usize))?;
    let n = car_file_number(race.kind_tables(), shared.own_car(net_race).car_kind.get());
    rc2k_core::drive::sharing::check_light_record(n)?;
    let mut out = [0i32; 6];
    for (k, o) in out.iter_mut().enumerate() {
        // The car file's light record's `axles[5 + k]`.
        *o = listener::light_figure(
            race.light_records()
                .read(|l| l.records[n as usize].axles[5 + k]),
        )?;
    }
    Ok(out)
}

/// The view the listener is placed by: its number, its matrix, its turn and its place, as the
/// last view drawn left them.
#[derive(Clone, Copy)]
pub struct ListenerView<'a> {
    /// Which car's view (0 or 1).
    pub drawn_view: u32,
    pub view_matrix: Matrix,
    pub view_turn_matrix: Matrix,
    pub view_place: [u32; 3],
    /// The view's sun (its place and the view's value).
    pub sun: &'a crate::scene::polygon_work_store::ViewSun,
}

impl<'a> ListenerView<'a> {
    /// A view's own, from its overlays (`race::run_view_marks`).
    pub fn of(view: &'a crate::race::view_scratch::ViewScratch, race: &Race) -> Self {
        Self {
            drawn_view: view.drawn_view(race),
            view_matrix: view.matrices.borrow().view_matrix,
            view_turn_matrix: view.matrices.borrow().view_turn_matrix,
            view_place: view.state.borrow().view_place,
            sun: &view.polygon.sun,
        }
    }

    /// Before any view of the race (`drive::race_init`'s settling steps): what the last view
    /// drawn left; the sun the first player's view's (all 0 before a session's first race
    /// either way).
    pub fn before_any_view(race: &'a Race, scene: &'a crate::scene::owner::Scene) -> Self {
        let frame = &race.frame;
        let kept = *frame.view_kept.borrow();
        Self {
            drawn_view: race.shared.view.get().drawn_view.get(),
            view_matrix: kept.view_matrix,
            view_turn_matrix: kept.view_turn_matrix,
            view_place: crate::memory::session::read_bits(
                &frame.view_and_records.get().view_vars.view_place,
            ),
            sun: &scene.scratch.main_view(0).polygon.sun,
        }
    }
}

pub fn set_listener(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    view: ListenerView<'_>,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let physics = &race.physics;
    let net_race = net_race.get();
    let f = figures(race, net_race, view.drawn_view)?;
    let car = race.shared.own_car(net_race);
    let s = race
        .shared
        .race_car_or(view.drawn_view as usize, "a car's block")?;
    let place: [u32; 3] = s.position.bits();
    let strength: u32 = race
        .car_blocks()
        .parts_of(
            crate::memory::drawn_table::DrawnTable::Store(&race.shared.car_parts),
            s,
        )
        .marks_count;
    race.effects_settings().listener_car_place[2].set(x87::from_int(place[2] as i32));
    race.effects_settings().listener_car_place[1].set(x87::from_int(place[1] as i32));
    race.effects_settings().listener_car_place[0].set(x87::from_int(place[0] as i32));
    race.effects_settings()
        .listener_near
        .set(u32::from(listener::car_ahead(strength)));
    for (cell, place) in race
        .effects_settings()
        .listener_place
        .iter()
        .zip(view.view_place)
    {
        cell.set(place.wrapping_neg());
    }
    for (k, figure) in listener::TURN_FIGURES.into_iter().enumerate() {
        race.effects_settings().listener_turn[k].set(view.view_turn_matrix[figure]);
    }
    let sun = *view.sun.borrow();
    race.effects_settings().listener_figures[0].set(listener::first_figure(
        race.shared.vars.get().settings_words[0].get(),
        || sun.view_out,
    ));
    race.effects_settings().listener_figures[1].set(sun.place[0]);
    race.effects_settings().listener_figures[2].set(sun.place[1]);
    let f = listener::listener_points(f, GROUND_DROP);
    for (k, &v) in f.iter().enumerate() {
        physics.camera_mut().listener_points[k] = v as u32;
    }
    // The car block 0's body matrix, turned by the car's turn and then by the view's matrix.
    let (turned, place_matrix) = {
        let own_parts = race.car_blocks().parts_of(
            crate::memory::drawn_table::DrawnTable::Store(&race.shared.car_parts),
            car,
        );
        (own_parts.body_matrix, own_parts.place_matrix)
    };
    let view_matrix = view.view_matrix;
    let made = {
        let mut kept = race.frame.matrices.borrow_mut();
        kept.turned_matrix = turned;
        kept.lit_matrix = product(&turned, &place_matrix, kept.lit_matrix);
        kept.made_object_matrix = product(&kept.lit_matrix, &view_matrix, kept.made_object_matrix);
        kept.listener_matrix = kept.made_object_matrix;
        kept.made_object_matrix
    };
    for (first, out, negate) in [
        (0, &race.effects_settings().light_point_a_negated, true),
        (0, &race.effects_settings().light_point_a, false),
        (3, &race.effects_settings().light_point_b_negated, true),
        (3, &race.effects_settings().light_point_b, false),
    ] {
        let p = [
            int_80(f[first], negate),
            int_80(f[first + 1], false),
            int_80(f[first + 2], false),
        ];
        let r = crate::scene::rotate_point::rotate_point_by(&made, p);
        for (cell, e) in out.iter().zip(r.iter()) {
            cell.set(ext_to_int(*e) as u32);
        }
    }
    let switches = listener::switches(
        race.frame.frame_work.borrow().mirror_wanted,
        car.surface_noise_a.get(),
        car.surface_noise_b.get(),
    );
    for (cell, v) in [
        &race.effects_settings().windscreen_drops_rate_factor,
        &race.effects_settings().rain_drops,
        &race.effects_settings().windscreen_drops_wanted,
        &race.effects_settings().windscreen_flakes_rate_factor,
        &race.effects_settings().snow_flakes,
        &race.effects_settings().mask_rate_switch,
        &race.effects_settings().windscreen_wet,
        &race.effects_settings().mask_number,
    ]
    .into_iter()
    .zip(switches)
    {
        cell.set(v);
    }
    let old = car.gear_change_pending.get();
    if old != 0 {
        race.effects_settings().spline_gear_changed.set(old);
    }
    car.gear_change_pending.set(0);
    race.effects_settings()
        .spline_handbrake
        .set(u32::from(car.handbrake.get() != 0));
    race.effects_settings().listener_scale.set(bh::stf(
        bh::int(car.steer_angle_visual.get() as i32) / bh::int(0x2000_0000)
            + bh::ld(scene.line_and_shake().camera_shake_listener),
    ));
    race.effects_settings()
        .engine_speed_copy
        .set(x87::loaded(&car.engine_speed));
    race.effects_settings()
        .engine_speed_scaled_copy
        .set(x87::loaded(&car.engine_speed_scaled));
    if physics.split_tables().still != 0 {
        race.effects_settings().spline_handbrake.set(0);
        race.effects_settings().spline_gear_changed.set(0);
    }
    Ok(())
}
