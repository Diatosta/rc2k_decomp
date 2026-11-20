//! `effects::run_ring`: the emitters' ring's frame. Each entry from the ring's cursor along its
//! links is stepped by its kind: the life runs down by the frame's ticks (dead when it borrows, or
//! below the ground), kinds are drawn as cell or subcell sprites when the settings' flag for the
//! entry's car is set, then the angle, place, motion, dampings, size and alpha are advanced (kinds
//! differ in their gains and in how the size is pulled toward the object). A dead entry is moved to
//! just before the cursor.
//!
//! The longer x87 runs are plain Rust on the x87 model (`math::x87_float`), over the entries and
//! the game's variables. Refuses, before anything, null settings, a null entry or link on the way,
//! a null object where one is read, and the draws' own refusals for an entry that may be drawn.

use crate::effects::emitters::matrix_float;
use crate::effects::mask_store::Masks;
use crate::effects::ring_entries::{Entry, walk_ring_copies};
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::effects::setup::MarksClock;
use crate::effects::tuning::MarksFloats;
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings};

/// The settings' flag for a ring entry's tag (its +8, a car's number times the car record's
/// length, as `cars_particles` makes it): the first word of that car's record.
fn entry_flag(marks: &EffectsSettings, tag: u32) -> Result<u32, BadInput> {
    let record = crate::race::effects_settings_store::CAR_LEN as u32;
    if !tag.is_multiple_of(record) {
        return Err(BadInput("a ring entry's tag that is no car's"));
    }
    marks
        .cars
        .get((tag / record) as usize)
        .map(|car| car.streaks_on.get())
        .ok_or(BadInput("a ring entry's tag past the cars"))
}

fn k(cell: &f32) -> u32 {
    cell.to_bits()
}

/// The car a ring entry is on (its +0x68, a `effects_settings_store::car_handle`).
fn entry_car(
    en: &Entry,
    effects: &crate::effects::owner::Effects,
) -> Result<&'static CarEffects, Refused> {
    effects
        .car_record(en.car)
        .ok_or_else(|| BadInput("a ring entry without its car").into())
}

/// The place less the object's (`[p + 0x30]` ..) into `RingWork.away[0]` .., turned by the object's
/// matrix (`m`), |x| and |z| rounded into `RingWork.extent[0]` / `RingWork.extent[1]`.
fn box_(
    ring: &Ring,
    en: &Entry,
    whole: &[crate::memory::session::PlainFloat; 16],
    car: &CarEffects,
) {
    box_x87(en, &ring.work, whole, car);
}

/// The car's whole matrix (its block's); refused when there is none.
fn whole_of(
    car: &CarEffects,
    effects: &crate::effects::owner::Effects,
) -> Result<&'static [crate::memory::session::PlainFloat; 16], BadInput> {
    effects
        .car_blocks()
        .ok()
        .and_then(|blocks| blocks.of(car.whole_block.get()))
        .map(|b| &b.whole_matrix)
        .ok_or(BadInput("a ring entry without its object"))
}

/// That way made one long (`MarksFloats.one` over its length), dotted with the object's +0x4C ..
/// into `RingWork.facing`.
fn facing(ring: &Ring, car: &CarEffects, effects: &crate::effects::owner::Effects) {
    facing_x87(car, &effects.setup.floats, &ring.work);
}

/// The place less the object's into `RingWork.away[0]` .., its length (kept, 80 bits, at `l`)
/// less a constant into `RingWork.near`.
fn pull_length(
    ring: &Ring,
    en: &Entry,
    whole: &[crate::memory::session::PlainFloat; 16],
    length: &mut [u8; 10],
) {
    pull_length_x87(en, whole, length, &ring.work, &ring.kinds);
}

/// The way made one long by the kept length, the object's +0x4C .. over its +0x88, minus their dot
/// into `RingWork.facing`.
fn pull_facing(
    ring: &Ring,
    car: &CarEffects,
    length: &[u8; 10],
    effects: &crate::effects::owner::Effects,
) {
    pull_facing_x87(car, length, &effects.setup.floats, &ring.work);
}

/// The entry's +0x64 pulled toward `MarksFloats.fixed_16_scale` (rounded into `RingWork.whole`).
fn pull_strength(
    ring: &Ring,
    en: &mut Entry,
    car: &CarEffects,
    effects: &crate::effects::owner::Effects,
) {
    let setup = &effects.setup;
    pull_strength_x87(
        en,
        car,
        &setup.clock,
        &setup.floats,
        &ring.work,
        &ring.kinds,
    );
}

