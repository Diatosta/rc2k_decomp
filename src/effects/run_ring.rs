//! `effects::run_ring` (0x4CB64D, with its chunk 0x4CE681 .. 0x4CF85E): the emitters' ring's
//! frame. Each of `g_marksRingCount` entries from `g_marksRingCursor` along +4 stepped by
//! its kind (+0xC; others nothing): the life (+0x84) less `g_marksFrameTicks` (dies when
//! that borrows) and, but for the boxed kinds' own test, death below its ground (+0x28,
//! `fcomp`). When the settings' entry flag (`g_marksSettings` + the entry's +8 + 0x60) is
//! set it is drawn: kinds 5, 0xD with their place on the object's (`[[+0x68] + 0x24] +
//! 0x30` .., the sums unrounded), 6, 0x15, 1, 0x13, 0x16 at their place, by
//! `effects::draw_cell_sprite` (0xB by its twin `draw_cell_sprite_2`; cell +0x58, angle
//! +0x50, alpha +0x74, size +0x60 as whole numbers or as they are); 0x10, 0x11 by
//! `effects::draw_subcell_sprite` (subcell +0x58 plus the life's frame of four). Then the
//! angle on (+0x54 times `flt_764FD4`, kept to 12 bits), the place moved by the motion, the
//! kind's gains, dampings (a part that changes sign stops at 0), the size pulled toward
//! +0x70 (kinds 6, 0x15 only while outside the object's box and not facing it; kind 1 also
//! its +0x64 pulled toward `flt_7651DC` when near and facing), the alpha faded by +0x78;
//! kinds 0x10, 0x11 die inside the object's box above `flt_935670`. A dead entry is moved
//! to just before the cursor (or that moved on past it).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its one caller reads no register after it)
//!
//! Caller: `effects::run_frame` (0x4D163B, after `run_ribbons`' set-up). The longer x87 runs
//! are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the entries and the game's variables. Refuses,
//! before anything, null settings, a null entry or link on the way, a null object where one
//! is read, and the draws' own refusals for an entry that may be drawn.

use crate::effects::emitters::matrix_cell;
use crate::effects::ring_entries::{get as eg, ld as eld, put as ep, st as est};
use crate::effects::ring_store::{RING_WORK, RingWork};
use crate::effects::setup::{CLOCK, MarksClock};
use crate::effects::tuning::{MARKS_FLOATS, MarksFloats};
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::marks_settings_store::{MarkCar, MarksSettings, car_or_none};
use crate::scene::matrix3::extended;
use crate::session::Float;

/// The settings' flag for a ring entry's tag (its +8, a car's number times 0x4C8 for the
/// cars' emitters): the word at the tag + 0x60 in the settings.
fn entry_flag(marks: &MarksSettings, tag: u32) -> Result<u32, BadInput> {
    marks.at(tag.wrapping_add(0x60)).read("a mark's field")
}

fn k(cell: &Float) -> u32 {
    cell.bits()
}

/// The original's 0x4CE9AF .. 0x4CEA40 (and 0x4CEC77 ..): the place less the object's (`[p + 0x30]` ..) into `flt_935718` .., turned by the object's matrix (`m`), |x| and |z| `fistp`ed into `dword_935680` / `dword_935684`.
/// The car a ring entry is on (its +0x68, a `marks_settings_store::car_handle`).
fn entry_car(e: u32) -> Result<&'static MarkCar, Refused> {
    car_or_none(eg(e, 0x68)?).ok_or_else(|| BadInput("a ring entry without its car").into())
}

fn box_(e: u32, whole: &[Float; 16], car: &MarkCar) {
    box_x87(&RING_WORK, e, whole, car);
}

/// The car's whole matrix (its block's), refused where the original would read address 0.
fn whole_of(car: &MarkCar) -> Result<&'static [Float; 16], BadInput> {
    crate::race::blocks_store::car_of(car.word(0x24)?)
        .map(|b| &b.whole_matrix)
        .ok_or(BadInput("a ring entry without its object"))
}

/// The original's 0x4CEA5E .. 0x4CEAD4 (and 0x4CED26 ..): that way made one long (`flt_76516C` over its length), dotted with the object's +0x4C .. into `flt_9356B4`.
fn facing(car: &MarkCar) {
    facing_x87(car, &MARKS_FLOATS, &RING_WORK);
}

/// The original's 0x4CEF8A .. 0x4CEFDA: the place less the object's into `flt_935718` .., its length (kept, 80 bits, at `l`) less `flt_93A526` into `flt_9356BC`.
fn pull_length(e: u32, whole: &[Float; 16], length: &mut [u8; 10]) {
    pull_length_x87(e, whole, length, &RING_WORK, &RING_KINDS);
}

/// The original's 0x4CEFEC .. 0x4CF07F: the way made one long by the kept length, the object's +0x4C .. over its +0x88, minus their dot into `flt_9356B4`.
fn pull_facing(car: &MarkCar, length: &[u8; 10]) {
    pull_facing_x87(car, length, &MARKS_FLOATS, &RING_WORK);
}

