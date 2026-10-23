//! Two of `sub_4C0F2E`'s steps (IDA, 2026-10-17): the shape tables of two kinds of marks'
//! items, named here by their shapes (what draws them is not read yet). Each clears its
//! 0x800 items, makes a quarter-circle curve of 0x100 floats and 8 circles of points (by
//! the radii after the curve), and sets up what follows.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

/// One kind's tables.
pub struct Shapes {
    pub done: usize,
    pub resume: usize,
    pub items: usize,
    pub item_len: u32,
    pub curve: usize,
    pub curve_a: usize,
    pub curve_b: usize,
    pub radii: usize,
    pub points: usize,
    pub sides: u32,
    pub step_c: usize,
    pub step_d: usize,
}

pub const ITEMS: u32 = 0x800;
const CURVE: u32 = 0x100;
const RADII: u32 = 8;

/// `sub_4BAA55`'s.
pub const HEXAGONS: Shapes = Shapes {
    done: rva(0x82E23E),   // g_marksHexagonsDone
    resume: rva(0x4BAAC5), // loc_4BAAC5
    items: rva(0x81623E),  // g_marksHexagonItems
    item_len: 0x30,
    curve: rva(0x8147C2),   // g_marksHexagonCurve
    curve_a: rva(0x82E25A), // flt_82E25A
    curve_b: rva(0x82E25E), // flt_82E25E
    radii: rva(0x814BC2),   // flt_814BC2
    points: rva(0x814F3E),  // g_marksHexagonPoints
    sides: 6,
    step_c: rva(0x82E262), // dword_82E262
    step_d: rva(0x82E266), // flt_82E266
};

/// `sub_4BF529`'s.
pub const SQUARES: Shapes = Shapes {
    done: rva(0x8BC18A),   // g_marksSquaresDone
    resume: rva(0x4BF545), // loc_4BF545
    items: rva(0x89E18A),  // g_marksSquareItems
    item_len: 0x3C,
    curve: rva(0x89CACA),   // g_marksSquareCurve
    curve_a: rva(0x8BC18E), // flt_8BC18E
    curve_b: rva(0x8BC192), // flt_8BC192
    radii: rva(0x89CECA),   // flt_89CECA
    points: rva(0x89CFE6),  // g_marksSquarePoints
    sides: 4,
    step_c: rva(0x8BC196), // dword_8BC196
    step_d: rva(0x8BC19A), // flt_8BC19A
};

/// The four floats `set_up_hexagons` scales once, by one or two factors each.
const SCALED: [(usize, &[usize]); 4] = [
    (0x816192, &[0x82E242]),
    (0x816196, &[0x82E246]),
    (0x81619A, &[0x82E24A, 0x82E24E]),
    (0x81619E, &[0x82E252, 0x82E256]),
];
const FANS_RVA: usize = rva(0x624240); // g_marksHexagonFans: 0x40 of 4 triangles (words)
const FANS: u32 = 0x40;
const FAN_RECORDS_RVA: usize = rva(0x625A44); // 0x40 of 0xC0
const FAN_VALUE_RVA: usize = rva(0x7651A0); // dword_7651A0
const CLEARED_RVA: usize = rva(0x8161AE); // dword_8161AE .. dword_8161BA

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
    /// Leaves esi past the radii, edi past the points, eax 8.
    unsafe fn make_plain(&self, g: &mut Regs) {
        use rc2k_core::marks::shapes as core;
        unsafe {
            let items = game_addr(self.items) as u32;
            for n in 0..ITEMS {
                ((items + n * self.item_len) as *mut u32).write_unaligned(0);
            }
            let f = |r: usize| f32::from_bits(game_read(r));
            let curve = core::curve(f(self.curve_a), f(self.curve_b));
            game_write(self.curve, curve.map(f32::to_bits));
            let (c, d): (i32, f32) = (game_read(self.step_c), f(self.step_d));
            g.esi = game_addr(self.radii) as u32;
            g.edi = game_addr(self.points) as u32;
            for _ in 0..RADII {
                let r = f32::from_bits((g.esi as *const u32).read_unaligned());
                for (k, [x, y]) in core::circle(r, self.sides, c, d).into_iter().enumerate() {
                    let at = g.edi + 12 * k as u32;
                    (at as *mut u32).write_unaligned(x.to_bits());
                    ((at + 4) as *mut u32).write_unaligned(y.to_bits());
                }
                g.edi += 12 * self.sides;
                g.esi += 4;
            }
            g.eax = RADII;
        }
    }

    /// The items cleared, the curve, the circles. Leaves esi past the radii, edi past the
    /// points, eax 8 (the loop's counter read back).
    unsafe fn make(&self, g: &mut Regs) {
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
            g.esi = game_addr(self.radii) as u32;
            g.edi = game_addr(self.points) as u32;
            for _ in 0..RADII {
                x87::circle_points(
                    g.esi as *const u32,
                    g.edi as *mut u32,
                    self.sides,
                    game_addr(self.step_c) as *const i32,
                    game_addr(self.step_d) as *const u32,
                );
                g.edi += 12 * self.sides;
                g.esi += 4;
            }
            g.eax = RADII;
        }
    }
}