/// The place turned by the object's matrix (`m`), |x| and |z| into `RingWork.extent[0]` /
/// `RingWork.extent[1]`.
fn box_place(ring: &Ring, en: &Entry, car: &CarEffects) {
    box_place_x87(en, &ring.work, car);
}

/// The kinds' constants: (the y gain, the three dampings, the size's rate).
struct Kind<'a> {
    gain: Option<&'a f32>,
    damp: [&'a f32; 3],
    pull: Option<&'a f32>,
}

impl<'a> Kind<'a> {
    /// The size's rate (the kinds that pull the size have one); refused for one that has none.
    fn pull(&self) -> Result<&'a f32, BadInput> {
        self.pull
            .ok_or(BadInput("a ring kind that pulls the size without a rate"))
    }

    /// The y gain (the kinds that gain have one; as `pull`).
    fn gain(&self) -> Result<&'a f32, BadInput> {
        self.gain
            .ok_or(BadInput("a ring kind that gains without a gain"))
    }
}

fn k6(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: None,
        damp: [&rk.k6_damp[0], &rk.k6_damp[1], &rk.k6_damp[2]],
        pull: Some(&rk.k6_pull),
    }
}
fn k15(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: None,
        damp: [&rk.k15_damp[0], &rk.k15_damp[1], &rk.k15_damp[2]],
        pull: Some(&rk.k15_pull),
    }
}
fn k1(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: None,
        damp: [&rk.k1_damp[0], &rk.k1_damp[1], &rk.k1_damp[2]],
        pull: Some(&rk.k1_pull),
    }
}
fn kd(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: Some(&rk.kd_gain),
        damp: [&rk.kd_damp[0], &rk.kd_damp[1], &rk.kd_damp[2]],
        pull: Some(&rk.kd_pull),
    }
}
fn k13(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: Some(&rk.k13_gain),
        damp: [&rk.k13_damp[0], &rk.k13_damp[1], &rk.k13_damp[2]],
        pull: None,
    }
}
fn k16(rk: &RingKinds) -> Kind<'_> {
    Kind {
        gain: Some(&rk.k16_gain),
        damp: [&rk.k16_damp[0], &rk.k16_damp[1], &rk.k16_damp[2]],
        pull: None,
    }
}

/// The life and the ground: whether the entry dies.
fn aged(
    en: &mut Entry,
    ground: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<bool, Refused> {
    Ok({
        let t: u32 = effects.setup.clock.frame_ticks.get();
        let life = en.life;
        en.life = life.wrapping_sub(t);
        if life < t {
            return Ok(true);
        }
        ground && x87::compare_status(en.place[1], en.ground_y) == 0x100
    })
}

/// The place on the object's (unrounded sums).
fn on_object(
    en: &Entry,
    effects: &crate::effects::owner::Effects,
) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        let m = entry_car(en, effects)?.whole(effects.car_blocks()?)?;
        [
            x87::add_80(en.place[0], matrix_float(m, 0x30)?),
            x87::add_80(en.place[1], matrix_float(m, 0x34)?),
            x87::add_80(en.place[2], matrix_float(m, 0x38)?),
        ]
    })
}

fn at_place(en: &Entry) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        [
            extended(en.place[0]),
            extended(en.place[1]),
            extended(en.place[2]),
        ]
    })
}

/// The settings' flag for the entry.
fn shown(effects: &crate::effects::owner::Effects, en: &Entry) -> Result<bool, Refused> {
    Ok(entry_flag(effects.settings()?, en.emitter_tag)? != 0)
}

/// The alpha (top byte) as a whole number (through `RingWork.whole`).
fn whole_alpha(ring: &Ring, en: &Entry) -> Result<u32, Refused> {
    Ok({
        let a = x87::to_int(en.alpha);
        ring.work.whole.set(a as u32);
        (a as u32) << 24
    })
}

/// The size as a whole number (through `RingWork.whole`).
fn whole(ring: &Ring, v: i32) -> u32 {
    ring.work.whole.set(v as u32);
    v as u32
}

