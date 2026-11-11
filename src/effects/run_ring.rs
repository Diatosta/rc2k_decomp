//! `effects::run_ring` (0x4CB64D, in two pieces): the emitters' ring's frame. Each of the
//! ring's entries from its cursor along +4 stepped by its kind (+0xC; others nothing): the
//! life (+0x84) less the frame's ticks (dies when that borrows) and, but for the boxed kinds'
//! own test, death below its ground (+0x28). When the settings' entry flag (the settings + the
//! entry's +8 + 0x60) is set it is drawn: kinds 5, 0xD with their place on the object's
//! (`[[+0x68] + 0x24] + 0x30` .., the sums unrounded), 6, 0x15, 1, 0x13, 0x16 at their place,
//! by `effects::draw_cell_sprite` (0xB by its twin `draw_cell_sprite_2`; cell +0x58, angle
//! +0x50, alpha +0x74, size +0x60 as whole numbers or as they are); 0x10, 0x11 by
//! `effects::draw_subcell_sprite` (subcell +0x58 plus the life's frame of four). Then the
//! angle on (+0x54 times a constant, kept to 12 bits), the place moved by the motion, the
//! kind's gains, dampings (a part that changes sign stops at 0), the size pulled toward
//! +0x70 (kinds 6, 0x15 only while outside the object's box and not facing it; kind 1 also
//! its +0x64 pulled toward a constant when near and facing), the alpha faded by +0x78; kinds
//! 0x10, 0x11 die inside the object's box above a height. A dead entry is moved to just
//! before the cursor (or that moved on past it).//!
//! In:  nothing (the effects settings)
//!
//! Caller: `effects::run_frame` (after `run_ribbons`' set-up). The longer x87 runs
//! are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the entries and the game's variables. Refuses,
//! before anything, null settings, a null entry or link on the way, a null object where one
//! is read, and the draws' own refusals for an entry that may be drawn.

use crate::effects::cell_x87::CellX87;
use crate::effects::emitters::matrix_cell;
use crate::effects::mask_store::Masks;
#[cfg(test)]
use crate::effects::owner::effects;
use crate::effects::ring_entries::{entry, ld, set, st};
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::effects::setup::MarksClock;
use crate::effects::tuning::MarksFloats;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::effects_settings_store::{CarEffects, EffectsSettings, car_or_none};
use crate::scene::matrix3::extended;

/// The settings' flag for a ring entry's tag (its +8, a car's number times 0x4C8 for the
/// cars' emitters): the word at the tag + 0x60 in the settings.
fn entry_flag(marks: &EffectsSettings, tag: u32) -> Result<u32, BadInput> {
    marks.at(tag.wrapping_add(0x60)).read("a mark's field")
}

fn k(cell: &f32) -> u32 {
    cell.to_bits()
}

/// The original's 0x4CE9AF .. 0x4CEA40 (and 0x4CEC77 ..): the place less the object's (`[p + 0x30]` ..) into `RingWork.away[0]` .., turned by the object's matrix (`m`), |x| and |z| `fistp`ed into `RingWork.extent[0]` / `RingWork.extent[1]`.
/// The car a ring entry is on (its +0x68, a `effects_settings_store::car_handle`).
fn entry_car(ring: &Ring, e: u32) -> Result<&'static CarEffects, Refused> {
    car_or_none(entry(ring, e)?.car).ok_or_else(|| BadInput("a ring entry without its car").into())
}

fn box_(ring: &Ring, e: u32, whole: &[crate::session::Float; 16], car: &CarEffects) {
    box_x87(ring, &ring.work, e, whole, car);
}

/// The car's whole matrix (its block's), refused where the original would read address 0.
fn whole_of(car: &CarEffects) -> Result<&'static [crate::session::Float; 16], BadInput> {
    crate::race::blocks_store::car_of(car.whole_block.get())
        .map(|b| &b.whole_matrix)
        .ok_or(BadInput("a ring entry without its object"))
}

/// The original's 0x4CEA5E .. 0x4CEAD4 (and 0x4CED26 ..): that way made one long (`MarksFloats.one` over its length), dotted with the object's +0x4C .. into `RingWork.facing`.
fn facing(ring: &Ring, car: &CarEffects, effects: &crate::effects::owner::Effects) {
    facing_x87(car, &effects.setup.floats, &ring.work);
}

/// The original's 0x4CEF8A .. 0x4CEFDA: the place less the object's into `RingWork.away[0]` .., its length (kept, 80 bits, at `l`) less `flt_93A526` into `RingWork.near`.
fn pull_length(ring: &Ring, e: u32, whole: &[crate::session::Float; 16], length: &mut [u8; 10]) {
    pull_length_x87(ring, e, whole, length, &ring.work, &ring.kinds);
}

/// The original's 0x4CEFEC .. 0x4CF07F: the way made one long by the kept length, the object's +0x4C .. over its +0x88, minus their dot into `RingWork.facing`.
fn pull_facing(
    ring: &Ring,
    car: &CarEffects,
    length: &[u8; 10],
    effects: &crate::effects::owner::Effects,
) {
    pull_facing_x87(car, length, &effects.setup.floats, &ring.work);
}

/// The original's 0x4CF08B .. 0x4CF0D8: the entry's +0x64 pulled toward `MarksFloats.fixed_16_scale` (`fist` into `RingWork.whole`).
fn pull_strength(ring: &Ring, e: u32, car: &CarEffects, effects: &crate::effects::owner::Effects) {
    let setup = &effects.setup;
    pull_strength_x87(
        ring,
        e,
        car,
        &setup.clock,
        &setup.floats,
        &ring.work,
        &ring.kinds,
    );
}

