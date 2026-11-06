//! `effects::run_ribbons` (0x4CB71A, with its chunk 0x4CF85E .. 0x4CFAAF): the marks' ring
//! walked for its streaks (kind 0x12) when the settings' +0x60 is set: each entry of
//! `g_marksRingCount` from `g_marksRingCursor` along +4; a streak's life (+0x84) less
//! `g_marksFrameTicks` (dies when that borrows, or when its y is below its ground +0x28:
//! `fcomp`); a new one (x 0: no place yet) has its place turned by the view into its last
//! (+0x1C ..) and is moved once; then moved (the motion's y gains `flt_93A572` a tick), its
//! last place into `dword_8EB8E1` .., its place turned into `flt_8EB8D5` .. (kept as the
//! last when the settings' +0xC is set or +0x10 is 1); the first vertex's specular
//! `scene::fog_at` of the last depth's bits; `effects::draw_ribbon` (cell 0x20, width 0x800,
//! the alpha the colour's index, the colour `dword_939DB2`[life times +0x78], or
//! `dword_939BB2`[..] when `dword_765238` is set). A dead entry is moved to just before
//! the cursor (or that moved on past it).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_frame` (0x4D1640 .., after `sub_4CB64D`). The x87 blocks are plain Rust on the x87 model
//! (`math::x87_float`), in the original's order, over the entries and the game's variables; the fog its Rust
//! version. Refuses, before anything (the walk simulated), null settings, a null entry or
//! link on the way, and a colour index out of its table.

use crate::effects::mask_store::Masks;
use crate::effects::ring_entries::field;
use crate::effects::ring_entries::{get as eg, ld as eld, put as ep, st as est};
use crate::effects::ring_store::Ring;
use crate::effects::setup::{CLOCK, MarksClock};
use crate::effects::tuning::MARKS_KIND_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::race;
use crate::scene::fog_at::fog_at;

const STREAK: u32 = 0x12;

/// The colour table used while `MARKS_KIND_FLOATS.other` is 0: 257 colours (past `SURFACE`'s
/// 128: the table `SURFACE.colours` is the one in use while it is not).
#[repr(C)]
pub struct Colours257 {
    /// 0x939DB2 .. 0x93A1B6: the 257 colours
    pub colours: [u32; 257],
}

/// The 257 colours as the image has them: 0x00FFFF00 + 2 i for the first 128, then 0x00FFFFFF.
impl Colours257 {
    pub const fn new() -> Self {
        Colours257 {
            colours: {
                let mut colours = [0; 257];
                let mut i = 0;
                while i < 257 {
                    colours[i] = if i < 128 {
                        0x00FF_FF00 + 2 * i as u32
                    } else {
                        0x00FF_FFFF
                    };
                    i += 1;
                }
                colours
            },
        }
    }
}

#[cfg(test)]
impl Colours257 {
    pub const START: usize = 0x939DB2;
    pub const LEN: usize = 0x404;
}

/// The original's 0x4CF890 .. 0x4CF94D: a new streak's place turned into its last (+0x1C ..), and a first move.
fn first_frame(ring: &Ring, e: u32) {
    first_frame_x87(ring, e, &CLOCK, ring.kinds.ribbon_a);
}

/// The original's 0x4CF94D .. 0x4CF98C: the place moved by the motion, the motion's y by `flt_93A572`.
fn move_(ring: &Ring, e: u32) {
    move_x87(ring, e, &CLOCK, ring.kinds.ribbon_b);
}

/// The original's 0x4CF9A6 .. 0x4CFA2D: the place turned by the view into `flt_8EB8D5` ...
fn turn_place(ring: &Ring, e: u32) {
    turn_place_x87(ring, e, &DRAW_WORK);
}

/// The size of the colour table in use: 128 colours (`SURFACE.colours`) while
/// `MARKS_KIND_FLOATS.other` is not 0, else the 257 at 0x939DB2.
fn colour_count() -> i32 {
    if MARKS_KIND_FLOATS.other.bits() != 0 {
        128
    } else {
        257
    }
}