/// The angle on and the place moved.
fn spin_and_move(
    en: &mut Entry,
    spin: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let t: i32 = setup.clock.frame_ticks.get() as i32;
        if spin {
            let turn =
                x87::int_times_to_int(en.angle_rate as i32, &setup.clock.ticks_scaled) as u32;
            en.angle = en.angle.wrapping_add(turn) & 0xFFF;
        }
        for axis in 0..3 {
            // The place plus the motion times the ticks.
            en.place[axis] = x87::plus_times_int(en.place[axis], en.motion[axis], t);
        }
    }
    Ok(())
}

fn gain_y(
    en: &mut Entry,
    k_addr: &f32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        en.motion[1] = x87::plus_times_int(en.motion[1], k(k_addr), t);
    }
    Ok(())
}

/// The three dampings (a part that changes sign stops at 0).
fn damp3(
    ring: &Ring,
    en: &mut Entry,
    ks: [&f32; 3],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        for (axis, kk) in ks.into_iter().enumerate() {
            let old = en.motion[axis];
            ring.work.whole.set(old.to_bits());
            let new = f32::from_bits(bh::st(
                bh::ld(old) - bh::ld(old) * bh::ld(k(kk)) * bh::int(t),
            ));
            en.motion[axis] = new;
            if (old.to_bits() ^ new.to_bits()) & 0x8000_0000 != 0 {
                en.motion[axis] = 0.0;
            }
        }
    }
    Ok(())
}

/// The size pulled toward +0x70.
fn pull_size(
    ring: &Ring,
    en: &mut Entry,
    k_addr: &f32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        let (w, f) = x87::approach(en.size_goal as i32, en.size, k(k_addr), t);
        ring.work.whole.set(w as u32);
        en.size = f.to_bits();
        if w > en.size_goal as i32 {
            en.size = x87::from_int(en.size_goal as i32).to_bits();
        }
    }
    Ok(())
}

fn fade(en: &mut Entry, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        en.alpha = bh::st(bh::ld(en.alpha) - bh::ld(en.alpha_fade.to_bits()) * bh::int(t));
    }
    Ok(())
}

/// The box test of kinds 6 and 0x15: whether the size is pulled.
fn outside_and_away(
    ring: &Ring,
    en: &Entry,
    effects: &crate::effects::owner::Effects,
) -> Result<bool, Refused> {
    Ok({
        let car = entry_car(en, effects)?;
        box_(ring, en, whole_of(car, effects)?, car);
        let outside = (en.box_limit[0] as i32) < (ring.work.extent[0].get() as i32)
            || (en.box_limit[1] as i32) < (ring.work.extent[1].get() as i32);
        if !outside {
            return Ok(false);
        }
        facing(ring, car, effects);
        crate::math::x87::bits_at_or_below(ring.work.facing.get(), 0.0)
    })
}

/// What a draw shows: the (sub)cell, its turn, alpha and size.
struct Sprite {
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
}

/// The draws: 1 `draw_cell_sprite`, 2 its twin (the same Rust version), 3
/// `draw_subcell_sprite`; `cell` is the (sub)cell, `size` the size.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's registers, the effects' groups and the owners"
)]
fn draw(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    place: u32,
    sprite: Sprite,
    e: u32,
    p: [[u8; 10]; 3],
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    use crate::effects::{draw_cell_sprite, draw_subcell_sprite};
    let Sprite {
        cell,
        angle,
        alpha,
        size,
    } = sprite;
    {
        match place {
            3 => draw_subcell_sprite::draw_subcell_sprite(
                buffer, masks, ring, scene, effects, cell, size, e, p, view,
            ),
            _ => draw_cell_sprite::draw_cell_sprite(
                buffer, masks, ring, scene, effects, cell, angle, alpha, size, e, p, view,
            ),
        }
    }
}

/// Kinds 5 and 0xB (draw 1 or 2; their y gain, size and alpha gains): whether it dies.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
fn drift_kind(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    e: u32,
    en: &mut Entry,
    place: u32,
    ks: [&f32; 3],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    {
        if aged(en, true, effects)? {
            return Ok(true);
        }
        if shown(effects, en)? {
            let p = on_object(en, effects)?;
            let [cell, angle, alpha, size] = [en.cell, en.angle, en.alpha, en.size];
            draw(
                buffer,
                masks,
                ring,
                scene,
                effects,
                place,
                Sprite {
                    cell,
                    angle,
                    alpha,
                    size,
                },
                e,
                p,
                view,
            )?;
        }
        spin_and_move(en, true, effects)?;
        gain_y(en, ks[0], effects)?;
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        en.size = bh::to_int(bh::ld(k(ks[1])) * bh::int(t) + bh::int(en.size as i32)) as u32;
        en.alpha = x87::int_times_int_plus_int(k(ks[2]) as i32, t, en.alpha as i32) as u32;
    }
    Ok(false)
}