/// The original's 0x4CF65C .. 0x4CF6B7 (and 0x4CF7C4 ..): the place turned by the object's matrix (`m`), |x| and |z| into `RingWork.extent[0]` / `RingWork.extent[1]`.
fn box_place(ring: &Ring, e: u32, car: &CarEffects) {
    box_place_x87(ring, &ring.work, e, car);
}

/// The kinds' constants: (the y gain, the three dampings, the size's rate).
struct Kind {
    gain: Option<&'static f32>,
    damp: [&'static f32; 3],
    pull: Option<&'static f32>,
}

impl Kind {
    /// The size's rate (the kinds that pull the size have one; the original would read
    /// address 0 for one that has none: refused).
    fn pull(&self) -> Result<&'static f32, BadInput> {
        self.pull
            .ok_or(BadInput("a ring kind that pulls the size without a rate"))
    }

    /// The y gain (the kinds that gain have one; as `pull`).
    fn gain(&self) -> Result<&'static f32, BadInput> {
        self.gain
            .ok_or(BadInput("a ring kind that gains without a gain"))
    }
}

static K6: Kind = Kind {
    gain: None,
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k6_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k6_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k6_damp[2],
    ],
    pull: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k6_pull,
    ),
};
static K15: Kind = Kind {
    gain: None,
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k15_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k15_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k15_damp[2],
    ],
    pull: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k15_pull,
    ),
};
static K1: Kind = Kind {
    gain: None,
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k1_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k1_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k1_damp[2],
    ],
    pull: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k1_pull,
    ),
};
static KD: Kind = Kind {
    gain: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .kd_gain,
    ),
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .kd_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .kd_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .kd_damp[2],
    ],
    pull: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .kd_pull,
    ),
};
static K13: Kind = Kind {
    gain: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k13_gain,
    ),
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k13_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k13_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k13_damp[2],
    ],
    pull: None,
};
static K16: Kind = Kind {
    gain: Some(
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k16_gain,
    ),
    damp: [
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k16_damp[0],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k16_damp[1],
        &crate::effects::owner::EFFECTS
            .unchecked()
            .ring
            .kinds
            .k16_damp[2],
    ],
    pull: None,
};

/// Why it refuses, if it does: the walk's pointers checked, nothing written.
fn refusal(
    effects: &crate::effects::owner::Effects,
    ring: &Ring,
    fog: &crate::scene::fog::Fog,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        let Ok(marks) = effects.settings() else {
            return Ok(Some("no marks settings"));
        };
        let mut e: u32 = ring.work.cursor.get();
        let count = ring.work.count.get() as i32;
        if count > 0 && (e == 0 || entry(ring, e)?.prev == 0) {
            return Ok(Some("a null ring entry"));
        }
        for _ in 0..count.max(0) {
            if e == 0 || entry(ring, e)?.prev == 0 || entry(ring, e)?.next == 0 {
                return Ok(Some("a null ring entry or link"));
            }
            let kind = entry(ring, e)?.kind;
            if [5, 0xB, 6, 0x15, 1, 0xD, 0x13, 0x16, 0x10, 0x11].contains(&kind) {
                let obj = car_or_none(entry(ring, e)?.car);
                let needs = matches!(kind, 6 | 0x15 | 1 | 0x10 | 0x11)
                    || entry_flag(marks, entry(ring, e)?.emitter_tag)? != 0;
                if needs && !obj.is_some_and(CarEffects::has_whole) {
                    return Ok(Some("a ring entry without its object"));
                }
                // The draws' own refusals, for an entry that may be drawn.
                if entry_flag(marks, entry(ring, e)?.emitter_tag)? != 0 {
                    if crate::scene::fog_at::fog_at(fog, 0).is_err() {
                        return Ok(Some("the fog table is not there"));
                    }
                    let bad = if matches!(kind, 0x10 | 0x11) {
                        entry(ring, e)?.cell.wrapping_add(3) >= 0x410
                    } else {
                        entry(ring, e)?.cell > 0x40 || entry(ring, e)?.angle >= 0x1000
                    };
                    if bad {
                        return Ok(Some("a ring entry's cell or angle past its table"));
                    }
                }
            }
            e = entry(ring, e)?.next;
        }
        None
    })
}

/// The life and the ground: whether the entry dies.
fn aged(
    ring: &Ring,
    e: u32,
    ground: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<bool, Refused> {
    Ok({
        let t: u32 = effects.setup.clock.frame_ticks.get();
        let life = entry(ring, e)?.life;
        set(ring, e, life.wrapping_sub(t), |en| &mut en.life)?;
        if life < t {
            return Ok(true);
        }
        ground && x87::compare_status(entry(ring, e)?.place[1], entry(ring, e)?.ground_y) == 0x100
    })
}

/// The place on the object's (unrounded sums), st(0) first.
fn on_object(ring: &Ring, e: u32) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        let m = entry_car(ring, e)?.whole()?;
        [
            x87::add_80(entry(ring, e)?.place[0], matrix_cell(m, 0x30)?),
            x87::add_80(entry(ring, e)?.place[1], matrix_cell(m, 0x34)?),
            x87::add_80(entry(ring, e)?.place[2], matrix_cell(m, 0x38)?),
        ]
    })
}