/// The original's 0x4CF08B .. 0x4CF0D8: the entry's +0x64 pulled toward `flt_7651DC` (`fist` into `dword_9356C0`).
fn pull_strength(e: u32, car: &MarkCar) {
    pull_strength_x87(e, car, &CLOCK, &MARKS_FLOATS, &RING_WORK, &RING_KINDS);
}

/// The original's 0x4CF65C .. 0x4CF6B7 (and 0x4CF7C4 ..): the place turned by the object's matrix (`m`), |x| and |z| into `dword_935680` / `dword_935684`.
fn box_place(e: u32, car: &MarkCar) {
    box_place_x87(&RING_WORK, e, car);
}

/// The kinds' constants: (the y gain, the three dampings, the size's rate).
struct Kind {
    gain: Option<&'static Float>,
    damp: [&'static Float; 3],
    pull: Option<&'static Float>,
}

impl Kind {
    /// The size's rate (the kinds that pull the size have one; the original would read
    /// address 0 for one that has none: refused).
    fn pull(&self) -> Result<&'static Float, BadInput> {
        self.pull
            .ok_or(BadInput("a ring kind that pulls the size without a rate"))
    }

    /// The y gain (the kinds that gain have one; as `pull`).
    fn gain(&self) -> Result<&'static Float, BadInput> {
        self.gain
            .ok_or(BadInput("a ring kind that gains without a gain"))
    }
}

static K6: Kind = Kind {
    gain: None,
    damp: [
        &RING_KINDS.k6_damp[0],
        &RING_KINDS.k6_damp[1],
        &RING_KINDS.k6_damp[2],
    ],
    pull: Some(&RING_KINDS.k6_pull),
};
static K15: Kind = Kind {
    gain: None,
    damp: [
        &RING_KINDS.k15_damp[0],
        &RING_KINDS.k15_damp[1],
        &RING_KINDS.k15_damp[2],
    ],
    pull: Some(&RING_KINDS.k15_pull),
};
static K1: Kind = Kind {
    gain: None,
    damp: [
        &RING_KINDS.k1_damp[0],
        &RING_KINDS.k1_damp[1],
        &RING_KINDS.k1_damp[2],
    ],
    pull: Some(&RING_KINDS.k1_pull),
};
static KD: Kind = Kind {
    gain: Some(&RING_KINDS.kd_gain),
    damp: [
        &RING_KINDS.kd_damp[0],
        &RING_KINDS.kd_damp[1],
        &RING_KINDS.kd_damp[2],
    ],
    pull: Some(&RING_KINDS.kd_pull),
};
static K13: Kind = Kind {
    gain: Some(&RING_KINDS.k13_gain),
    damp: [
        &RING_KINDS.k13_damp[0],
        &RING_KINDS.k13_damp[1],
        &RING_KINDS.k13_damp[2],
    ],
    pull: None,
};
static K16: Kind = Kind {
    gain: Some(&RING_KINDS.k16_gain),
    damp: [
        &RING_KINDS.k16_damp[0],
        &RING_KINDS.k16_damp[1],
        &RING_KINDS.k16_damp[2],
    ],
    pull: None,
};

/// Why it refuses, if it does: the walk's pointers checked, nothing written.
fn refusal() -> Result<Option<&'static str>, Refused> {
    Ok({
        let Ok(marks) = crate::effects::setup::marks() else {
            return Ok(Some("no marks settings"));
        };
        let mut e: u32 = RING_WORK.cursor.get();
        let count = RING_WORK.count.get() as i32;
        if count > 0 && (e == 0 || eg(e, 0)? == 0) {
            return Ok(Some("a null ring entry"));
        }
        for _ in 0..count.max(0) {
            if e == 0 || eg(e, 0)? == 0 || eg(e, 4)? == 0 {
                return Ok(Some("a null ring entry or link"));
            }
            let kind = eg(e, 0xC)?;
            if [5, 0xB, 6, 0x15, 1, 0xD, 0x13, 0x16, 0x10, 0x11].contains(&kind) {
                let obj = car_or_none(eg(e, 0x68)?);
                let needs = matches!(kind, 6 | 0x15 | 1 | 0x10 | 0x11)
                    || entry_flag(marks, eg(e, 8)?)? != 0;
                if needs && !obj.map_or(Ok(false), |c| c.has_block(0x24))? {
                    return Ok(Some("a ring entry without its object"));
                }
                // The draws' own refusals, for an entry that may be drawn.
                if entry_flag(marks, eg(e, 8)?)? != 0 {
                    if crate::scene::fog_at::fog_at(0).is_err() {
                        return Ok(Some("the fog table is not there"));
                    }
                    let bad = if matches!(kind, 0x10 | 0x11) {
                        eg(e, 0x58)?.wrapping_add(3) >= 0x410
                    } else {
                        eg(e, 0x58)? > 0x40 || eg(e, 0x50)? >= 0x1000
                    };
                    if bad {
                        return Ok(Some("a ring entry's cell or angle past its table"));
                    }
                }
            }
            e = eg(e, 4)?;
        }
        None
    })
}