/// Kinds 6, 0x15, 1, 0xD, 0x13, 0x16 (draw 1): whether it dies.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
fn fading_kind(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    e: u32,
    en: &mut Entry,
    kind: u32,
    c: &Kind<'_>,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    {
        if aged(en, true, effects)? {
            return Ok(true);
        }
        if (en.alpha as i32) < 0 {
            en.alpha = 0;
        }
        if shown(effects, en)? {
            let p = if kind == 0xD {
                on_object(en, effects)?
            } else {
                at_place(en)?
            };
            let [cell, angle] = [en.cell, en.angle];
            let alpha = whole_alpha(ring, en)?;
            let size = if kind == 1 {
                bh::to_int(
                    bh::ld(en.size) + bh::ld(en.size_factor) * bh::ld(k(&ring.kinds.k1_size)),
                )
            } else {
                x87::to_int(en.size)
            };
            let size = whole(ring, size);
            draw(
                buffer,
                masks,
                ring,
                scene,
                effects,
                1,
                Sprite {
                    cell,
                    angle,
                    alpha,
                    size,
                },
                e,
                p,
                view,
            )?;
        }
        spin_and_move(en, true, effects)?;
        if c.gain.is_some() && kind == 0xD {
            gain_y(en, c.gain()?, effects)?;
        }
        damp3(ring, en, c.damp, effects)?;
        match kind {
            6 | 0x15 => {
                if outside_and_away(ring, en, effects)? {
                    pull_size(ring, en, c.pull()?, effects)?;
                }
                fade(en, effects)?;
            }
            1 => {
                pull_size(ring, en, c.pull()?, effects)?;
                fade(en, effects)?;
                pull_toward(ring, en, effects)?;
            }
            0xD => {
                pull_size(ring, en, c.pull()?, effects)?;
                fade(en, effects)?;
            }
            _ => {
                fade(en, effects)?;
                let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
                en.size = x87::plus_times_int(en.size, k(c.gain()?), t).to_bits();
            }
        }
    }
    Ok(false)
}

/// Kind 1's pull of its +0x64 when near and facing.
fn pull_toward(
    ring: &Ring,
    en: &mut Entry,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let car = entry_car(en, effects)?;
        let mut length = [0u8; 10];
        pull_length(ring, en, whole_of(car, effects)?, &mut length);
        if ring.work.near.get().is_sign_negative() {
            return Ok(());
        }
        pull_facing(ring, car, &length, effects);
        if crate::math::x87::bits_at_or_below(ring.work.facing.get(), 0.0) {
            return Ok(());
        }
        pull_strength(ring, en, car, effects);
        if (ring.work.whole.get() as i32) > (setup.floats.pull_limit.get()) {
            en.size_factor = x87::loaded(&setup.floats.fixed_16_scale);
        }
    }
    Ok(())
}

/// Kinds 0x10, 0x11 (draw 3): boxed, no ground test of their own first. Whether it dies.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
fn boxed_kind(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    e: u32,
    en: &mut Entry,
    gain: &f32,
    both: bool,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    if aged(en, true, effects)? {
        return Ok(true);
    }
    spin_and_move(en, false, effects)?;
    gain_y(en, gain, effects)?;
    if shown(effects, en)? {
        let p = on_object(en, effects)?;
        let size = en.size;
        let f = bh::to_int(
            bh::int(en.life as i32)
                * bh::ld(effects.setup.floats.twentieth.get())
                * bh::ld(en.cell_rate.to_bits()),
        );
        ring.work.whole.set(f as u32);
        let subcell = ((f as u32) & 3).wrapping_add(en.cell);
        draw(
            buffer,
            masks,
            ring,
            scene,
            effects,
            3,
            Sprite {
                cell: subcell,
                angle: 0,
                alpha: 0,
                size,
            },
            e,
            p,
            view,
        )?;
    }
    box_place(ring, en, entry_car(en, effects)?);
    let mut outside = (en.box_limit[0] as i32) < (ring.work.extent[0].get() as i32);
    if !outside && both {
        outside = (en.box_limit[1] as i32) < (ring.work.extent[1].get() as i32);
    }
    if !outside && x87::compare_status(en.place[1], &ring.work.top) == 0 {
        return Ok(true);
    }
    Ok(x87::compare_status(en.place[1], en.ground_y) == 0x100)
}