/// The colour `i` of the table in use (`i` below `colour_count`).
fn colour_at(masks: &Masks, ring: &Ring, i: usize) -> u32 {
    if MARKS_KIND_FLOATS.other.bits() != 0 {
        masks.surface.colours[i].get()
    } else {
        ring.colours.colours[i]
    }
}

/// Why it refuses, if it does: the walk as the original makes it, nothing written.
fn refusal(ring: &Ring) -> Result<Option<&'static str>, Refused> {
    Ok({
        let Ok(marks) = crate::effects::setup::marks() else {
            return Ok(Some("no marks settings"));
        };
        let ticks: u32 = CLOCK.frame_ticks.get();
        let mut e: u32 = ring.work.cursor.get();
        let count = ring.work.count.get() as i32;
        if count > 0 && (e == 0 || eg(ring, e, 0)? == 0) {
            return Ok(Some("a null ring entry"));
        }
        for _ in 0..count.max(0) {
            if e == 0 || eg(ring, e, 0)? == 0 || eg(ring, e, 4)? == 0 {
                return Ok(Some("a null ring entry or link"));
            }
            if marks.cars[0].word(0)? != 0 && eg(ring, e, field::KIND)? == STREAK {
                let life = eg(ring, e, field::LIFE)?;
                if life >= ticks
                    && x87::compare_status(
                        eg(ring, e, field::PLACE_Y)?,
                        eg(ring, e, field::GROUND_Y)?,
                    ) != 0x100
                {
                    if fog_at(0).is_err() {
                        return Ok(Some("the fog table is not there"));
                    }
                    let i = x87::int_times_to_int(
                        life.wrapping_sub(ticks) as i32,
                        eg(ring, e, field::ALPHA_FADE)?,
                    );
                    if !(0..colour_count()).contains(&i) {
                        return Ok(Some("a streak's colour index past its table"));
                    }
                }
            }
            e = eg(ring, e, 4)?;
        }
        None
    })
}

/// The streak step at e (see the header): whether the entry dies.
fn streak(ring: &Ring, e: u32) -> Result<bool, Refused> {
    let t: u32 = CLOCK.frame_ticks.get();
    let life = eg(ring, e, field::LIFE)?;
    ep(ring, e, field::LIFE, life.wrapping_sub(t))?;
    if life < t {
        return Ok(true);
    }
    if x87::compare_status(eg(ring, e, field::PLACE_Y)?, eg(ring, e, field::GROUND_Y)?) == 0x100 {
        return Ok(true);
    }
    if eg(ring, e, field::PLACE_X)? == 0 {
        first_frame(ring, e);
    }
    move_on(ring, e);
    DRAW_WORK.second[0].set_bits(eg(ring, e, field::LAST_PLACE_X)?);
    DRAW_WORK.second[1].set_bits(eg(ring, e, field::LAST_PLACE_Y)?);
    DRAW_WORK.second[2].set_bits(eg(ring, e, field::LAST_PLACE_Z)?);
    turn_place(ring, e);
    let marks = crate::effects::setup::marks()?;
    if marks.own_marks_shown.get() != 0 || marks.other_marks_shown.get() == 1 {
        ep(ring, e, field::LAST_PLACE_X, DRAW_WORK.turned[0].bits())?;
        ep(ring, e, field::LAST_PLACE_Y, DRAW_WORK.turned[1].bits())?;
        ep(ring, e, field::LAST_PLACE_Z, DRAW_WORK.turned[2].bits())?;
    }
    VERTICES.vertices[0]
        .specular
        .set(fog_at(eg(ring, e, field::LAST_PLACE_Z)?)?);
    let i = x87::int_times_to_int(
        eg(ring, e, field::LIFE)? as i32,
        eg(ring, e, field::ALPHA_FADE)?,
    );
    ring.work.whole.set(i as u32);
    let colour = colour_at(&crate::effects::owner::effects().masks, ring, i as usize);
    crate::effects::draw_ribbon::draw_ribbon(
        &crate::effects::owner::effects().masks,
        0x20,
        i as u32,
        0x800,
        colour,
    )?;
    Ok(false)
}