/// The life and the ground: whether the entry dies.
fn aged(e: u32, ground: bool) -> Result<bool, Refused> {
    Ok({
        let t: u32 = CLOCK.frame_ticks.get();
        let life = eg(e, 0x84)?;
        ep(e, 0x84, life.wrapping_sub(t))?;
        if life < t {
            return Ok(true);
        }
        ground && x87::compare_status(eg(e, 0x14)?, eg(e, 0x28)?) == 0x100
    })
}

/// The place on the object's (unrounded sums), st(0) first.
fn on_object(e: u32) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        let m = entry_car(e)?.whole()?;
        [
            x87::add_80(eg(e, 0x10)?, matrix_cell(m, 0x30)?),
            x87::add_80(eg(e, 0x14)?, matrix_cell(m, 0x34)?),
            x87::add_80(eg(e, 0x18)?, matrix_cell(m, 0x38)?),
        ]
    })
}

fn at_place(e: u32) -> Result<[[u8; 10]; 3], Refused> {
    Ok({
        [
            extended(eg(e, 0x10)?),
            extended(eg(e, 0x14)?),
            extended(eg(e, 0x18)?),
        ]
    })
}

/// The settings' flag for the entry.
fn shown(e: u32) -> Result<bool, Refused> {
    Ok(entry_flag(crate::effects::setup::marks()?, eg(e, 8)?)? != 0)
}

/// The alpha (top byte) as a whole number (`fistp` through `dword_9356C0`).
fn whole_alpha(e: u32) -> Result<u32, Refused> {
    Ok({
        let a = x87::to_int(eg(e, 0x74)?);
        RING_WORK.whole.set(a as u32);
        (a as u32) << 24
    })
}

/// The size as a whole number (through `dword_9356C0`).
fn whole(v: i32) -> u32 {
    RING_WORK.whole.set(v as u32);
    v as u32
}

/// The angle on and the place moved.
fn spin_and_move(e: u32, spin: bool) -> Result<(), Refused> {
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        if spin {
            let turn = x87::int_times_to_int(eg(e, 0x54)? as i32, CLOCK.ticks_scaled.bits()) as u32;
            ep(e, 0x50, eg(e, 0x50)?.wrapping_add(turn) & 0xFFF)?;
        }
        for o in [0x10u32, 0x14, 0x18] {
            ep(e, o, x87::plus_times_int(eg(e, o)?, eg(e, o + 0x1C)?, t))?;
        }
    }
    Ok(())
}

fn gain_y(e: u32, k_addr: &Float) -> Result<(), Refused> {
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        ep(e, 0x30, x87::plus_times_int(eg(e, 0x30)?, k(k_addr), t))?;
    }
    Ok(())
}

/// The three dampings (a part that changes sign stops at 0).
fn damp3(e: u32, ks: [&Float; 3]) -> Result<(), Refused> {
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        for (o, kk) in [0x2Cu32, 0x30, 0x34].into_iter().zip(ks) {
            let old = eg(e, o)?;
            RING_WORK.whole.set(old);
            let new = x87::damp(old, k(kk), t);
            ep(e, o, new)?;
            if (old ^ new) & 0x8000_0000 != 0 {
                ep(e, o, 0)?;
            }
        }
    }
    Ok(())
}

/// The size pulled toward +0x70.
fn pull_size(e: u32, k_addr: &Float) -> Result<(), Refused> {
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        let (w, f) = x87::approach(eg(e, 0x70)? as i32, eg(e, 0x60)?, k(k_addr), t);
        RING_WORK.whole.set(w as u32);
        ep(e, 0x60, f)?;
        if w > eg(e, 0x70)? as i32 {
            ep(e, 0x60, x87::int_to_float(eg(e, 0x70)? as i32))?;
        }
    }
    Ok(())
}

fn fade(e: u32) -> Result<(), Refused> {
    {
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        ep(e, 0x74, x87::less_times_int(eg(e, 0x74)?, eg(e, 0x78)?, t))?;
    }
    Ok(())
}

/// The box test of kinds 6 and 0x15: whether the size is pulled.
fn outside_and_away(e: u32) -> Result<bool, Refused> {
    Ok({
        let car = entry_car(e)?;
        box_(e, whole_of(car)?, car);
        let outside = (eg(e, 0x7C)? as i32) < (RING_WORK.extent[0].get() as i32)
            || (eg(e, 0x80)? as i32) < (RING_WORK.extent[1].get() as i32);
        if !outside {
            return Ok(false);
        }
        facing(car);
        (RING_WORK.facing.bits() as i32) <= 0
    })
}

/// The draws: 1 `draw_cell_sprite`, 2 its twin (the same Rust version), 3
/// `draw_subcell_sprite`; `cell` is the (sub)cell, `size` the size.
fn draw(
    place: u32,
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    e: u32,
    p: [[u8; 10]; 3],
) -> Result<(), Refused> {
    use crate::effects::{draw_cell_sprite as c, draw_subcell_sprite as s};
    {
        match place {
            3 => s::draw_subcell_sprite(cell, size, e, p),
            _ => c::draw_cell_sprite(cell, angle, alpha, size, e, p),
        }
    }
}