pub(super) fn run_ring(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if effects.settings().is_err() {
        return Err(BadInput("no marks settings").into());
    }
    walk_ring_copies(ring, |e, en| {
        Ok(match en.kind {
            5 => drift_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                1,
                [
                    &ring.kinds.drift5[0],
                    &ring.kinds.drift5[1],
                    &ring.kinds.drift5[2],
                ],
                scene,
                effects,
                view,
            )?,
            0xB => drift_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                2,
                [
                    &ring.kinds.drift_b[0],
                    &ring.kinds.drift_b[1],
                    &ring.kinds.drift_b[2],
                ],
                scene,
                effects,
                view,
            )?,
            6 => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                6,
                &k6(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            0x15 => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                0x15,
                &k15(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            1 => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                1,
                &k1(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            0xD => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                0xD,
                &kd(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            0x13 => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                0x13,
                &k13(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            0x16 => fading_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                0x16,
                &k16(&ring.kinds),
                scene,
                effects,
                view,
            )?,
            0x10 => boxed_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                &ring.kinds.k10_gain,
                true,
                scene,
                effects,
                view,
            )?,
            0x11 => boxed_kind(
                buffer,
                masks,
                ring,
                e,
                en,
                &ring.kinds.k11_gain,
                false,
                scene,
                effects,
                view,
            )?,
            _ => false,
        })
    })
}

use crate::effects::plain::{ld_cell, ldi_cell};
use crate::math::x87_float as bh;

/// `fld dword ptr` of a float of a car record in the settings.
fn car_float(cell: &impl crate::memory::session::Cell32) -> bh::X {
    cell.ld()
}

/// The car's 3 x 3 turning matrix (the record's `matrix`, 4 floats a row): cell `n`.
fn turn(car: &CarEffects, n: usize) -> bh::X {
    car_float(&car.matrix[n])
}

/// `box_`'s arithmetic as plain Rust on the x87 model, in the same order.
fn box_x87(
    en: &Entry,
    w: &RingWork,
    whole: &[impl crate::memory::session::Cell32; 16],
    car: &CarEffects,
) {
    let t0 = bh::ld(en.place[0]);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].st(t1);
    let t2 = bh::ld(en.place[1]);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].st(t3);
    let t4 = bh::ld(en.place[2]);
    let t5 = t4 - ld_cell(&whole[14]);
    w.away[2].st(t5);
    let t6 = w.away[2].ld();
    let t7 = w.away[1].ld();
    let t8 = w.away[0].ld();
    let t9 = turn(car, 0);
    let t10 = t9 * t8;
    let t11 = turn(car, 4);
    let t12 = t11 * t7;
    let t13 = turn(car, 8);
    let t14 = t13 * t6;
    let t15 = t12 + t14;
    let t16 = t10 + t15;
    let t17 = turn(car, 1);
    let t18 = t17 * t8;
    let t19 = turn(car, 5);
    let t20 = t19 * t7;
    let t21 = turn(car, 9);
    let t22 = t21 * t6;
    let t23 = t20 + t22;
    let _t24 = t18 + t23;
    let t25 = turn(car, 2);
    let t26 = t8 * t25;
    let t27 = turn(car, 6);
    let t28 = t7 * t27;
    let t29 = turn(car, 10);
    let t30 = t6 * t29;
    let t31 = t30 + t28;
    let t32 = t26 + t31;
    let t33 = t16.abs();
    w.extent[0].set(bh::to_int(t33) as u32);
    let t34 = t32.abs();
    w.extent[1].set(bh::to_int(t34) as u32);
}