fn move_on(ring: &Ring, e: u32) {
    move_(ring, e);
}

/// The Rust version (see the header).
pub fn run_ribbons(ring: &Ring) -> Result<(), Refused> {
    if let Some(why) = refusal(ring)? {
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
            let next = eg(ring, e, 4)?;
            // The settings' +0x60: the first car record's first word.
            if crate::effects::setup::marks()?.cars[0].word(0)? != 0
                && eg(ring, e, field::KIND)? == STREAK
                && streak(ring, e)?
            {
                ring.work.count.set(ring.work.count.get().wrapping_sub(1));
                if e == ring.work.cursor.get() {
                    ring.work.cursor.set(eg(ring, e, 4)?);
                } else {
                    let (before, after) = (eg(ring, e, 0)?, eg(ring, e, 4)?);
                    ep(ring, before, 4, after)?;
                    ep(ring, after, 0, before)?;
                    let cursor: u32 = ring.work.cursor.get();
                    let last = eg(ring, cursor, 0)?;
                    ep(ring, cursor, 0, e)?;
                    ep(ring, last, 4, e)?;
                    ep(ring, e, 0, last)?;
                    ep(ring, e, 4, cursor)?;
                }
            }
            e = next;
        }
    }
    Ok(())
}

use crate::d3d::vertices_store::VERTICES;
use crate::effects::frame_state::{DRAW_WORK, DrawWork};
use crate::math::x87_float as bh;
use crate::math::x87_float::int;

/// `first_frame`'s x87 block as plain Rust on the x87 model (52 instructions), same order and memory places.
fn first_frame_x87(ring: &Ring, e: u32, c: &MarksClock, ribbon_a: f32) {
    let t0 = eld(ring, e, field::PLACE_Z);
    let t1 = eld(ring, e, field::PLACE_Y);
    let t2 = eld(ring, e, field::PLACE_X);
    let t3 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[0].bits());
    let t4 = t3 * t2;
    let t5 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[4].bits());
    let t6 = t5 * t1;
    let t7 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[8].bits());
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[12].bits());
    let t12 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[1].bits());
    let t13 = t12 * t2;
    let t14 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[5].bits());
    let t15 = t14 * t1;
    let t16 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[9].bits());
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[13].bits());
    let t21 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[2].bits());
    let t22 = t2 * t21;
    let t23 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[6].bits());
    let t24 = t1 * t23;
    let t25 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[10].bits());
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[14].bits());
    est(ring, e, field::LAST_PLACE_X, t11);
    est(ring, e, field::LAST_PLACE_Y, t20);
    est(ring, e, field::LAST_PLACE_Z, t29);
    let t30 = eld(ring, e, field::MOTION_X);
    let t31 = t30 * int(c.frame_ticks.get() as i32);
    let t32 = t31 + eld(ring, e, field::PLACE_X);
    est(ring, e, field::PLACE_X, t32);
    let t33 = eld(ring, e, field::MOTION_Y);
    let t34 = t33 * int(c.frame_ticks.get() as i32);
    let t35 = t34 + eld(ring, e, field::PLACE_Y);
    est(ring, e, field::PLACE_Y, t35);
    let t36 = eld(ring, e, field::MOTION_Z);
    let t37 = t36 * int(c.frame_ticks.get() as i32);
    let t38 = t37 + eld(ring, e, field::PLACE_Z);
    est(ring, e, field::PLACE_Z, t38);
    let t39 = bh::ld(ribbon_a.to_bits());
    let t40 = t39 * int(c.frame_ticks.get() as i32);
    let t41 = t40 + eld(ring, e, field::MOTION_Y);
    est(ring, e, field::MOTION_Y, t41);
}