/// Kinds 5 and 0xB (draw 1 or 2; their y gain, size and alpha gains): whether it dies.
fn drift_kind(e: u32, place: u32, ks: [&Float; 3]) -> Result<bool, Refused> {
    {
        if aged(e, true)? {
            return Ok(true);
        }
        if shown(e)? {
            let p = on_object(e)?;
            let [cell, angle, alpha, size] =
                [eg(e, 0x58)?, eg(e, 0x50)?, eg(e, 0x74)?, eg(e, 0x60)?];
            draw(place, cell, angle, alpha, size, e, p)?;
        }
        spin_and_move(e, true)?;
        gain_y(e, ks[0])?;
        let t: i32 = CLOCK.frame_ticks.get() as i32;
        ep(
            e,
            0x60,
            x87::times_int_plus_int_to_int(k(ks[1]), t, eg(e, 0x60)? as i32) as u32,
        )?;
        ep(
            e,
            0x74,
            x87::int_times_int_plus_int(k(ks[2]) as i32, t, eg(e, 0x74)? as i32) as u32,
        )?;
    }
    Ok(false)
}

/// Kinds 6, 0x15, 1, 0xD, 0x13, 0x16 (draw 1): whether it dies.
fn fading_kind(e: u32, kind: u32, c: &Kind) -> Result<bool, Refused> {
    {
        if aged(e, true)? {
            return Ok(true);
        }
        if (eg(e, 0x74)? as i32) < 0 {
            ep(e, 0x74, 0)?;
        }
        if shown(e)? {
            let p = if kind == 0xD {
                on_object(e)?
            } else {
                at_place(e)?
            };
            let [cell, angle] = [eg(e, 0x58)?, eg(e, 0x50)?];
            let alpha = whole_alpha(e)?;
            let size = if kind == 1 {
                x87::plus_product_to_int(eg(e, 0x60)?, eg(e, 0x64)?, k(&RING_KINDS.k1_size))
            } else {
                x87::to_int(eg(e, 0x60)?)
            };
            let size = whole(size);
            draw(1, cell, angle, alpha, size, e, p)?;
        }
        spin_and_move(e, true)?;
        if c.gain.is_some() && kind == 0xD {
            gain_y(e, c.gain()?)?;
        }
        damp3(e, c.damp)?;
        match kind {
            6 | 0x15 => {
                if outside_and_away(e)? {
                    pull_size(e, c.pull()?)?;
                }
                fade(e)?;
            }
            1 => {
                pull_size(e, c.pull()?)?;
                fade(e)?;
                pull_toward(e)?;
            }
            0xD => {
                pull_size(e, c.pull()?)?;
                fade(e)?;
            }
            _ => {
                fade(e)?;
                let t: i32 = CLOCK.frame_ticks.get() as i32;
                ep(e, 0x60, x87::plus_times_int(eg(e, 0x60)?, k(c.gain()?), t))?;
            }
        }
    }
    Ok(false)
}

/// Kind 1's pull of its +0x64 when near and facing.
fn pull_toward(e: u32) -> Result<(), Refused> {
    {
        let car = entry_car(e)?;
        let mut length = [0u8; 10];
        pull_length(e, whole_of(car)?, &mut length);
        if (RING_WORK.near.bits() as i32) < 0 {
            return Ok(());
        }
        pull_facing(car, &length);
        if (RING_WORK.facing.bits() as i32) <= 0 {
            return Ok(());
        }
        pull_strength(e, car);
        if (RING_WORK.whole.get() as i32) > (MARKS_FLOATS.pull_limit.bits() as i32) {
            ep(e, 0x64, x87::loaded(MARKS_FLOATS.two_pow_16.bits()))?;
        }
    }
    Ok(())
}

/// Kinds 0x10, 0x11 (draw 3): boxed, no ground test of their own first. Whether it dies.
fn boxed_kind(e: u32, gain: &Float, both: bool) -> Result<bool, Refused> {
    if aged(e, true)? {
        return Ok(true);
    }
    spin_and_move(e, false)?;
    gain_y(e, gain)?;
    if shown(e)? {
        let p = on_object(e)?;
        let size = eg(e, 0x60)?;
        let f = x87::int_times_times_to_int(
            eg(e, 0x84)? as i32,
            MARKS_FLOATS.twentieth.bits(),
            eg(e, 0x5C)?,
        );
        RING_WORK.whole.set(f as u32);
        let subcell = ((f as u32) & 3).wrapping_add(eg(e, 0x58)?);
        draw(3, subcell, 0, 0, size, e, p)?;
    }
    box_place(e, entry_car(e)?);
    let mut outside = (eg(e, 0x7C)? as i32) < (RING_WORK.extent[0].get() as i32);
    if !outside && both {
        outside = (eg(e, 0x80)? as i32) < (RING_WORK.extent[1].get() as i32);
    }
    if !outside && x87::compare_status(eg(e, 0x14)?, RING_WORK.top.bits()) == 0 {
        return Ok(true);
    }
    Ok(x87::compare_status(eg(e, 0x14)?, eg(e, 0x28)?) == 0x100)
}