/// `facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn facing_x87(car: &CarEffects, f: &MarksFloats, w: &RingWork) {
    let t0 = w.away[0].ld();
    let t1 = t0 * t0;
    let t2 = w.away[1].ld();
    let t3 = t2 * t2;
    let t4 = w.away[2].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = f.one.ld() / t8;
    let t10 = w.away[0].ld();
    let t11 = t10 * t9;
    w.away[0].st(t11);
    let t12 = w.away[1].ld();
    let t13 = t12 * t9;
    w.away[1].st(t13);
    let t14 = w.away[2].ld();
    let t15 = t9 * t14;
    w.away[2].st(t15);
    let t16 = w.away[0].ld();
    let t17 = t16 * car_float(&car.motion[0]);
    let t18 = w.away[1].ld();
    let t19 = t18 * car_float(&car.motion[1]);
    let t20 = w.away[2].ld();
    let t21 = t20 * car_float(&car.motion[2]);
    let t22 = t19 + t21;
    let t23 = t17 + t22;
    w.facing.st(t23);
}

/// `pull_length`'s arithmetic as plain Rust on the x87 model, in the same order.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_length_x87(
    en: &Entry,
    whole: &[impl crate::memory::session::Cell32; 16],
    l: &mut [u8; 10],
    w: &RingWork,
    rk: &RingKinds,
) {
    let t0 = bh::ld(en.place[0]);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].st(t1);
    let t2 = bh::ld(en.place[1]);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].st(t3);
    let t4 = bh::ld(en.place[2]);
    let t5 = t4 - ld_cell(&whole[14]);
    w.away[2].st(t5);
    let t6 = w.away[0].ld();
    let t7 = t6 * t6;
    let t8 = w.away[1].ld();
    let t9 = t8 * t8;
    let t10 = w.away[2].ld();
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 - bh::ld(rk.near_less);
    w.near.st(t15);
    *l = bh::f80(t14);
}

/// `pull_facing`'s arithmetic as plain Rust on the x87 model, in the same order.
fn pull_facing_x87(car: &CarEffects, l: &[u8; 10], f: &MarksFloats, w: &RingWork) {
    let t0 = bh::from_f80(*l);
    let t1 = f.one.ld() / t0;
    let t2 = w.away[0].ld();
    let t3 = t2 * t1;
    w.away[0].st(t3);
    let t4 = w.away[1].ld();
    let t5 = t4 * t1;
    w.away[1].st(t5);
    let t6 = w.away[2].ld();
    let t7 = t1 * t6;
    w.away[2].st(t7);
    let t8 = ldi_cell(&car.pace);
    let t9 = f.one.ld() / t8;
    let t10 = car_float(&car.motion[0]);
    let t11 = t10 * t9;
    w.turned[0].st(t11);
    let t12 = car_float(&car.motion[1]);
    let t13 = t12 * t9;
    w.turned[1].st(t13);
    let t14 = car_float(&car.motion[2]);
    let t15 = t9 * t14;
    w.turned[2].st(t15);
    let t16 = w.away[0].ld();
    let t17 = t16 * w.turned[0].ld();
    let t18 = w.away[1].ld();
    let t19 = t18 * w.turned[1].ld();
    let t20 = w.away[2].ld();
    let t21 = t20 * w.turned[2].ld();
    let t22 = t19 + t21;
    let t23 = t17 + t22;
    let t24 = -t23;
    w.facing.st(t24);
}

/// `pull_strength`'s arithmetic as plain Rust on the x87 model, in the same order.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_strength_x87(
    en: &mut Entry,
    car: &CarEffects,
    c: &MarksClock,
    f: &MarksFloats,
    w: &RingWork,
    rk: &RingKinds,
) {
    let t0 = ldi_cell(&car.pace);
    let t1 = t0 * bh::ld(rk.near_scale);
    let t2 = t1 + f.one.ld();
    let t3 = f.one.ld() / t2;
    let t4 = f.one.ld() - t3;
    w.whole.st(t4);
    let t5 = f.fixed_16_scale.ld();
    let t6 = t5 - bh::ld(en.size_factor);
    let t7 = bh::ld(rk.near_rate);
    let t8 = t7 * w.whole.ld();
    let t9 = t6 * t8;
    let t10 = t9 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    let t11 = t10 + bh::ld(en.size_factor);
    w.whole.set(bh::to_int(t11) as u32);
    en.size_factor = <f32 as bh::Stored>::stored(t11);
}

