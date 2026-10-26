//! Two of `sub_4C0F2E`'s steps (IDA, 2026-10-17): the shape tables of two kinds of marks'
//! items, named here by their shapes (what draws them is not read yet). Each clears its
//! 0x800 items, makes a quarter-circle curve of 0x100 floats and 8 circles of points (by
//! the radii after the curve), and sets up what follows.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::hexagon_batch::HEXAGON_BATCH;
use crate::marks::hexagon_state::HEXAGON_WORK;
use crate::marks::hexagon_vars::HEX;
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;
use crate::pointers::GamePtr;
use crate::session::{Dword, Float};

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

    fn addr(&self) -> usize {
        match self {
            Var::Dword(d) => d.addr(),
            Var::Float(f) => f.addr(),
        }
    }
}

/// A table of the shape tables: typed (its first cell's address).
#[derive(Clone, Copy)]
pub enum Table {
    Typed(fn() -> usize),
}

impl Table {
    fn addr(self) -> usize {
        match self {
            Table::Typed(first) => first(),
        }
    }
}

/// One kind's tables.
pub struct Shapes {
    pub done: Var,
    pub resume: usize,
    /// The items' address.
    pub items: fn() -> usize,
    pub item_len: u32,
    pub curve: Table,
    pub curve_a: Var,
    pub curve_b: Var,
    pub radii: Table,
    pub points: Table,
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
    items: || HEX.items.addr(),  // g_marksHexagonItems
    item_len: 0x30,
    curve: Table::Typed(|| HEXAGON_WORK.curve[0].addr()), // g_marksHexagonCurve
    curve_a: Var::Float(&HEX.curve_a),                    // flt_82E25A
    curve_b: Var::Float(&HEX.curve_b),                    // flt_82E25E
    radii: Table::Typed(|| HEXAGON_WORK.radii[0].addr()), // flt_814BC2
    points: Table::Typed(|| HEXAGON_WORK.points[0].addr()), // g_marksHexagonPoints
    sides: 6,
    step_c: Var::Dword(&HEX.step_c), // dword_82E262
    step_d: Var::Float(&HEX.step_d), // flt_82E266
};

/// Where `sub_4BF529` goes on once done: `loc_4BF545`.
pub const SQUARES_RESUME: usize = rva(0x4BF545);

/// The squares' side of `Shapes` (`sub_4BF529`'s): the same steps over the squares' typed
/// variables (`marks::squares_store`), four points to a circle.
mod squares {
    use super::{CURVE, RADII};
    use crate::errors::{BadInput, Refused};
    use crate::marks::squares_store::{SQUARE_ITEMS, SQUARE_STATE, SQUARE_TABLES};
    use crate::math::x87;
    use crate::session::{address_of, write_bits};

    pub const SIDES: usize = 4;

    /// The once-only mark: refuses one neither 0 nor `resume`; true when not yet done.
    pub fn first(resume: u32) -> Result<bool, Refused> {
        use rc2k_core::marks::waves::{Mark, mark};
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

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::marks::shapes`).
    pub fn make_plain() {
        use rc2k_core::marks::shapes as core;
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
            unsafe {
                x87::circle_points(
                    SQUARE_TABLES.radii[n].addr() as *const u32,
                    address_of(&SQUARE_TABLES.points[n]) as *mut u32,
                    SIDES as u32,
                    SQUARE_STATE.steps.as_ptr() as *const i32,
                    SQUARE_STATE.step_angle.addr() as *const u32,
                );
            }
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
        use rc2k_core::marks::waves::{Mark, mark};
        match mark(self.done.bits(), resume) {
            Mark::Elsewhere => {
                Err(BadInput("a shapes' mark neither 0 nor where it goes on").into())
            }
            m => Ok(m == Mark::ToMake),
        }
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::marks::shapes`).
    fn make_plain(&self) -> Result<(), Refused> {
        use rc2k_core::marks::shapes as core;
        {
            let items = (self.items)() as u32;
            for n in 0..ITEMS {
                GamePtr::new(items + n * self.item_len).write::<u32>(0, "a mark item")?;
            }
            let f = |r: &Var| f32::from_bits(r.bits());
            let curve = core::curve(f(&self.curve_a), f(&self.curve_b));
            GamePtr::new(self.curve.addr() as u32)
                .write::<[u32; CURVE as usize]>(curve.map(f32::to_bits), "the shapes' curve")?;
            let (c, d): (i32, f32) = (self.step_c.bits() as i32, f(&self.step_d));
            let radii = self.radii.addr() as u32;
            let points = self.points.addr() as u32;
            for n in 0..RADII {
                let r = f32::from_bits(GamePtr::new(radii + 4 * n).read::<u32>("a mark's field")?);
                let to = points + 12 * self.sides * n;
                for (k, [x, y]) in core::circle(r, self.sides, c, d).into_iter().enumerate() {
                    let at = to + 12 * k as u32;
                    GamePtr::new(at).write::<u32>(x.to_bits(), "a mark's field")?;
                    GamePtr::new(at + 4).write::<u32>(y.to_bits(), "a mark's field")?;
                }
            }
        }
        Ok(())
    }