/// The Rust version (see the header).
pub fn run_ring() -> Result<(), Refused> {
    if let Some(why) = refusal()? {
        return Err(BadInput(why).into());
    }
    {
        let mut e: u32 = RING_WORK.cursor.get();
        let mut counter = RING_WORK.count.get() as i32;
        loop {
            counter = counter.wrapping_sub(1);
            if counter < 0 {
                break;
            }
            let next = eg(e, 4)?;
            let dies = match eg(e, 0xC)? {
                5 => drift_kind(
                    e,
                    1,
                    [
                        &RING_KINDS.drift5[0],
                        &RING_KINDS.drift5[1],
                        &RING_KINDS.drift5[2],
                    ],
                )?,
                0xB => drift_kind(
                    e,
                    2,
                    [
                        &RING_KINDS.drift_b[0],
                        &RING_KINDS.drift_b[1],
                        &RING_KINDS.drift_b[2],
                    ],
                )?,
                6 => fading_kind(e, 6, &K6)?,
                0x15 => fading_kind(e, 0x15, &K15)?,
                1 => fading_kind(e, 1, &K1)?,
                0xD => fading_kind(e, 0xD, &KD)?,
                0x13 => fading_kind(e, 0x13, &K13)?,
                0x16 => fading_kind(e, 0x16, &K16)?,
                0x10 => boxed_kind(e, &RING_KINDS.k10_gain, true)?,
                0x11 => boxed_kind(e, &RING_KINDS.k11_gain, false)?,
                _ => false,
            };
            if dies {
                RING_WORK.count.set(RING_WORK.count.get().wrapping_sub(1));
                if e == RING_WORK.cursor.get() {
                    RING_WORK.cursor.set(eg(e, 4)?);
                } else {
                    let (before, after) = (eg(e, 0)?, eg(e, 4)?);
                    ep(before, 4, after)?;
                    ep(after, 0, before)?;
                    let cursor: u32 = RING_WORK.cursor.get();
                    let last = eg(cursor, 0)?;
                    ep(cursor, 0, e)?;
                    ep(last, 4, e)?;
                    ep(e, 0, last)?;
                    ep(e, 4, cursor)?;
                }
            }
            e = next;
        }
    }
    Ok(())
}

use crate::effects::plain::{ld_cell, ldi_cell};
use crate::math::x87_float as bh;

/// `fld dword ptr [car + off]` of a car record in the settings.
fn car_float(car: &MarkCar, off: usize) -> bh::X {
    bh::ld(car.head[off / 4].get())
}

/// `fld dword ptr [m + off]` of the car's 3 x 3 matrix (the record's +0x94).
fn turn(car: &MarkCar, off: usize) -> bh::X {
    car_float(car, 0x94 + off)
}

/// `box_`'s x87 block as plain Rust on the x87 model (45 instructions), same order and memory places.
fn box_x87(w: &RingWork, e: u32, whole: &[Float; 16], car: &MarkCar) {
    let t0 = eld(e, 0x10);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].set_bits(bh::st(t1));
    let t2 = eld(e, 0x14);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].set_bits(bh::st(t3));
    let t4 = eld(e, 0x18);
    let t5 = t4 - ld_cell(&whole[14]);
    w.away[2].set_bits(bh::st(t5));
    let t6 = bh::ld(w.away[2].bits());
    let t7 = bh::ld(w.away[1].bits());
    let t8 = bh::ld(w.away[0].bits());
    let t9 = turn(car, 0);
    let t10 = t9 * t8;
    let t11 = turn(car, 0x10);
    let t12 = t11 * t7;
    let t13 = turn(car, 0x20);
    let t14 = t13 * t6;
    let t15 = t12 + t14;
    let t16 = t10 + t15;
    let t17 = turn(car, 0x4);
    let t18 = t17 * t8;
    let t19 = turn(car, 0x14);
    let t20 = t19 * t7;
    let t21 = turn(car, 0x24);
    let t22 = t21 * t6;
    let t23 = t20 + t22;
    let _t24 = t18 + t23;
    let t25 = turn(car, 0x8);
    let t26 = t8 * t25;
    let t27 = turn(car, 0x18);
    let t28 = t7 * t27;
    let t29 = turn(car, 0x28);
    let t30 = t6 * t29;
    let t31 = t30 + t28;
    let t32 = t26 + t31;
    let t33 = t16.abs();
    w.extent[0].set_bits(bh::to_int(t33) as u32);
    let t34 = t32.abs();
    w.extent[1].set_bits(bh::to_int(t34) as u32);
}