/// `box_place`'s arithmetic as plain Rust on the x87 model, in the same order.
fn box_place_x87(en: &Entry, w: &RingWork, car: &CarEffects) {
    let t0 = bh::ld(en.place[2]);
    let t1 = bh::ld(en.place[1]);
    let t2 = bh::ld(en.place[0]);
    let t3 = turn(car, 0);
    let t4 = t3 * t2;
    let t5 = turn(car, 4);
    let t6 = t5 * t1;
    let t7 = turn(car, 8);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = turn(car, 1);
    let t12 = t11 * t2;
    let t13 = turn(car, 5);
    let t14 = t13 * t1;
    let t15 = turn(car, 9);
    let t16 = t15 * t0;
    let t17 = t14 + t16;
    let _t18 = t12 + t17;
    let t19 = turn(car, 2);
    let t20 = t2 * t19;
    let t21 = turn(car, 6);
    let t22 = t1 * t21;
    let t23 = turn(car, 10);
    let t24 = t0 * t23;
    let t25 = t24 + t22;
    let t26 = t20 + t25;
    let t27 = t10.abs();
    w.extent[0].set(bh::to_int(t27) as u32);
    let t28 = t26.abs();
    w.extent[1].set(bh::to_int(t28) as u32);
}

/// The ring kinds' constants (`run_ring`, and the ribbons' two).
#[derive(Clone, Copy)]
pub struct RingKinds {
    /// Kind 5's drift factors
    pub drift5: [f32; 3],
    /// Kind 0xB's drift factors
    pub drift_b: [f32; 3],
    /// Kind 6's three dampings
    pub k6_damp: [f32; 3],
    /// Kind 6's size rate
    pub k6_pull: f32,
    /// Kind 0x15's three dampings
    pub k15_damp: [f32; 3],
    /// Kind 0x15's size rate
    pub k15_pull: f32,
    /// Kind 1's size factor (+0x64 times it)
    pub k1_size: f32,
    /// Kind 1's three dampings
    pub k1_damp: [f32; 3],
    /// Kind 1's size rate
    pub k1_pull: f32,
    /// Less this from the length to the object (`pull_length`)
    pub near_less: f32,
    /// The strength's pull scale (`pull_strength`)
    pub near_scale: f32,
    /// The strength's pull rate (`pull_strength`)
    pub near_rate: f32,
    /// Kind 0xD's y gain
    pub kd_gain: f32,
    /// Kind 0xD's three dampings
    pub kd_damp: [f32; 3],
    /// Kind 0xD's size rate
    pub kd_pull: f32,
    /// Kind 0x13's three dampings
    pub k13_damp: [f32; 3],
    /// Kind 0x13's y gain
    pub k13_gain: f32,
    /// Kind 0x16's three dampings
    pub k16_damp: [f32; 3],
    /// Kind 0x16's y gain
    pub k16_gain: f32,
    /// Kind 0x10's y gain
    pub k10_gain: f32,
    /// Kind 0x11's y gain
    pub k11_gain: f32,
    /// A ribbon's y gain (`run_ribbons::first_frame`)
    pub ribbon_a: f32,
    /// A ribbon's y gain per frame (`run_ribbons::move_`)
    pub ribbon_b: f32,
}

/// RingKinds's first values.
impl RingKinds {
    pub const fn new() -> Self {
        RingKinds {
            drift5: [-2.0, 92.0, f32::from_bits(0xFF800000)],
            drift_b: [-2.0, 92.0, f32::from_bits(0xFF800000)],
            k6_damp: [0.001, 0.001, 0.001],
            k6_pull: 0.01_f32,
            k15_damp: [0.001, 0.001, 0.001],
            k15_pull: 0.01_f32,
            k1_size: 1.0_f32,
            k1_damp: [0.003, 0.003, 0.003],
            k1_pull: 0.0015_f32,
            near_less: 111072.0_f32,
            near_scale: 0.0002_f32,
            near_rate: 0.005_f32,
            kd_gain: 0.01_f32,
            kd_damp: [0.004, 0.004, 0.004],
            kd_pull: 0.01_f32,
            k13_damp: [0.003, 0.003, 0.003],
            k13_gain: 16.0_f32,
            k16_damp: [0.003, 0.003, 0.003],
            k16_gain: 16.0_f32,
            k10_gain: -0.5_f32,
            k11_gain: -0.5_f32,
            ribbon_a: -0.5_f32,
            ribbon_b: -0.5_f32,
        }
    }
}

#[cfg(test)]
mod tests;
