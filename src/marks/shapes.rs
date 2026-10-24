//! Two of `sub_4C0F2E`'s steps (IDA, 2026-10-17): the shape tables of two kinds of marks'
//! items, named here by their shapes (what draws them is not read yet). Each clears its
//! 0x800 items, makes a quarter-circle curve of 0x100 floats and 8 circles of points (by
//! the radii after the curve), and sets up what follows.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::hexagon_batch::Slot as HexSlot;
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;

/// One kind's tables.
pub struct Shapes {
    pub done: Slot,
    pub resume: usize,
    pub items: Slot,
    pub item_len: u32,
    pub curve: Slot,
    pub curve_a: Slot,
    pub curve_b: Slot,
    pub radii: Slot,
    pub points: Slot,
    pub sides: u32,
    pub step_c: Slot,
    pub step_d: Slot,
}

pub const ITEMS: u32 = 0x800;
const CURVE: u32 = 0x100;
const RADII: u32 = 8;

/// `sub_4BAA55`'s.
pub const HEXAGONS: Shapes = Shapes {
    done: Slot::at(0x82E23E),  // g_marksHexagonsDone
    resume: rva(0x4BAAC5),     // loc_4BAAC5
    items: Slot::at(0x81623E), // g_marksHexagonItems
    item_len: 0x30,
    curve: Slot::at(0x8147C2),   // g_marksHexagonCurve
    curve_a: Slot::at(0x82E25A), // flt_82E25A
    curve_b: Slot::at(0x82E25E), // flt_82E25E
    radii: Slot::at(0x814BC2),   // flt_814BC2
    points: Slot::at(0x814F3E),  // g_marksHexagonPoints
    sides: 6,
    step_c: Slot::at(0x82E262), // dword_82E262
    step_d: Slot::at(0x82E266), // flt_82E266
};

/// `sub_4BF529`'s.
pub const SQUARES: Shapes = Shapes {
    done: Slot::at(0x8BC18A),  // g_marksSquaresDone
    resume: rva(0x4BF545),     // loc_4BF545
    items: Slot::at(0x89E18A), // g_marksSquareItems
    item_len: 0x3C,
    curve: Slot::at(0x89CACA),   // g_marksSquareCurve
    curve_a: Slot::at(0x8BC18E), // flt_8BC18E
    curve_b: Slot::at(0x8BC192), // flt_8BC192
    radii: Slot::at(0x89CECA),   // flt_89CECA
    points: Slot::at(0x89CFE6),  // g_marksSquarePoints
    sides: 4,
    step_c: Slot::at(0x8BC196), // dword_8BC196
    step_d: Slot::at(0x8BC19A), // flt_8BC19A
};

/// The four floats `set_up_hexagons` scales once, by one or two factors each.
const SCALED: [(Slot, &[Slot]); 4] = [
    (Slot::at(0x816192), &[Slot::at(0x82E242)]),
    (Slot::at(0x816196), &[Slot::at(0x82E246)]),
    (
        Slot::at(0x81619A),
        &[Slot::at(0x82E24A), Slot::at(0x82E24E)],
    ),
    (
        Slot::at(0x81619E),
        &[Slot::at(0x82E252), Slot::at(0x82E256)],
    ),
];
const FANS_RVA: HexSlot = HexSlot::at(0x624240); // g_marksHexagonFans: 0x40 of 4 triangles (words)
const FANS: u32 = 0x40;
const FAN_RECORDS_RVA: HexSlot = HexSlot::at(0x625A44); // 0x40 of 0xC0
const FAN_VALUE_RVA: Slot = Slot::at(0x7651A0); // dword_7651A0
const CLEARED_RVA: Slot = Slot::at(0x8161AE); // dword_8161AE .. dword_8161BA