/// `facing`'s x87 block as plain Rust on the x87 model (28 instructions), same order and memory places.
fn facing_x87(car: &MarkCar, f: &MarksFloats, w: &RingWork) {
    let t0 = bh::ld(w.away[0].bits());
    let t1 = t0 * t0;
    let t2 = bh::ld(w.away[1].bits());
    let t3 = t2 * t2;
    let t4 = bh::ld(w.away[2].bits());
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = bh::ld(f.one.bits()) / t8;
    let t10 = bh::ld(w.away[0].bits());
    let t11 = t10 * t9;
    w.away[0].set_bits(bh::st(t11));
    let t12 = bh::ld(w.away[1].bits());
    let t13 = t12 * t9;
    w.away[1].set_bits(bh::st(t13));
    let t14 = bh::ld(w.away[2].bits());
    let t15 = t9 * t14;
    w.away[2].set_bits(bh::st(t15));
    let t16 = bh::ld(w.away[0].bits());
    let t17 = t16 * car_float(car, 0x4C);
    let t18 = bh::ld(w.away[1].bits());
    let t19 = t18 * car_float(car, 0x50);
    let t20 = bh::ld(w.away[2].bits());
    let t21 = t20 * car_float(car, 0x54);
    let t22 = t19 + t21;
    let t23 = t17 + t22;
    w.facing.set_bits(bh::st(t23));
}

/// `pull_length`'s x87 block as plain Rust on the x87 model (22 instructions), same order and memory places.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_length_x87(e: u32, whole: &[Float; 16], l: &mut [u8; 10], w: &RingWork, rk: &RingKinds) {
    let t0 = eld(e, 0x10);
    let t1 = t0 - ld_cell(&whole[12]);
    w.away[0].set_bits(bh::st(t1));
    let t2 = eld(e, 0x14);
    let t3 = t2 - ld_cell(&whole[13]);
    w.away[1].set_bits(bh::st(t3));
    let t4 = eld(e, 0x18);
    let t5 = t4 - ld_cell(&whole[14]);
    w.away[2].set_bits(bh::st(t5));
    let t6 = bh::ld(w.away[0].bits());
    let t7 = t6 * t6;
    let t8 = bh::ld(w.away[1].bits());
    let t9 = t8 * t8;
    let t10 = bh::ld(w.away[2].bits());
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 - ld_cell(&rk.near_less);
    w.near.set_bits(bh::st(t15));
    *l = bh::f80(t14);
}

/// `pull_facing`'s x87 block as plain Rust on the x87 model (32 instructions), same order and memory places.
fn pull_facing_x87(car: &MarkCar, l: &[u8; 10], f: &MarksFloats, w: &RingWork) {
    let t0 = bh::from_f80(*l);
    let t1 = bh::ld(f.one.bits()) / t0;
    let t2 = bh::ld(w.away[0].bits());
    let t3 = t2 * t1;
    w.away[0].set_bits(bh::st(t3));
    let t4 = bh::ld(w.away[1].bits());
    let t5 = t4 * t1;
    w.away[1].set_bits(bh::st(t5));
    let t6 = bh::ld(w.away[2].bits());
    let t7 = t1 * t6;
    w.away[2].set_bits(bh::st(t7));
    let t8 = ldi_cell(&car.head[0x88 / 4]);
    let t9 = bh::ld(f.one.bits()) / t8;
    let t10 = car_float(car, 0x4C);
    let t11 = t10 * t9;
    w.turned[0].set_bits(bh::st(t11));
    let t12 = car_float(car, 0x50);
    let t13 = t12 * t9;
    w.turned[1].set_bits(bh::st(t13));
    let t14 = car_float(car, 0x54);
    let t15 = t9 * t14;
    w.turned[2].set_bits(bh::st(t15));
    let t16 = bh::ld(w.away[0].bits());
    let t17 = t16 * bh::ld(w.turned[0].bits());
    let t18 = bh::ld(w.away[1].bits());
    let t19 = t18 * bh::ld(w.turned[1].bits());
    let t20 = bh::ld(w.away[2].bits());
    let t21 = t20 * bh::ld(w.turned[2].bits());
    let t22 = t19 + t21;
    let t23 = t17 + t22;
    let t24 = -t23;
    w.facing.set_bits(bh::st(t24));
}

/// `pull_strength`'s x87 block as plain Rust on the x87 model (15 instructions), same order and memory places.
/// `rk` the ring kinds' constants (`RingKinds`).
fn pull_strength_x87(
    e: u32,
    car: &MarkCar,
    c: &MarksClock,
    f: &MarksFloats,
    w: &RingWork,
    rk: &RingKinds,
) {
    let t0 = ldi_cell(&car.head[0x88 / 4]);
    let t1 = t0 * ld_cell(&rk.near_scale);
    let t2 = t1 + bh::ld(f.one.bits());
    let t3 = bh::ld(f.one.bits()) / t2;
    let t4 = bh::ld(f.one.bits()) - t3;
    w.whole.set_bits(bh::st(t4));
    let t5 = bh::ld(f.two_pow_16.bits());
    let t6 = t5 - eld(e, 0x64);
    let t7 = ld_cell(&rk.near_rate);
    let t8 = t7 * bh::ld(w.whole.bits());
    let t9 = t6 * t8;
    let t10 = t9 * crate::math::x87_float::int(c.frame_ticks.get() as i32);
    let t11 = t10 + eld(e, 0x64);
    w.whole.set_bits(bh::to_int(t11) as u32);
    est(e, 0x64, t11);
}