    /// The items cleared, the curve, the circles.
    fn make(&self) -> Result<(), Refused> {
        unsafe {
            let items = (self.items)() as u32;
            for n in 0..ITEMS {
                GamePtr::new(items + n * self.item_len).write::<u32>(0, "a mark item")?;
            }
            let (a, b): (u32, u32) = (self.curve_a.bits(), self.curve_b.bits());
            for n in 0..CURVE {
                let i = 8 * n as i32;
                GamePtr::new((self.curve.addr() + 4 * n as usize) as u32)
                    .write::<u32>(x87::circle_curve(i, a, b, i < 0x400), "the shapes' curve")?;
            }
            let radii = self.radii.addr() as u32;
            let points = self.points.addr() as u32;
            for n in 0..RADII {
                x87::circle_points(
                    (radii + 4 * n) as *const u32,
                    (points + 12 * self.sides * n) as *mut u32,
                    self.sides,
                    self.step_c.addr() as *const i32,
                    self.step_d.addr() as *const u32,
                );
            }
        }
        Ok(())
    }
}

/// `marks::set_up_hexagons` (0x4BAA55): once (`g_marksHexagonsDone`, then `resume`) four
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
                    rc2k_core::marks::shapes::scaled(f32::from_bits(v), &by).to_bits()
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
        let mut at = HEXAGON_BATCH.fans.addr() as u32;
        for n in 0..FANS {
            for triangle in rc2k_core::marks::shapes::fan(n) {
                for (k, v) in triangle.into_iter().enumerate() {
                    GamePtr::new(at + 2 * k as u32).write::<u16>(v, "a mark's field")?;
                }
                at += 6;
            }
        }
        let mut at = HEXAGON_BATCH.records.addr() as u32;
        let value: u32 = MARKS_FLOATS.two_fifty_six.bits();
        for _ in 0..FANS {
            for part in 0..6 {
                let p = at + 0x20 * part;
                for (o, v) in [(0xC, value), (0x10, u32::MAX), (0x14, 0xFF00_0000)] {
                    GamePtr::new(p + o).write::<u32>(v, "a mark's field")?;
                }
            }
            at += 0xC0;
        }
        HEX.started.set(0);
        HEX.moved.set(0);
        HEX.spare[0].set(0);
        HEX.spare[1].set(0);
    }
    Ok(())
}

/// `marks::set_up_hexagons`' x87 version, for `marks::load_edges`.
pub fn set_up_hexagons(resume: u32) -> Result<(), Refused> {
    hexagons_with(resume, false)
}

/// `marks::set_up_squares` (0x4BF529): its once-only mark set (`g_marksSquaresDone`, then
/// `resume`; nothing else is done once); the square items cleared, their curve and circles
/// (4 points); then `marks::reset_pong`. Refuses a mark neither 0 nor `resume`.
fn squares_with(resume: u32, plain: bool) -> Result<(), Refused> {
    {
        if squares::first(resume)? {
            crate::marks::squares_store::SQUARE_STATE.done.set(resume);
        }
        if plain {
            squares::make_plain();
        } else {
            squares::make();
        }
        crate::marks::splines::reset_pong();
    }
    Ok(())
}

/// `marks::set_up_squares`' x87 version, for `marks::load_edges`.
pub fn set_up_squares(resume: u32) -> Result<(), Refused> {
    squares_with(resume, false)
}