/// `marks::set_up_hexagons` (0x4BAA55): once (`g_marksHexagonsDone`, then `resume`) four
/// floats at 0x816192 scaled; then the hexagon items cleared, their curve and circles (6
/// points); 0x40 fans of four triangles (words: n6, n6+1 .. n6+5); 0x40 records of six
/// parts (+0xC `dword_7651A0`, +0x10 -1, +0x14 0xFF000000, 0x20 apart); `dword_8161AE` ..
/// `dword_8161BA` 0. Leaves eax 0x40, ebx 0x17D, esi past the records, edi past the
/// points. Refuses a mark neither 0 nor `resume`.
unsafe fn hexagons_with(entry: &Regs, resume: u32, plain: bool) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if HEXAGONS.first(resume)? {
            for (to, by) in SCALED {
                let v: u32 = game_read(rva(to));
                let v = if plain {
                    let by: Vec<f32> = by
                        .iter()
                        .map(|&b| f32::from_bits(game_read(rva(b))))
                        .collect();
                    rc2k_core::marks::shapes::scaled(f32::from_bits(v), &by).to_bits()
                } else {
                    match by {
                        [b] => x87::mul(v, game_read(rva(*b))),
                        [b, c] => x87::mul2(v, game_read(rva(*b)), game_read(rva(*c))),
                        _ => unreachable!(),
                    }
                };
                game_write(rva(to), v);
            }
            g.eax = game_addr(HEXAGONS.done) as u32;
            game_write(HEXAGONS.done, resume);
        }
        if plain {
            HEXAGONS.make_plain(&mut g);
        } else {
            HEXAGONS.make(&mut g);
        }
        g.esi = game_addr(FANS_RVA) as u32;
        for n in 0..FANS {
            for triangle in rc2k_core::marks::shapes::fan(n) {
                for (k, v) in triangle.into_iter().enumerate() {
                    ((g.esi + 2 * k as u32) as *mut u16).write_unaligned(v);
                }
                g.esi += 6;
            }
        }
        g.esi = game_addr(FAN_RECORDS_RVA) as u32;
        let value: u32 = game_read(FAN_VALUE_RVA);
        for _ in 0..FANS {
            for part in 0..6 {
                let p = g.esi + 0x20 * part;
                for (o, v) in [(0xC, value), (0x10, u32::MAX), (0x14, 0xFF00_0000)] {
                    ((p + o) as *mut u32).write_unaligned(v);
                }
            }
            g.esi += 0xC0;
        }
        g.eax = FANS;
        g.ebx = (FANS - 1) * 6 + 5;
        game_write(CLEARED_RVA, [0u32; 4]);
    }
    Ok(g)
}

/// `marks::set_up_hexagons`' x87 version, for `marks::load_edges` (unconverted).
pub unsafe fn set_up_hexagons(entry: &Regs, resume: u32) -> Result<Regs, Refused> {
    unsafe { hexagons_with(entry, resume, false) }
}

/// The converted `set_up_hexagons` (plain floats): the hook's.
// Converted: `rc2k_core::marks::{waves::mark, shapes}`
pub unsafe fn set_up_hexagons_plain(entry: &Regs, resume: u32) -> Result<Regs, Refused> {
    unsafe { hexagons_with(entry, resume, true) }
}

/// `marks::set_up_squares` (0x4BF529): its once-only mark set (`g_marksSquaresDone`, then
/// `resume`; nothing else is done once); the square items cleared, their curve and circles
/// (4 points); then `marks::reset_pong`. Leaves eax 8, esi past the radii, edi past the
/// points. Refuses a mark neither 0 nor `resume`.
unsafe fn squares_with(entry: &Regs, resume: u32, plain: bool) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if SQUARES.first(resume)? {
            game_write(SQUARES.done, resume);
        }
        if plain {
            SQUARES.make_plain(&mut g);
        } else {
            SQUARES.make(&mut g);
        }
        crate::marks::splines::reset_pong();
    }
    Ok(g)
}

/// `marks::set_up_squares`' x87 version, for `marks::load_edges` (unconverted).
pub unsafe fn set_up_squares(entry: &Regs, resume: u32) -> Result<Regs, Refused> {
    unsafe { squares_with(entry, resume, false) }
}

/// The converted `set_up_squares` (plain floats): the hook's.
// Converted: `rc2k_core::marks::{waves::mark, shapes}`
pub unsafe fn set_up_squares_plain(entry: &Regs, resume: u32) -> Result<Regs, Refused> {
    unsafe { squares_with(entry, resume, true) }
}