/// `box_place`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
fn box_place_x87(w: &RingWork, e: u32, car: &MarkCar) {
    let t0 = eld(e, 0x18);
    let t1 = eld(e, 0x14);
    let t2 = eld(e, 0x10);
    let t3 = turn(car, 0);
    let t4 = t3 * t2;
    let t5 = turn(car, 0x10);
    let t6 = t5 * t1;
    let t7 = turn(car, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = turn(car, 0x4);
    let t12 = t11 * t2;
    let t13 = turn(car, 0x14);
    let t14 = t13 * t1;
    let t15 = turn(car, 0x24);
    let t16 = t15 * t0;
    let t17 = t14 + t16;
    let _t18 = t12 + t17;
    let t19 = turn(car, 0x8);
    let t20 = t2 * t19;
    let t21 = turn(car, 0x18);
    let t22 = t1 * t21;
    let t23 = turn(car, 0x28);
    let t24 = t0 * t23;
    let t25 = t24 + t22;
    let t26 = t20 + t25;
    let t27 = t10.abs();
    w.extent[0].set_bits(bh::to_int(t27) as u32);
    let t28 = t26.abs();
    w.extent[1].set_bits(bh::to_int(t28) as u32);
}

/// The ring kinds' constants (`run_ring`, and the ribbons' two) (IDA 0x93A4DA .. 0x93A576, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RingKinds {
    /// 0x93A4DA: `flt_93A4DA`, `flt_93A4DE`, `dword_93A4E2`: kind 5's drift factors
    pub drift5: [Float; 3],
    /// 0x93A4E6: `flt_93A4E6`, `flt_93A4EA`, `dword_93A4EE`: kind 0xB's drift factors
    pub drift_b: [Float; 3],
    /// 0x93A4F2: `flt_93A4F2` .. `flt_93A4FA`: kind 6's three dampings
    pub k6_damp: [Float; 3],
    /// 0x93A4FE: `flt_93A4FE`: kind 6's size rate
    pub k6_pull: Float,
    /// 0x93A502: `flt_93A502` .. `flt_93A50A`: kind 0x15's three dampings
    pub k15_damp: [Float; 3],
    /// 0x93A50E: `flt_93A50E`: kind 0x15's size rate
    pub k15_pull: Float,
    /// 0x93A512: `flt_93A512`: kind 1's size factor (+0x64 times it)
    pub k1_size: Float,
    /// 0x93A516: `flt_93A516` .. `flt_93A51E`: kind 1's three dampings
    pub k1_damp: [Float; 3],
    /// 0x93A522: `flt_93A522`: kind 1's size rate
    pub k1_pull: Float,
    /// 0x93A526: `flt_93A526`: less this from the length to the object (`pull_length`)
    pub near_less: Float,
    /// 0x93A52A: `flt_93A52A`: the strength's pull scale (`pull_strength`)
    pub near_scale: Float,
    /// 0x93A52E: `flt_93A52E`: the strength's pull rate (`pull_strength`)
    pub near_rate: Float,
    /// 0x93A532: `flt_93A532`: kind 0xD's y gain
    pub kd_gain: Float,
    /// 0x93A536: `flt_93A536` .. `flt_93A53E`: kind 0xD's three dampings
    pub kd_damp: [Float; 3],
    /// 0x93A542: `flt_93A542`: kind 0xD's size rate
    pub kd_pull: Float,
    /// 0x93A546: `flt_93A546` .. `flt_93A54E`: kind 0x13's three dampings
    pub k13_damp: [Float; 3],
    /// 0x93A552: `flt_93A552`: kind 0x13's y gain
    pub k13_gain: Float,
    /// 0x93A556: `flt_93A556` .. `flt_93A55E`: kind 0x16's three dampings
    pub k16_damp: [Float; 3],
    /// 0x93A562: `flt_93A562`: kind 0x16's y gain
    pub k16_gain: Float,
    /// 0x93A566: `flt_93A566`: kind 0x10's y gain
    pub k10_gain: Float,
    /// 0x93A56A: `flt_93A56A`: kind 0x11's y gain
    pub k11_gain: Float,
    /// 0x93A56E: `flt_93A56E`: a ribbon's y gain (`run_ribbons::first_frame`)
    pub ribbon_a: Float,
    /// 0x93A572: `flt_93A572`: a ribbon's y gain per frame (`run_ribbons::move_`)
    pub ribbon_b: Float,
}