fn at_place(ring: &Ring, e: u32) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        [
            extended(entry(ring, e)?.place[0]),
            extended(entry(ring, e)?.place[1]),
            extended(entry(ring, e)?.place[2]),
        ]
    })
}

/// The settings' flag for the entry.
fn shown(effects: &crate::effects::owner::Effects, ring: &Ring, e: u32) -> Result<bool, Refused> {
    Ok(entry_flag(effects.settings()?, entry(ring, e)?.emitter_tag)? != 0)
}

/// The alpha (top byte) as a whole number (`fistp` through `RingWork.whole`).
fn whole_alpha(ring: &Ring, e: u32) -> Result<u32, Refused> {
    Ok({
        let a = x87::to_int(entry(ring, e)?.alpha);
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
    ring: &Ring,
    e: u32,
    spin: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let t: i32 = setup.clock.frame_ticks.get() as i32;
        if spin {
            let turn =
                x87::int_times_to_int(entry(ring, e)?.angle_rate as i32, &setup.clock.ticks_scaled)
                    as u32;
            set(
                ring,
                e,
                entry(ring, e)?.angle.wrapping_add(turn) & 0xFFF,
                |en| &mut en.angle,
            )?;
        }
        for axis in 0..3 {
            // The place plus the motion times the ticks.
            set(
                ring,
                e,
                x87::plus_times_int(entry(ring, e)?.place[axis], entry(ring, e)?.motion[axis], t)
                    .to_bits(),
                |en| &mut en.place[axis],
            )?;
        }
    }
    Ok(())
}

fn gain_y(
    ring: &Ring,
    e: u32,
    k_addr: &f32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        set(
            ring,
            e,
            x87::plus_times_int(entry(ring, e)?.motion[1], k(k_addr), t).to_bits(),
            |en| &mut en.motion[1],
        )?;
    }
    Ok(())
}

/// The three dampings (a part that changes sign stops at 0).
fn damp3(
    ring: &Ring,
    e: u32,
    ks: [&f32; 3],
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        for (axis, kk) in ks.into_iter().enumerate() {
            let old = entry(ring, e)?.motion[axis];
            ring.work.whole.set(old);
            let new = bh::st(bh::ld(old) - bh::ld(old) * bh::ld(k(kk)) * bh::int(t));
            set(ring, e, new, |en| &mut en.motion[axis])?;
            if (old ^ new) & 0x8000_0000 != 0 {
                set(ring, e, 0, |en| &mut en.motion[axis])?;
            }
        }
    }
    Ok(())
}

/// The size pulled toward +0x70.
fn pull_size(
    ring: &Ring,
    e: u32,
    k_addr: &f32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        let (w, f) = x87::approach(
            entry(ring, e)?.size_goal as i32,
            entry(ring, e)?.size,
            k(k_addr),
            t,
        );
        ring.work.whole.set(w as u32);
        set(ring, e, f.to_bits(), |en| &mut en.size)?;
        if w > entry(ring, e)?.size_goal as i32 {
            set(
                ring,
                e,
                x87::from_int(entry(ring, e)?.size_goal as i32).to_bits(),
                |en| &mut en.size,
            )?;
        }
    }
    Ok(())
}

fn fade(ring: &Ring, e: u32, effects: &crate::effects::owner::Effects) -> Result<(), Refused> {
    {
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        set(
            ring,
            e,
            bh::st(bh::ld(entry(ring, e)?.alpha) - bh::ld(entry(ring, e)?.alpha_fade) * bh::int(t)),
            |en| &mut en.alpha,
        )?;
    }
    Ok(())
}