/// `move_`'s x87 block as plain Rust on the x87 model (16 instructions), same order and memory places.
fn move_x87(ring: &Ring, e: u32, c: &MarksClock, ribbon_b: f32) {
    let t0 = eld(ring, e, field::MOTION_X);
    let t1 = t0 * int(c.frame_ticks.get() as i32);
    let t2 = t1 + eld(ring, e, field::PLACE_X);
    est(ring, e, field::PLACE_X, t2);
    let t3 = eld(ring, e, field::MOTION_Y);
    let t4 = t3 * int(c.frame_ticks.get() as i32);
    let t5 = t4 + eld(ring, e, field::PLACE_Y);
    est(ring, e, field::PLACE_Y, t5);
    let t6 = eld(ring, e, field::MOTION_Z);
    let t7 = t6 * int(c.frame_ticks.get() as i32);
    let t8 = t7 + eld(ring, e, field::PLACE_Z);
    est(ring, e, field::PLACE_Z, t8);
    let t9 = bh::ld(ribbon_b.to_bits());
    let t10 = t9 * int(c.frame_ticks.get() as i32);
    let t11 = t10 + eld(ring, e, field::MOTION_Y);
    est(ring, e, field::MOTION_Y, t11);
}

/// `turn_place`'s x87 block as plain Rust on the x87 model (36 instructions), same order and memory places.
fn turn_place_x87(ring: &Ring, e: u32, dw: &DrawWork) {
    let t0 = eld(ring, e, field::PLACE_Z);
    let t1 = eld(ring, e, field::PLACE_Y);
    let t2 = eld(ring, e, field::PLACE_X);
    let t3 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[0].bits());
    let t4 = t3 * t2;
    let t5 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[4].bits());
    let t6 = t5 * t1;
    let t7 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[8].bits());
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[12].bits());
    let t12 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[1].bits());
    let t13 = t12 * t2;
    let t14 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[5].bits());
    let t15 = t14 * t1;
    let t16 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[9].bits());
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[13].bits());
    let t21 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[2].bits());
    let t22 = t2 * t21;
    let t23 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[6].bits());
    let t24 = t1 * t23;
    let t25 = crate::math::x87_float::ld(race().frame.matrices.view_matrix[10].bits());
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + crate::math::x87_float::ld(race().frame.matrices.view_matrix[14].bits());
    dw.turned[0].set_bits(bh::st(t11));
    dw.turned[1].set_bits(bh::st(t20));
    dw.turned[2].set_bits(bh::st(t29));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn first_frame_x87(rc: usize, mk: usize, e: usize, rk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{e} + 0x1C]",
            "fstp dword ptr [{e} + 0x20]",
            "fstp dword ptr [{e} + 0x24]",
            "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{rk} + 0x94]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            e = in(reg) e,
            rk = in(reg) rk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn move_x87(mk: usize, e: usize, rk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x2C]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x10]",
            "fstp dword ptr [{e} + 0x10]",
            "fld dword ptr [{e} + 0x30]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x14]",
            "fstp dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x34]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x18]",
            "fstp dword ptr [{e} + 0x18]",
            "fld dword ptr [{rk} + 0x98]",
            "fimul dword ptr [{mk} + 0x90]",
            "fadd dword ptr [{e} + 0x30]",
            "fstp dword ptr [{e} + 0x30]",
            mk = in(reg) mk,
            e = in(reg) e,
            rk = in(reg) rk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            );
        }
    }

    pub(super) fn turn_place_x87(rc: usize, mk: usize, e: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{e} + 0x18]",
            "fld dword ptr [{e} + 0x14]",
            "fld dword ptr [{e} + 0x10]",
            "fld dword ptr [{rc} + 0x1EA2CC]",
            ".byte 0xD8, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2DC]",
            ".byte 0xD8, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2EC]",
            ".byte 0xD8, 0xCD",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA2FC]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D0]",
            ".byte 0xD8, 0xCA",
            "fld dword ptr [{rc} + 0x1EA2E0]",
            ".byte 0xD8, 0xCC",
            "fld dword ptr [{rc} + 0x1EA2F0]",
            ".byte 0xD8, 0xCB",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA300]",
            ".byte 0xD9, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2D4]",
            ".byte 0xDE, 0xCB",
            "fld dword ptr [{rc} + 0x1EA2E4]",
            ".byte 0xDE, 0xC9",
            "fld dword ptr [{rc} + 0x1EA2F4]",
            ".byte 0xDE, 0xCA",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{rc} + 0x1EA304]",
            ".byte 0xD9, 0xCA",
            "fstp dword ptr [{mk} + 0x186995]",
            "fstp dword ptr [{mk} + 0x186999]",
            "fstp dword ptr [{mk} + 0x18699D]",
            rc = in(reg) rc,
            mk = in(reg) mk,
            e = in(reg) e,
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
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::frame_state::DrawWork;

    fn word(at: usize) -> u32 {
        unsafe { crate::pointers::host_at(at).cast::<u32>().read_unaligned() }
    }

    #[test]
    fn first_frame_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x90,
                    hi: 0x94,
                    reads: &[(0x90, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x30, F32),
                    ],
                },
                Op::Base {
                    lo: 0x94,
                    hi: 0x98,
                    reads: &[(0x94, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::first_frame_x87(b[0], b[1], b[2], b[3]),
            |b| {
                crate::effects::ring_entries::with_entry_at(
                    &crate::effects::owner::effects().ring,
                    b[2],
                    0x10,
                    0x38,
                    |e| {
                        super::first_frame_x87(
                            &crate::effects::owner::effects().ring,
                            e,
                            crate::effects::setup::clock_at(b[1]),
                            f32::from_bits(word(b[3] + 0x94)),
                        );
                    },
                );
            },
        );
    }

    #[test]
    fn move_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x90,
                    hi: 0x94,
                    reads: &[(0x90, I32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x38,
                    reads: &[
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x2C, F32),
                        (0x30, F32),
                        (0x34, F32),
                    ],
                    writes: &[(0x10, F32), (0x14, F32), (0x18, F32), (0x30, F32)],
                },
                Op::Base {
                    lo: 0x98,
                    hi: 0x9C,
                    reads: &[(0x98, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::move_x87(b[0], b[1], b[2]),
            |b| {
                crate::effects::ring_entries::with_entry_at(
                    &crate::effects::owner::effects().ring,
                    b[1],
                    0x10,
                    0x38,
                    |e| {
                        super::move_x87(
                            &crate::effects::owner::effects().ring,
                            e,
                            crate::effects::setup::clock_at(b[0]),
                            f32::from_bits(word(b[2] + 0x98)),
                        );
                    },
                );
            },
        );
    }

    #[test]
    fn turn_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x186A31,
                    reads: &[],
                    writes: &[(0x186995, F32), (0x186999, F32), (0x18699D, F32)],
                },
                Op::Base {
                    lo: 0x10,
                    hi: 0x1C,
                    reads: &[(0x10, F32), (0x14, F32), (0x18, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::turn_place_x87(b[0], b[1], b[2]),
            |b| {
                with_copy::<DrawWork, _>(b[1], DrawWork::START, |dw| {
                    crate::effects::ring_entries::with_entry_at(
                        &crate::effects::owner::effects().ring,
                        b[2],
                        0x10,
                        0x1C,
                        |e| {
                            super::turn_place_x87(&crate::effects::owner::effects().ring, e, dw);
                        },
                    );
                });
            },
        );
    }
}

#[cfg(test)]
mod colours_tests {
    use super::*;

    #[test]
    fn colours_start_as_the_games() {
        assert_eq!(size_of::<Colours257>(), Colours257::LEN);
        let p = (&raw const crate::effects::owner::EFFECTS.ring.colours).cast::<u8>();
        let own: Vec<u8> = (0..Colours257::LEN)
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(own == crate::moved::image_bytes(Colours257::START, Colours257::LEN));
    }
}
