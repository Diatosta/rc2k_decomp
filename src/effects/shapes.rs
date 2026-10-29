//! Two of `sub_4C0F2E`'s steps (IDA, 2026-10-17): the shape tables of two kinds of marks'
//! items, named here by their shapes (what draws them is not read yet). Each clears its
//! 0x800 items, makes a quarter-circle curve of 0x100 floats and 8 circles of points (by
//! the radii after the curve), and sets up what follows.

use crate::effects::hexagon_state::HEXAGON_WORK;
use crate::effects::hexagon_vars::HEX;
use crate::effects::tuning::MARKS_FLOATS;
use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::hexagon_batch::HEXAGON_BATCH;
use crate::math::x87;
use crate::session::{Bytes, Dword, Float, write_bits};

/// One variable of a kind's tables: a typed cell.
pub enum Var {
    Dword(&'static Dword),
    Float(&'static Float),
}

impl Var {
    fn bits(&self) -> u32 {
        match self {
            Var::Dword(d) => d.get(),
            Var::Float(f) => f.bits(),
        }
    }

    fn set_bits(&self, v: u32) {
        match self {
            Var::Dword(d) => d.set(v),
            Var::Float(f) => f.set_bits(v),
        }
    }
}

/// One kind's tables.
pub struct Shapes {
    pub done: Var,
    pub resume: usize,
    /// The items (`ITEMS` of `item_len` bytes).
    pub items: &'static Bytes<0x18000>,
    pub item_len: u32,
    pub curve: &'static [Float; CURVE as usize],
    pub curve_a: Var,
    pub curve_b: Var,
    pub radii: &'static [Float],
    /// The circles' points, three floats a point, `sides` points a circle.
    pub points: &'static [Float],
    pub sides: u32,
    pub step_c: Var,
    pub step_d: Var,
}

pub const ITEMS: u32 = 0x800;
const CURVE: u32 = 0x100;
const RADII: u32 = 8;

/// `sub_4BAA55`'s.
pub static HEXAGONS: Shapes = Shapes {
    done: Var::Dword(&HEX.done), // g_marksHexagonsDone
    resume: rva(0x4BAAC5),       // loc_4BAAC5
    items: &HEX.items,           // g_marksHexagonItems
    item_len: 0x30,
    curve: &HEXAGON_WORK.curve,        // g_marksHexagonCurve
    curve_a: Var::Float(&HEX.curve_a), // flt_82E25A
    curve_b: Var::Float(&HEX.curve_b), // flt_82E25E
    radii: &HEXAGON_WORK.radii,        // flt_814BC2
    points: &HEXAGON_WORK.points,      // g_marksHexagonPoints
    sides: 6,
    step_c: Var::Dword(&HEX.step_c), // dword_82E262
    step_d: Var::Float(&HEX.step_d), // flt_82E266
};

/// Where `sub_4BF529` goes on once done: `loc_4BF545`.
pub const SQUARES_RESUME: usize = rva(0x4BF545);

/// The squares' side of `Shapes` (`sub_4BF529`'s): the same steps over the squares' typed
/// variables (`effects::squares_store`), four points to a circle.
mod squares {
    use super::{CURVE, RADII};
    use crate::effects::squares_store::{SQUARE_ITEMS, SQUARE_STATE, SQUARE_TABLES};
    use crate::errors::{BadInput, Refused};
    use crate::math::x87;
    use crate::session::write_bits;

    pub const SIDES: usize = 4;

    /// The once-only mark: refuses one neither 0 nor `resume`; true when not yet done.
    pub fn first(resume: u32) -> Result<bool, Refused> {
        use rc2k_core::effects::waves::{Mark, mark};
        match mark(SQUARE_STATE.done.get(), resume) {
            Mark::Elsewhere => {
                Err(BadInput("a shapes' mark neither 0 nor where it goes on").into())
            }
            m => Ok(m == Mark::ToMake),
        }
    }

    /// The items cleared.
    fn clear_items() {
        for item in &SQUARE_ITEMS.items {
            item.live.set(0);
        }
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::effects::shapes`).
    pub fn make_plain() {
        use rc2k_core::effects::shapes as core;
        clear_items();
        let f = |bits: u32| f32::from_bits(bits);
        let curve = core::curve(
            f(SQUARE_STATE.curve_ends[0].bits()),
            f(SQUARE_STATE.curve_ends[1].bits()),
        );
        write_bits(&SQUARE_TABLES.curve, curve.map(f32::to_bits));
        let (c, d): (i32, f32) = (
            SQUARE_STATE.steps.get() as i32,
            f(SQUARE_STATE.step_angle.bits()),
        );
        for n in 0..RADII as usize {
            let r = f(SQUARE_TABLES.radii[n].bits());
            for (k, [x, y]) in core::circle(r, SIDES as u32, c, d).into_iter().enumerate() {
                SQUARE_TABLES.points[n][k][0].set_bits(x.to_bits());
                SQUARE_TABLES.points[n][k][1].set_bits(y.to_bits());
            }
        }
    }

    /// The items cleared, the curve, the circles.
    pub fn make() {
        clear_items();
        let (a, b) = (
            SQUARE_STATE.curve_ends[0].bits(),
            SQUARE_STATE.curve_ends[1].bits(),
        );
        for n in 0..CURVE as usize {
            let i = 8 * n as i32;
            SQUARE_TABLES.curve[n].set_bits(x87::circle_curve(i, a, b, i < 0x400));
        }
        for n in 0..RADII as usize {
            super::circle_cells(
                SQUARE_TABLES.radii[n].bits(),
                SQUARE_TABLES.points[n].as_flattened(),
                SIDES as u32,
                SQUARE_STATE.steps.get(),
                SQUARE_STATE.step_angle.bits(),
            );
        }
    }
}

/// The four floats `set_up_hexagons` scales once, by one or two factors each.
static SCALED: [(&Float, &[&Float]); 4] = [
    (&HEX.scale[0], &[&HEX.factors[0]]),
    (&HEX.scale[1], &[&HEX.factors[1]]),
    (&HEX.scale[2], &[&HEX.factors[2], &HEX.factors[3]]),
    (&HEX.scale[3], &[&HEX.factors[4], &HEX.factors[5]]),
];
const FANS: u32 = 0x40;

impl Shapes {
    /// The once-only mark: refuses one neither 0 nor `resume`; true when not yet done.
    fn first(&self, resume: u32) -> Result<bool, Refused> {
        use rc2k_core::effects::waves::{Mark, mark};
        match mark(self.done.bits(), resume) {
            Mark::Elsewhere => {
                Err(BadInput("a shapes' mark neither 0 nor where it goes on").into())
            }
            m => Ok(m == Mark::ToMake),
        }
    }

    /// The items' first dwords cleared (each item's live mark).
    fn clear_items(&self) -> Result<(), Refused> {
        for n in 0..ITEMS {
            let at = (n * self.item_len) as usize;
            self.items.write_at(at, &[0; 4], "a mark item")?;
        }
        Ok(())
    }

    /// Circle `n`'s points.
    fn circle(&self, n: usize) -> &'static [Float] {
        let len = 3 * self.sides as usize;
        &self.points[len * n..len * (n + 1)]
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::effects::shapes`).
    fn make_plain(&self) -> Result<(), Refused> {
        use rc2k_core::effects::shapes as core;
        self.clear_items()?;
        let f = |r: &Var| f32::from_bits(r.bits());
        let curve = core::curve(f(&self.curve_a), f(&self.curve_b));
        write_bits(self.curve, curve.map(f32::to_bits));
        let (c, d): (i32, f32) = (self.step_c.bits() as i32, f(&self.step_d));
        for n in 0..RADII as usize {
            let r = f32::from_bits(self.radii[n].bits());
            let to = self.circle(n);
            for (k, [x, y]) in core::circle(r, self.sides, c, d).into_iter().enumerate() {
                to[3 * k].set_bits(x.to_bits());
                to[3 * k + 1].set_bits(y.to_bits());
            }
        }
        Ok(())
    }

    /// The items cleared, the curve, the circles.
    fn make(&self) -> Result<(), Refused> {
        self.clear_items()?;
        let (a, b): (u32, u32) = (self.curve_a.bits(), self.curve_b.bits());
        for n in 0..CURVE as usize {
            let i = 8 * n as i32;
            self.curve[n].set_bits(x87::circle_curve(i, a, b, i < 0x400));
        }
        for n in 0..RADII as usize {
            circle_cells(
                self.radii[n].bits(),
                self.circle(n),
                self.sides,
                self.step_c.bits(),
                self.step_d.bits(),
            );
        }
        Ok(())
    }
}

/// `x87::circle_points` on cells: radius `r` (bits), `out` three floats a point (the
/// third left alone), the angle's step `pi / c * d`.
fn circle_cells(r: u32, out: &[Float], count: u32, c: u32, d: u32) {
    let mut buf = [0u32; 3 * 6];
    let buf = &mut buf[..out.len()];
    for (b, f) in buf.iter_mut().zip(out) {
        *b = f.bits();
    }
    let c = c as i32;
    x87::circle_points(&r, buf.as_mut_ptr(), count, &c, &d);
    for (f, &b) in out.iter().zip(buf.iter()) {
        f.set_bits(b);
    }
}

/// `effects::set_up_hexagons` (0x4BAA55): once (`g_marksHexagonsDone`, then `resume`) four
/// floats at 0x816192 scaled; then the hexagon items cleared, their curve and circles (6
/// points); 0x40 fans of four triangles (words: n6, n6+1 .. n6+5); 0x40 records of six
/// parts (+0xC `dword_7651A0`, +0x10 -1, +0x14 0xFF000000, 0x20 apart); `dword_8161AE` ..
/// `dword_8161BA` 0. Refuses a mark neither 0 nor `resume`.
fn hexagons_with(resume: u32, plain: bool) -> Result<(), Refused> {
    {
        if HEXAGONS.first(resume)? {
            for (to, by) in SCALED.iter() {
                let v: u32 = to.bits();
                let v = if plain {
                    let by: Vec<f32> = by.iter().map(|b| f32::from_bits(b.bits())).collect();
                    rc2k_core::effects::shapes::scaled(f32::from_bits(v), &by).to_bits()
                } else {
                    match by {
                        [b] => x87::mul(v, b.bits()),
                        [b, c] => x87::mul2(v, b.bits(), c.bits()),
                        _ => unreachable!(),
                    }
                };
                to.set_bits(v);
            }
            HEXAGONS.done.set_bits(resume);
        }
        if plain {
            HEXAGONS.make_plain()?;
        } else {
            HEXAGONS.make()?;
        }
        let mut at = 0usize;
        for n in 0..FANS {
            for triangle in rc2k_core::effects::shapes::fan(n) {
                for (k, v) in triangle.into_iter().enumerate() {
                    HEXAGON_BATCH
                        .fans
                        .write_at(at + 2 * k, &v.to_le_bytes(), "a mark's field")?;
                }
                at += 6;
            }
        }
        let value: u32 = MARKS_FLOATS.two_fifty_six.bits();
        for n in 0..FANS as usize {
            for part in 0..6 {
                let p = 0xC0 * n + 0x20 * part;
                for (o, v) in [(0xC, value), (0x10, u32::MAX), (0x14, 0xFF00_0000)] {
                    HEXAGON_BATCH
                        .records
                        .write_at(p + o, &v.to_le_bytes(), "a mark's field")?;
                }
            }
        }
        HEX.started.set(0);
        HEX.moved.set(0);
        HEX.spare[0].set(0);
        HEX.spare[1].set(0);
    }
    Ok(())
}

/// `effects::set_up_hexagons`' x87 version, for `effects::load_edges`.
pub fn set_up_hexagons(resume: u32) -> Result<(), Refused> {
    hexagons_with(resume, false)
}

/// `effects::set_up_squares` (0x4BF529): its once-only mark set (`g_marksSquaresDone`, then
/// `resume`; nothing else is done once); the square items cleared, their curve and circles
/// (4 points); then `effects::reset_pong`. Refuses a mark neither 0 nor `resume`.
fn squares_with(resume: u32, plain: bool) -> Result<(), Refused> {
    {
        if squares::first(resume)? {
            crate::effects::squares_store::SQUARE_STATE.done.set(resume);
        }
        if plain {
            squares::make_plain();
        } else {
            squares::make();
        }
        crate::effects::splines::reset_pong();
    }
    Ok(())
}

/// `effects::set_up_squares`' x87 version, for `effects::load_edges`.
pub fn set_up_squares(resume: u32) -> Result<(), Refused> {
    squares_with(resume, false)
}