impl Shapes {
    /// The once-only mark: refuses one neither 0 nor `resume`; true when not yet done.
    unsafe fn first(&self, resume: u32) -> Result<bool, Refused> {
        use rc2k_core::marks::waves::{Mark, mark};
        match mark(unsafe { game_read(self.done) }, resume) {
            Mark::Elsewhere => {
                Err(BadInput("a shapes' mark neither 0 nor where it goes on").into())
            }
            m => Ok(m == Mark::ToMake),
        }
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::marks::shapes`).
    unsafe fn make_plain(&self) {
        use rc2k_core::marks::shapes as core;
        unsafe {
            let items = game_addr(self.items) as u32;
            for n in 0..ITEMS {
                ((items + n * self.item_len) as *mut u32).write_unaligned(0);
            }
            let f = |r: Slot| f32::from_bits(game_read(r));
            let curve = core::curve(f(self.curve_a), f(self.curve_b));
            game_write(self.curve, curve.map(f32::to_bits));
            let (c, d): (i32, f32) = (game_read(self.step_c), f(self.step_d));
            let radii = game_addr(self.radii) as u32;
            let points = game_addr(self.points) as u32;
            for n in 0..RADII {
                let r = f32::from_bits(((radii + 4 * n) as *const u32).read_unaligned());
                let to = points + 12 * self.sides * n;
                for (k, [x, y]) in core::circle(r, self.sides, c, d).into_iter().enumerate() {
                    let at = to + 12 * k as u32;
                    (at as *mut u32).write_unaligned(x.to_bits());
                    ((at + 4) as *mut u32).write_unaligned(y.to_bits());
                }
            }
        }
    }

    /// The items cleared, the curve, the circles.
    unsafe fn make(&self) {
        unsafe {
            let items = game_addr(self.items) as u32;
            for n in 0..ITEMS {
                ((items + n * self.item_len) as *mut u32).write_unaligned(0);
            }
            let (a, b): (u32, u32) = (game_read(self.curve_a), game_read(self.curve_b));
            for n in 0..CURVE {
                let i = 8 * n as i32;
                game_write(
                    self.curve + 4 * n as usize,
                    x87::circle_curve(i, a, b, i < 0x400),
                );
            }
            let radii = game_addr(self.radii) as u32;
            let points = game_addr(self.points) as u32;
            for n in 0..RADII {
                x87::circle_points(
                    (radii + 4 * n) as *const u32,
                    (points + 12 * self.sides * n) as *mut u32,
                    self.sides,
                    game_addr(self.step_c) as *const i32,
                    game_addr(self.step_d) as *const u32,
                );
            }
        }
    }
}

/// `marks::set_up_hexagons` (0x4BAA55): once (`g_marksHexagonsDone`, then `resume`) four
/// floats at 0x816192 scaled; then the hexagon items cleared, their curve and circles (6
/// points); 0x40 fans of four triangles (words: n6, n6+1 .. n6+5); 0x40 records of six
/// parts (+0xC `dword_7651A0`, +0x10 -1, +0x14 0xFF000000, 0x20 apart); `dword_8161AE` ..
/// `dword_8161BA` 0. Refuses a mark neither 0 nor `resume`.
unsafe fn hexagons_with(resume: u32, plain: bool) -> Result<(), Refused> {
    unsafe {
        if HEXAGONS.first(resume)? {
            for (to, by) in SCALED {
                let v: u32 = game_read(to);
                let v = if plain {
                    let by: Vec<f32> = by.iter().map(|&b| f32::from_bits(game_read(b))).collect();
                    rc2k_core::marks::shapes::scaled(f32::from_bits(v), &by).to_bits()
                } else {
                    match by {
                        [b] => x87::mul(v, game_read(*b)),
                        [b, c] => x87::mul2(v, game_read(*b), game_read(*c)),
                        _ => unreachable!(),
                    }
                };
                game_write(to, v);
            }
            game_write(HEXAGONS.done, resume);
        }
        if plain {
            HEXAGONS.make_plain();
        } else {
            HEXAGONS.make();
        }
        let mut at = game_addr(FANS_RVA) as u32;
        for n in 0..FANS {
            for triangle in rc2k_core::marks::shapes::fan(n) {
                for (k, v) in triangle.into_iter().enumerate() {
                    ((at + 2 * k as u32) as *mut u16).write_unaligned(v);
                }
                at += 6;
            }
        }
        let mut at = game_addr(FAN_RECORDS_RVA) as u32;
        let value: u32 = game_read(FAN_VALUE_RVA);
        for _ in 0..FANS {
            for part in 0..6 {
                let p = at + 0x20 * part;
                for (o, v) in [(0xC, value), (0x10, u32::MAX), (0x14, 0xFF00_0000)] {
                    ((p + o) as *mut u32).write_unaligned(v);
                }
            }
            at += 0xC0;
        }
        game_write(CLEARED_RVA, [0u32; 4]);
    }
    Ok(())
}

/// `marks::set_up_hexagons`' x87 version, for `marks::load_edges`.
pub unsafe fn set_up_hexagons(resume: u32) -> Result<(), Refused> {
    unsafe { hexagons_with(resume, false) }
}

/// `marks::set_up_squares` (0x4BF529): its once-only mark set (`g_marksSquaresDone`, then
/// `resume`; nothing else is done once); the square items cleared, their curve and circles
/// (4 points); then `marks::reset_pong`. Refuses a mark neither 0 nor `resume`.
unsafe fn squares_with(resume: u32, plain: bool) -> Result<(), Refused> {
    unsafe {
        if SQUARES.first(resume)? {
            game_write(SQUARES.done, resume);
        }
        if plain {
            SQUARES.make_plain();
        } else {
            SQUARES.make();
        }
        crate::marks::splines::reset_pong();
    }
    Ok(())
}

/// `marks::set_up_squares`' x87 version, for `marks::load_edges`.
pub unsafe fn set_up_squares(resume: u32) -> Result<(), Refused> {
    unsafe { squares_with(resume, false) }
}