/// The box test of kinds 6 and 0x15: whether the size is pulled.
fn outside_and_away(
    ring: &Ring,
    e: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<bool, Refused> {
    Ok({
        let car = entry_car(ring, e)?;
        box_(ring, e, whole_of(car)?, car);
        let outside = (entry(ring, e)?.box_limit[0] as i32) < (ring.work.extent[0].get() as i32)
            || (entry(ring, e)?.box_limit[1] as i32) < (ring.work.extent[1].get() as i32);
        if !outside {
            return Ok(false);
        }
        facing(ring, car, effects);
        (ring.work.facing.bits() as i32) <= 0
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
    masks: &Masks,
    ring: &Ring,
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
                masks, ring, effects, cell, size, e, p, view,
            ),
            _ => draw_cell_sprite::draw_cell_sprite(
                masks, ring, effects, cell, angle, alpha, size, e, p, view,
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
    masks: &Masks,
    ring: &Ring,
    e: u32,
    place: u32,
    ks: [&f32; 3],
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    {
        if aged(ring, e, true, effects)? {
            return Ok(true);
        }
        if shown(effects, ring, e)? {
            let p = on_object(ring, e)?;
            let [cell, angle, alpha, size] = [
                entry(ring, e)?.cell,
                entry(ring, e)?.angle,
                entry(ring, e)?.alpha,
                entry(ring, e)?.size,
            ];
            draw(
                masks,
                ring,
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
        spin_and_move(ring, e, true, effects)?;
        gain_y(ring, e, ks[0], effects)?;
        let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
        set(
            ring,
            e,
            bh::to_int(bh::ld(k(ks[1])) * bh::int(t) + bh::int(entry(ring, e)?.size as i32)) as u32,
            |en| &mut en.size,
        )?;
        set(
            ring,
            e,
            x87::int_times_int_plus_int(k(ks[2]) as i32, t, entry(ring, e)?.alpha as i32) as u32,
            |en| &mut en.alpha,
        )?;
    }
    Ok(false)
}

/// Kinds 6, 0x15, 1, 0xD, 0x13, 0x16 (draw 1): whether it dies.
#[allow(
    clippy::too_many_arguments,
    reason = "the effects' groups, the owners and the view's fog"
)]
fn fading_kind(
    masks: &Masks,
    ring: &Ring,
    e: u32,
    kind: u32,
    c: &Kind,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    {
        if aged(ring, e, true, effects)? {
            return Ok(true);
        }
        if (entry(ring, e)?.alpha as i32) < 0 {
            set(ring, e, 0, |en| &mut en.alpha)?;
        }
        if shown(effects, ring, e)? {
            let p = if kind == 0xD {
                on_object(ring, e)?
            } else {
                at_place(ring, e)?
            };
            let [cell, angle] = [entry(ring, e)?.cell, entry(ring, e)?.angle];
            let alpha = whole_alpha(ring, e)?;
            let size = if kind == 1 {
                bh::to_int(
                    bh::ld(entry(ring, e)?.size)
                        + bh::ld(entry(ring, e)?.size_factor) * bh::ld(k(&ring.kinds.k1_size)),
                )
            } else {
                x87::to_int(entry(ring, e)?.size)
            };
            let size = whole(ring, size);
            draw(
                masks,
                ring,
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
        spin_and_move(ring, e, true, effects)?;
        if c.gain.is_some() && kind == 0xD {
            gain_y(ring, e, c.gain()?, effects)?;
        }
        damp3(ring, e, c.damp, effects)?;
        match kind {
            6 | 0x15 => {
                if outside_and_away(ring, e, effects)? {
                    pull_size(ring, e, c.pull()?, effects)?;
                }
                fade(ring, e, effects)?;
            }
            1 => {
                pull_size(ring, e, c.pull()?, effects)?;
                fade(ring, e, effects)?;
                pull_toward(ring, e, effects)?;
            }
            0xD => {
                pull_size(ring, e, c.pull()?, effects)?;
                fade(ring, e, effects)?;
            }
            _ => {
                fade(ring, e, effects)?;
                let t: i32 = effects.setup.clock.frame_ticks.get() as i32;
                set(
                    ring,
                    e,
                    x87::plus_times_int(entry(ring, e)?.size, k(c.gain()?), t).to_bits(),
                    |en| &mut en.size,
                )?;
            }
        }
    }
    Ok(false)
}

/// Kind 1's pull of its +0x64 when near and facing.
fn pull_toward(
    ring: &Ring,
    e: u32,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let car = entry_car(ring, e)?;
        let mut length = [0u8; 10];
        pull_length(ring, e, whole_of(car)?, &mut length);
        if (ring.work.near.bits() as i32) < 0 {
            return Ok(());
        }
        pull_facing(ring, car, &length, effects);
        if (ring.work.facing.bits() as i32) <= 0 {
            return Ok(());
        }
        pull_strength(ring, e, car, effects);
        if (ring.work.whole.get() as i32) > (setup.floats.pull_limit.bits() as i32) {
            set(
                ring,
                e,
                x87::loaded(&setup.floats.fixed_16_scale).to_bits(),
                |en| &mut en.size_factor,
            )?;
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
    masks: &Masks,
    ring: &Ring,
    e: u32,
    gain: &f32,
    both: bool,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<bool, Refused> {
    if aged(ring, e, true, effects)? {
        return Ok(true);
    }
    spin_and_move(ring, e, false, effects)?;
    gain_y(ring, e, gain, effects)?;
    if shown(effects, ring, e)? {
        let p = on_object(ring, e)?;
        let size = entry(ring, e)?.size;
        let f = bh::to_int(
            bh::int(entry(ring, e)?.life as i32)
                * bh::ld(effects.setup.floats.twentieth.bits())
                * bh::ld(entry(ring, e)?.cell_rate),
        );
        ring.work.whole.set(f as u32);
        let subcell = ((f as u32) & 3).wrapping_add(entry(ring, e)?.cell);
        draw(
            masks,
            ring,
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
    box_place(ring, e, entry_car(ring, e)?);
    let mut outside = (entry(ring, e)?.box_limit[0] as i32) < (ring.work.extent[0].get() as i32);
    if !outside && both {
        outside = (entry(ring, e)?.box_limit[1] as i32) < (ring.work.extent[1].get() as i32);
    }
    if !outside && x87::compare_status(entry(ring, e)?.place[1], &ring.work.top) == 0 {
        return Ok(true);
    }
    Ok(x87::compare_status(entry(ring, e)?.place[1], entry(ring, e)?.ground_y) == 0x100)
}

pub fn run_ring(
    masks: &Masks,
    ring: &Ring,
    effects: &crate::effects::owner::Effects,
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if let Some(why) = refusal(effects, ring, view.fog)? {
        return Err(BadInput(why).into());
    }
    {
        let mut e: u32 = ring.work.cursor.get();
        let mut counter = ring.work.count.get() as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = entry(ring, e)?.next;
            let dies = match entry(ring, e)?.kind {
                5 => drift_kind(
                    masks,
                    ring,
                    e,
                    1,
                    [
                        &ring.kinds.drift5[0],
                        &ring.kinds.drift5[1],
                        &ring.kinds.drift5[2],
                    ],
                    effects,
                    view,
                )?,
                0xB => drift_kind(
                    masks,
                    ring,
                    e,
                    2,
                    [
                        &ring.kinds.drift_b[0],
                        &ring.kinds.drift_b[1],
                        &ring.kinds.drift_b[2],
                    ],
                    effects,
                    view,
                )?,
                6 => fading_kind(masks, ring, e, 6, &K6, effects, view)?,
                0x15 => fading_kind(masks, ring, e, 0x15, &K15, effects, view)?,
                1 => fading_kind(masks, ring, e, 1, &K1, effects, view)?,
                0xD => fading_kind(masks, ring, e, 0xD, &KD, effects, view)?,
                0x13 => fading_kind(masks, ring, e, 0x13, &K13, effects, view)?,
                0x16 => fading_kind(masks, ring, e, 0x16, &K16, effects, view)?,
                0x10 => boxed_kind(masks, ring, e, &ring.kinds.k10_gain, true, effects, view)?,
                0x11 => boxed_kind(masks, ring, e, &ring.kinds.k11_gain, false, effects, view)?,
                _ => false,
            };
            if dies {
                ring.work.count.set(ring.work.count.get().wrapping_sub(1));
                if e == ring.work.cursor.get() {
                    ring.work.cursor.set(entry(ring, e)?.next);
                } else {
                    let (before, after) = (entry(ring, e)?.prev, entry(ring, e)?.next);
                    set(ring, before, after, |en| &mut en.next)?;
                    set(ring, after, before, |en| &mut en.prev)?;
                    let cursor: u32 = ring.work.cursor.get();
                    let last = entry(ring, cursor)?.prev;
                    set(ring, cursor, e, |en| &mut en.prev)?;
                    set(ring, last, e, |en| &mut en.next)?;
                    set(ring, e, last, |en| &mut en.prev)?;
                    set(ring, e, cursor, |en| &mut en.next)?;
                }
            }
            e = next;
        }
    }
    Ok(())
}

use crate::effects::plain::{ld_cell, ldi_cell};
use crate::math::x87_float as bh;

/// `fld dword ptr` of a float of a car record in the settings.
fn car_float(cell: &impl crate::session::Cell32) -> bh::X {
    cell.ld()
}

/// The car's 3 x 3 turning matrix (the record's `matrix`, 4 floats a row): cell `n`.
fn turn(car: &CarEffects, n: usize) -> bh::X {
    car_float(&car.matrix[n])
}

/// `box_`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places.
fn box_x87(
    ring: &Ring,
    w: &RingWork,
    e: u32,
    whole: &[crate::session::Float; 16],
    car: &CarEffects,
) {
    let t0 = ld(ring, e, |en| en.place[0]);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].st(t1);
    let t2 = ld(ring, e, |en| en.place[1]);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].st(t3);
    let t4 = ld(ring, e, |en| en.place[2]);
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
    w.extent[0].set_bits(bh::to_int(t33) as u32);
    let t34 = t32.abs();
    w.extent[1].set_bits(bh::to_int(t34) as u32);
}

/// `facing`'s x87 block as plain Rust on the x87 model (28 instructions), same order and memory places.
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

/// `pull_length`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_length_x87(
    ring: &Ring,
    e: u32,
    whole: &[crate::session::Float; 16],
    l: &mut [u8; 10],
    w: &RingWork,
    rk: &RingKinds,
) {
    let t0 = ld(ring, e, |en| en.place[0]);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].st(t1);
    let t2 = ld(ring, e, |en| en.place[1]);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].st(t3);
    let t4 = ld(ring, e, |en| en.place[2]);
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
    let t15 = t14 - bh::ld(rk.near_less.to_bits());
    w.near.st(t15);
    *l = bh::f80(t14);
}

/// `pull_facing`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
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

/// `pull_strength`'s x87 block as plain Rust on the x87 model (15 instructions), same order and memory places.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_strength_x87(
    ring: &Ring,
    e: u32,
    car: &CarEffects,
    c: &MarksClock,
    f: &MarksFloats,
    w: &RingWork,
    rk: &RingKinds,
) {
    let t0 = ldi_cell(&car.pace);
    let t1 = t0 * bh::ld(rk.near_scale.to_bits());
    let t2 = t1 + f.one.ld();
    let t3 = f.one.ld() / t2;
    let t4 = f.one.ld() - t3;
    w.whole.st(t4);
    let t5 = f.fixed_16_scale.ld();
    let t6 = t5 - ld(ring, e, |en| en.size_factor);
    let t7 = bh::ld(rk.near_rate.to_bits());
    let t8 = t7 * w.whole.ld();
    let t9 = t6 * t8;
    let t10 = t9 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    let t11 = t10 + ld(ring, e, |en| en.size_factor);
    w.whole.set_bits(bh::to_int(t11) as u32);
    st(ring, e, t11, |en| &mut en.size_factor);
}

/// `box_place`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
fn box_place_x87(ring: &Ring, w: &RingWork, e: u32, car: &CarEffects) {
    let t0 = ld(ring, e, |en| en.place[2]);
    let t1 = ld(ring, e, |en| en.place[1]);
    let t2 = ld(ring, e, |en| en.place[0]);
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
    w.extent[0].set_bits(bh::to_int(t27) as u32);
    let t28 = t26.abs();
    w.extent[1].set_bits(bh::to_int(t28) as u32);
}

/// The ring kinds' constants (`run_ring`, and the ribbons' two) (IDA 0x93A4DA .. 0x93A576).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RingKinds {
    /// 0x93A4DA: `flt_93A4DA`, `flt_93A4DE`, `dword_93A4E2`: kind 5's drift factors
    pub drift5: [f32; 3],
    /// 0x93A4E6: `flt_93A4E6`, `flt_93A4EA`, `dword_93A4EE`: kind 0xB's drift factors
    pub drift_b: [f32; 3],
    /// 0x93A4F2: `flt_93A4F2` .. `flt_93A4FA`: kind 6's three dampings
    pub k6_damp: [f32; 3],
    /// 0x93A4FE: `flt_93A4FE`: kind 6's size rate
    pub k6_pull: f32,
    /// 0x93A502: `flt_93A502` .. `flt_93A50A`: kind 0x15's three dampings
    pub k15_damp: [f32; 3],
    /// 0x93A50E: `flt_93A50E`: kind 0x15's size rate
    pub k15_pull: f32,
    /// 0x93A512: `flt_93A512`: kind 1's size factor (+0x64 times it)
    pub k1_size: f32,
    /// 0x93A516: `flt_93A516` .. `flt_93A51E`: kind 1's three dampings
    pub k1_damp: [f32; 3],
    /// 0x93A522: `flt_93A522`: kind 1's size rate
    pub k1_pull: f32,
    /// 0x93A526: `flt_93A526`: less this from the length to the object (`pull_length`)
    pub near_less: f32,
    /// 0x93A52A: `flt_93A52A`: the strength's pull scale (`pull_strength`)
    pub near_scale: f32,
    /// 0x93A52E: `flt_93A52E`: the strength's pull rate (`pull_strength`)
    pub near_rate: f32,
    /// 0x93A532: `flt_93A532`: kind 0xD's y gain
    pub kd_gain: f32,
    /// 0x93A536: `flt_93A536` .. `flt_93A53E`: kind 0xD's three dampings
    pub kd_damp: [f32; 3],
    /// 0x93A542: `flt_93A542`: kind 0xD's size rate
    pub kd_pull: f32,
    /// 0x93A546: `flt_93A546` .. `flt_93A54E`: kind 0x13's three dampings
    pub k13_damp: [f32; 3],
    /// 0x93A552: `flt_93A552`: kind 0x13's y gain
    pub k13_gain: f32,
    /// 0x93A556: `flt_93A556` .. `flt_93A55E`: kind 0x16's three dampings
    pub k16_damp: [f32; 3],
    /// 0x93A562: `flt_93A562`: kind 0x16's y gain
    pub k16_gain: f32,
    /// 0x93A566: `flt_93A566`: kind 0x10's y gain
    pub k10_gain: f32,
    /// 0x93A56A: `flt_93A56A`: kind 0x11's y gain
    pub k11_gain: f32,
    /// 0x93A56E: `flt_93A56E`: a ribbon's y gain (`run_ribbons::first_frame`)
    pub ribbon_a: f32,
    /// 0x93A572: `flt_93A572`: a ribbon's y gain per frame (`run_ribbons::move_`)
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
impl RingKinds {
    pub const START: usize = 0x93A4DA;
    pub const LEN: usize = 0x9C;
}

#[cfg(test)]
mod ring_kinds_tests {
    use super::*;
    use core::mem::offset_of;

    fn bytes<T>(v: &T) -> Vec<u8> {
        let p = std::ptr::from_ref::<T>(v).cast::<u8>();
        (0..size_of::<T>())
            .map(|i| unsafe { p.add(i).read() })
            .collect()
    }

    #[test]
    fn ring_kinds_starts_as_the_games() {
        assert_eq!(size_of::<RingKinds>(), RingKinds::LEN);
        assert!(
            bytes(&effects().ring.kinds)
                == crate::moved::image_bytes(RingKinds::START, RingKinds::LEN),
            "RING_KINDS starts differently from the image"
        );
        assert_eq!(offset_of!(RingKinds, drift5), 0x93A4DA - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, drift_b), 0x93A4E6 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k6_damp), 0x93A4F2 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k6_pull), 0x93A4FE - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k15_damp), 0x93A502 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k15_pull), 0x93A50E - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k1_size), 0x93A512 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k1_damp), 0x93A516 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k1_pull), 0x93A522 - RingKinds::START);
        assert_eq!(
            offset_of!(RingKinds, near_less),
            0x93A526 - RingKinds::START
        );
        assert_eq!(
            offset_of!(RingKinds, near_scale),
            0x93A52A - RingKinds::START
        );
        assert_eq!(
            offset_of!(RingKinds, near_rate),
            0x93A52E - RingKinds::START
        );
        assert_eq!(offset_of!(RingKinds, kd_gain), 0x93A532 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, kd_damp), 0x93A536 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, kd_pull), 0x93A542 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k13_damp), 0x93A546 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k13_gain), 0x93A552 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k16_damp), 0x93A556 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k16_gain), 0x93A562 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k10_gain), 0x93A566 - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, k11_gain), 0x93A56A - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, ribbon_a), 0x93A56E - RingKinds::START);
        assert_eq!(offset_of!(RingKinds, ribbon_b), 0x93A572 - RingKinds::START);
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn box_x87(mk: usize, e: usize, p: usize, m: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{m} + 0x0]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{m} + 0x10]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{m} + 0x20]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x4]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{m} + 0x14]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{m} + 0x24]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{m} + 0x18]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{m} + 0x28]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0740]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0744]",
            mk = in(reg) mk,
            e = in(reg) e,
            p = in(reg) p,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn facing_x87(mk: usize, o: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            "fmul dword ptr [{o} + 0x4C]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fmul dword ptr [{o} + 0x50]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fmul dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0x1D0774]",
            mk = in(reg) mk,
            o = in(reg) o,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn pull_length_x87(mk: usize, e: usize, p: usize, l: usize, rk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x10]",
            "fsub dword ptr [{p} + 0x30]",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{e} + 0x14]",
            "fsub dword ptr [{p} + 0x34]",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{e} + 0x18]",
            "fsub dword ptr [{p} + 0x38]",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            ".byte 0xD9, 0xC0",
            "fsub dword ptr [{rk} + 0x4C]",
            "fstp dword ptr [{mk} + 0x1D077C]",
            "fstp tbyte ptr [{l}]",
            mk = in(reg) mk,
            e = in(reg) e,
            p = in(reg) p,
            l = in(reg) l,
            rk = in(reg) rk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn pull_facing_x87(mk: usize, o: usize, l: usize) {
        unsafe {
            core::arch::asm!(
                    "fld tbyte ptr [{l}]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07D8]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07DC]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E0]",
            "fild dword ptr [{o} + 0x88]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fld dword ptr [{o} + 0x4C]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E4]",
            "fld dword ptr [{o} + 0x50]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07E8]",
            "fld dword ptr [{o} + 0x54]",
            ".byte 0xDE, 0xC9",
            "fstp dword ptr [{mk} + 0x1D07EC]",
            "fld dword ptr [{mk} + 0x1D07D8]",
            "fmul dword ptr [{mk} + 0x1D07E4]",
            "fld dword ptr [{mk} + 0x1D07DC]",
            "fmul dword ptr [{mk} + 0x1D07E8]",
            "fld dword ptr [{mk} + 0x1D07E0]",
            "fmul dword ptr [{mk} + 0x1D07EC]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xE0",
            "fstp dword ptr [{mk} + 0x1D0774]",
            mk = in(reg) mk,
            o = in(reg) o,
            l = in(reg) l,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn pull_strength_x87(mk: usize, e: usize, o: usize, rk: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{o} + 0x88]",
            "fmul dword ptr [{rk} + 0x50]",
            "fadd dword ptr [{mk} + 0x22C]",
            "fdivr dword ptr [{mk} + 0x22C]",
            "fsubr dword ptr [{mk} + 0x22C]",
            "fstp dword ptr [{mk} + 0x1D0780]",
            "fld dword ptr [{mk} + 0x29C]",
            "fsub dword ptr [{e} + 0x64]",
            "fld dword ptr [{rk} + 0x54]",
            "fmul dword ptr [{mk} + 0x1D0780]",
            ".byte 0xDE, 0xC9",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x64]",
            "fist dword ptr [{mk} + 0x1D0780]",
            "fstp dword ptr [{e} + 0x64]",
            mk = in(reg) mk,
            e = in(reg) e,
            o = in(reg) o,
            rk = in(reg) rk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn box_place_x87(mk: usize, e: usize, m: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{m} + 0x0]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{m} + 0x10]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{m} + 0x20]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x4]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{m} + 0x14]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{m} + 0x24]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{m} + 0x8]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{m} + 0x18]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{m} + 0x28]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xCA",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0740]",
            ".byte 0xDD, 0xC0",
            ".byte 0xD9, 0xF7",
            ".byte 0xD9, 0xE1",
            "fistp dword ptr [{mk} + 0x1D0744]",
            mk = in(reg) mk,
            e = in(reg) e,
            m = in(reg) m,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, differential, with_copy};
    use crate::effects::owner::effects;
    use crate::effects::ring_store::RingWork;
    use crate::effects::tuning::MarksFloats;
    use crate::race::effects_settings_store::CarEffects;

    fn word(at: usize) -> u32 {
        unsafe { crate::pointers::host_at(at).cast::<u32>().read_unaligned() }
    }

    /// A car record whose words at the record `offsets` hold the test buffer's dwords at `at + offset - base`.
    fn car_with(at: usize, base: usize, offsets: &[usize]) -> CarEffects {
        let car = CarEffects::zeroed();
        for &o in offsets {
            crate::pointers::GamePtr::to(&car)
                .offset(o as u32)
                .write(word(at + o - base), "a test car record")
                .unwrap();
        }
        car
    }

    /// The car's 3 x 3 matrix (the record's +0x94) from the buffer at `at`.
    fn car_matrix(at: usize) -> CarEffects {
        let offsets: Vec<usize> = [0x0, 0x4, 0x8, 0x10, 0x14, 0x18, 0x20, 0x24, 0x28]
            .iter()
            .map(|o| 0x94 + o)
            .collect();
        car_with(at, 0x94, &offsets)
    }

    /// A whole matrix whose translation (+0x30 ..) is the buffer's at `at` + 0x30 (`at` the
    /// matrix's place, as the original reads it).
    fn whole_at(at: usize) -> [crate::session::Float; 16] {
        std::array::from_fn(|i| {
            crate::session::Float::from_bits(if (12..15).contains(&i) {
                word(at + 4 * i)
            } else {
                0
            })
        })
    }

    #[test]
    fn box_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D06E0,
                    hi: 0x1D0826,
                    reads: &[(0x1D07D8, F32), (0x1D07DC, F32), (0x1D07E0, F32)],
                    writes: &[
                        (0x1D0740, I32),
                        (0x1D0744, I32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x30,
                    hi: 0x3C,
                    reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2C,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::box_x87(b[0], b[1], b[2], b[3]),
            |b| {
                with_copy(b[0], RingWork::START, |w| {
                    crate::effects::ring_entries::with_entry_at(ring, b[1], 0x10, 0x1C, |e| {
                        super::box_x87(ring, w, e, &whole_at(b[2]), &car_matrix(b[3]));
                    });
                });
            },
        );
    }

    #[test]
    fn facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x200,
                    hi: 0x1D0826,
                    reads: &[
                        (0x22C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                    writes: &[
                        (0x1D0774, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x4C,
                    hi: 0x58,
                    reads: &[(0x4C, F32), (0x50, F32), (0x54, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::facing_x87(b[0], b[1]),
            |b| {
                with_copy(b[0], MarksFloats::START, |f| {
                    with_copy(b[0], RingWork::START, |w| {
                        let car = car_with(b[1], 0, &[0x4C, 0x50, 0x54]);
                        super::facing_x87(&car, f, w);
                    });
                });
            },
        );
    }

    #[test]
    fn pull_length_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D06E0,
                    hi: 0x1D07FC,
                    reads: &[(0x1D07D8, F32), (0x1D07DC, F32), (0x1D07E0, F32)],
                    writes: &[
                        (0x1D077C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                    ],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x30,
                    hi: 0x3C,
                    reads: &[(0x30, F32), (0x34, F32), (0x38, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[],
                    writes: &[(0x0, F80)],
                },
                Op::Base {
                    lo: 0x4C,
                    hi: 0x50,
                    reads: &[(0x4C, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::pull_length_x87(b[0], b[1], b[2], b[3], b[4]),
            |b| {
                with_copy(b[0], RingWork::START, |w| {
                    crate::effects::ring_entries::with_entry_at(ring, b[1], 0x10, 0x1C, |e| {
                        let mut length = [0xAA; 10];
                        let kinds = super::RingKinds {
                            near_less: f32::from_bits(word(b[4] + 0x4C)),
                            ..crate::effects::owner::effects().ring.kinds
                        };
                        super::pull_length_x87(ring, e, &whole_at(b[2]), &mut length, w, &kinds);
                        unsafe {
                            crate::pointers::host_at(b[3])
                                .copy_from_nonoverlapping(length.as_ptr(), 10);
                        }
                    });
                });
            },
        );
    }

    #[test]
    fn pull_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x200,
                    hi: 0x1D0826,
                    reads: &[
                        (0x22C, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                        (0x1D07E4, F32),
                        (0x1D07E8, F32),
                        (0x1D07EC, F32),
                    ],
                    writes: &[
                        (0x1D0774, F32),
                        (0x1D07D8, F32),
                        (0x1D07DC, F32),
                        (0x1D07E0, F32),
                        (0x1D07E4, F32),
                        (0x1D07E8, F32),
                        (0x1D07EC, F32),
                    ],
                },
                Op::Base {
                    lo: 0x4C,
                    hi: 0x8C,
                    reads: &[(0x4C, F32), (0x50, F32), (0x54, F32), (0x88, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xA,
                    reads: &[(0x0, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::pull_facing_x87(b[0], b[1], b[2]),
            |b| {
                with_copy(b[0], MarksFloats::START, |f| {
                    with_copy(b[0], RingWork::START, |w| {
                        let car = car_with(b[1], 0, &[0x4C, 0x50, 0x54, 0x88]);
                        let length: [u8; 10] = std::array::from_fn(|i| unsafe {
                            crate::pointers::host_at(b[2] + i).read()
                        });
                        super::pull_facing_x87(&car, &length, f, w);
                    });
                });
            },
        );
    }

    #[test]
    fn pull_strength_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0x1D07FC,
                    reads: &[(0x90, I32), (0x22C, F32), (0x29C, F32), (0x1D0780, F32)],
                    writes: &[(0x1D0780, F32), (0x1D0780, I32)],
                },
                Op::Base {
                    lo: 0x64,
                    hi: 0x68,
                    reads: &[(0x64, F32)],
                    writes: &[(0x64, F32)],
                },
                Op::Base {
                    lo: 0x88,
                    hi: 0x8C,
                    reads: &[(0x88, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x50,
                    hi: 0x58,
                    reads: &[(0x50, F32), (0x54, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::pull_strength_x87(b[0], b[1], b[2], b[3]),
            |b| {
                with_copy(b[0], MarksFloats::START, |f| {
                    with_copy(b[0], RingWork::START, |w| {
                        crate::effects::ring_entries::with_entry_at(ring, b[1], 0x64, 0x68, |e| {
                            let car = car_with(b[2], 0, &[0x88]);
                            let kinds = super::RingKinds {
                                near_scale: f32::from_bits(word(b[3] + 0x50)),
                                near_rate: f32::from_bits(word(b[3] + 0x54)),
                                ..crate::effects::owner::effects().ring.kinds
                            };
                            super::pull_strength_x87(
                                ring,
                                e,
                                &car,
                                crate::effects::setup::clock_at(b[0]),
                                f,
                                w,
                                &kinds,
                            );
                        });
                    });
                });
            },
        );
    }

    #[test]
    fn box_place_x87_matches() {
        let ring = &effects().ring;
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x1D06E0,
                    hi: 0x1D0826,
                    reads: &[],
                    writes: &[(0x1D0740, I32), (0x1D0744, I32)],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x2C,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::box_place_x87(b[0], b[1], b[2]),
            |b| {
                with_copy(b[0], RingWork::START, |w| {
                    crate::effects::ring_entries::with_entry_at(ring, b[1], 0x10, 0x1C, |e| {
                        super::box_place_x87(ring, w, e, &car_matrix(b[2]));
                    });
                });
            },
        );
    }
}