/// RingKinds's first values.
pub static RING_KINDS: RingKinds = RingKinds {
    drift5: [
        Float::from_bits(0xC0000000),
        Float::from_bits(0x42B80000),
        Float::from_bits(0xFF800000),
    ],
    drift_b: [
        Float::from_bits(0xC0000000),
        Float::from_bits(0x42B80000),
        Float::from_bits(0xFF800000),
    ],
    k6_damp: [
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3A83126F),
    ],
    k6_pull: Float::from_bits(0x3C23D70A),
    k15_damp: [
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3A83126F),
        Float::from_bits(0x3A83126F),
    ],
    k15_pull: Float::from_bits(0x3C23D70A),
    k1_size: Float::from_bits(0x3F800000),
    k1_damp: [
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
    ],
    k1_pull: Float::from_bits(0x3AC49BA6),
    near_less: Float::from_bits(0x47D8F000),
    near_scale: Float::from_bits(0x3951B717),
    near_rate: Float::from_bits(0x3BA3D70A),
    kd_gain: Float::from_bits(0x3C23D70A),
    kd_damp: [
        Float::from_bits(0x3B83126F),
        Float::from_bits(0x3B83126F),
        Float::from_bits(0x3B83126F),
    ],
    kd_pull: Float::from_bits(0x3C23D70A),
    k13_damp: [
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
    ],
    k13_gain: Float::from_bits(0x41800000),
    k16_damp: [
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
        Float::from_bits(0x3B449BA6),
    ],
    k16_gain: Float::from_bits(0x41800000),
    k10_gain: Float::from_bits(0xBF000000),
    k11_gain: Float::from_bits(0xBF000000),
    ribbon_a: Float::from_bits(0xBF000000),
    ribbon_b: Float::from_bits(0xBF000000),
};

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
            bytes(&RING_KINDS) == crate::moved::image_bytes(RingKinds::START, RingKinds::LEN),
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
    use crate::effects::ring_store::RingWork;
    use crate::effects::tuning::MarksFloats;
    use crate::race::marks_settings_store::MarkCar;
    use crate::session::{Dword, Float};

    fn word(at: usize) -> u32 {
        unsafe { crate::pointers::host_at(at).cast::<u32>().read_unaligned() }
    }

    /// A car record whose words at the record `offsets` hold the test buffer's dwords at `at + offset - base`.
    fn car_with(at: usize, base: usize, offsets: &[usize]) -> MarkCar {
        let car = MarkCar {
            head: std::array::from_fn(|_| Dword::new(0)),
            records: std::array::from_fn(|_| {
                crate::race::marks_settings_store::MarkRecord::zeroed()
            }),
            tail: std::array::from_fn(|_| Dword::new(0)),
        };
        for &o in offsets {
            car.head[o / 4].set(word(at + o - base));
        }
        car
    }

    /// The car's 3 x 3 matrix (the record's +0x94) from the buffer at `at`.
    fn car_matrix(at: usize) -> MarkCar {
        let offsets: Vec<usize> = [0x0, 0x4, 0x8, 0x10, 0x14, 0x18, 0x20, 0x24, 0x28]
            .iter()
            .map(|o| 0x94 + o)
            .collect();
        car_with(at, 0x94, &offsets)
    }

    /// Runs `f` with the ring kinds' cells holding the given bits, then as they were.
    fn with_kinds(cells: &[(&Float, u32)], f: impl FnOnce()) {
        let saved: Vec<u32> = cells.iter().map(|(c, _)| c.bits()).collect();
        for (c, v) in cells {
            c.set_bits(*v);
        }
        f();
        for ((c, _), v) in cells.iter().zip(saved) {
            c.set_bits(v);
        }
    }

    /// A whole matrix whose translation (+0x30 ..) is the buffer's at `at` + 0x30 (`at` the
    /// matrix's place, as the original reads it).
    fn whole_at(at: usize) -> [Float; 16] {
        std::array::from_fn(|i| {
            Float::from_bits(if (12..15).contains(&i) {
                word(at + 4 * i)
            } else {
                0
            })
        })
    }

    #[test]
    fn box_x87_matches() {
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
                    crate::effects::ring_entries::with_entry_at(b[1], 0x10, 0x1C, |e| {
                        super::box_x87(w, e, &whole_at(b[2]), &car_matrix(b[3]));
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
                    crate::effects::ring_entries::with_entry_at(b[1], 0x10, 0x1C, |e| {
                        let mut length = [0xAA; 10];
                        with_kinds(&[(&super::RING_KINDS.near_less, word(b[4] + 0x4C))], || {
                            super::pull_length_x87(
                                e,
                                &whole_at(b[2]),
                                &mut length,
                                w,
                                &super::RING_KINDS,
                            );
                        });
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
                        crate::effects::ring_entries::with_entry_at(b[1], 0x64, 0x68, |e| {
                            let car = car_with(b[2], 0, &[0x88]);
                            let rates = [
                                (&super::RING_KINDS.near_scale, word(b[3] + 0x50)),
                                (&super::RING_KINDS.near_rate, word(b[3] + 0x54)),
                            ];
                            with_kinds(&rates, || {
                                super::pull_strength_x87(
                                    e,
                                    &car,
                                    crate::effects::setup::clock_at(b[0]),
                                    f,
                                    w,
                                    &super::RING_KINDS,
                                );
                            });
                        });
                    });
                });
            },
        );
    }

    #[test]
    fn box_place_x87_matches() {
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
                    crate::effects::ring_entries::with_entry_at(b[1], 0x10, 0x1C, |e| {
                        super::box_place_x87(w, e, &car_matrix(b[2]));
                    });
                });
            },
        );
    }
}
