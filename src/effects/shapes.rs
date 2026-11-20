//! Two of `effects::load_edges`'s steps: the shape tables of two kinds of marks' items, named here
//! by their shapes (what draws them is not read yet). Each clears its 0x800 items, makes a
//! quarter-circle curve of 0x100 floats and 8 circles of points (by the radii after the curve), and
//! sets up what follows.

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;
use crate::memory::session::{PlainBytes, PlainFlag, PlainFloat, PlainInt};

/// One kind's tables.
pub struct Shapes<'a> {
    /// Set once the set-up's once-only part is done.
    pub done: &'a PlainFlag,
    /// The items (`ITEMS` of `item_len` bytes).
    pub items: &'a PlainBytes<0x18000>,
    pub item_len: u32,
    pub curve: &'a [PlainFloat; CURVE as usize],
    pub curve_a: &'a PlainFloat,
    pub curve_b: &'a PlainFloat,
    pub radii: &'a [PlainFloat],
    /// The circles' points, three floats a point, `sides` points a circle.
    pub points: &'a [PlainFloat],
    pub sides: u32,
    /// The circles' angle step is `pi / step_c * step_d`.
    pub step_c: &'a PlainInt,
    pub step_d: &'a PlainFloat,
}

pub const ITEMS: u32 = 0x800;
const CURVE: u32 = 0x100;
const RADII: u32 = 8;

/// `effects::set_up_windscreen_drops`'s, over the group's stores.
impl<'a> Shapes<'a> {
    pub fn windscreen_drops(drops: &'a WindscreenDrops) -> Self {
        Shapes {
            done: &drops.hex.done,
            items: &drops.hex.items,
            item_len: 0x30,
            curve: &drops.work.curve,
            curve_a: &drops.hex.curve_a,
            curve_b: &drops.hex.curve_b,
            radii: &drops.work.radii,
            points: &drops.work.points,
            sides: 6,
            step_c: &drops.hex.step_c,
            step_d: &drops.hex.step_d,
        }
    }
}

/// The squares' side of `Shapes`: the same steps over the squares' typed
/// variables (`effects::windscreen_flakes_store`), four points to a circle.
mod squares {
    use super::{CURVE, RADII};
    use crate::effects::windscreen_flakes_store::WindscreenFlakes;

    use crate::math::x87;

    pub const SIDES: usize = 4;

    /// The items cleared.
    fn clear_items(fl: &WindscreenFlakes) {
        for item in &fl.items.items {
            item.live.set(0);
        }
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::effects::shapes`).
    pub fn make_plain(fl: &WindscreenFlakes) {
        use rc2k_core::effects::shapes as core;
        clear_items(fl);
        let (ends, steps, angle) = {
            let state = fl.state.borrow();
            (state.curve_ends, state.steps as i32, state.step_angle)
        };
        let curve = core::curve(ends[0], ends[1]);
        let mut tables = fl.tables.borrow_mut();
        tables.curve = curve;
        for n in 0..RADII as usize {
            let r = tables.radii[n];
            for (k, [x, y]) in core::circle(r, SIDES as u32, steps, angle)
                .into_iter()
                .enumerate()
            {
                tables.points[n][k][0] = x;
                tables.points[n][k][1] = y;
            }
        }
    }

    /// The items cleared, the curve, the circles.
    pub fn make(fl: &WindscreenFlakes) {
        clear_items(fl);
        let (ends, steps, angle) = {
            let state = fl.state.borrow();
            (state.curve_ends, state.steps as i32, state.step_angle)
        };
        let mut tables = fl.tables.borrow_mut();
        for (n, cell) in tables.curve.iter_mut().enumerate().take(CURVE as usize) {
            let i = 8 * n as i32;
            *cell = x87::circle_curve(i, ends[0], ends[1], i < 0x400);
        }
        for n in 0..RADII as usize {
            let r = tables.radii[n];
            x87::circle_points(
                r,
                tables.points[n].as_flattened_mut(),
                SIDES as u32,
                steps,
                angle,
            );
        }
    }
}

/// The four floats `set_up_windscreen_drops` scales once, by one or two factors each.
fn scaled(drops: &WindscreenDrops) -> [(&PlainFloat, Vec<&PlainFloat>); 4] {
    let hex = &drops.hex;
    [
        (&hex.scale[0], vec![&hex.factors[0]]),
        (&hex.scale[1], vec![&hex.factors[1]]),
        (&hex.scale[2], vec![&hex.factors[2], &hex.factors[3]]),
        (&hex.scale[3], vec![&hex.factors[4], &hex.factors[5]]),
    ]
}
const FANS: u32 = 0x40;

impl<'a> Shapes<'a> {
    /// The items' first dwords cleared (each item's live mark).
    fn clear_items(&self) -> Result<(), Refused> {
        for n in 0..ITEMS {
            let at = (n * self.item_len) as usize;
            self.items.write_at(at, &[0; 4], "a mark item")?;
        }
        Ok(())
    }

    /// Circle `n`'s points.
    fn circle(&self, n: usize) -> &'a [PlainFloat] {
        let len = 3 * self.sides as usize;
        &self.points[len * n..len * (n + 1)]
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::effects::shapes`).
    fn make_plain(&self) -> Result<(), Refused> {
        use rc2k_core::effects::shapes as core;
        self.clear_items()?;
        let curve = core::curve(self.curve_a.get(), self.curve_b.get());
        for (cell, v) in self.curve.iter().zip(curve) {
            cell.set(v);
        }
        let (c, d): (i32, f32) = (self.step_c.get(), self.step_d.get());
        for n in 0..RADII as usize {
            let r = self.radii[n].get();
            let to = self.circle(n);
            for (k, [x, y]) in core::circle(r, self.sides, c, d).into_iter().enumerate() {
                to[3 * k].set(x);
                to[3 * k + 1].set(y);
            }
        }
        Ok(())
    }

    /// The items cleared, the curve, the circles.
    fn make(&self) -> Result<(), Refused> {
        self.clear_items()?;
        let (a, b) = (self.curve_a.get(), self.curve_b.get());
        for (n, cell) in self.curve.iter().enumerate().take(CURVE as usize) {
            let i = 8 * n as i32;
            cell.set(x87::circle_curve(i, a, b, i < 0x400));
        }
        for n in 0..RADII as usize {
            circle_cells(
                self.radii[n].get(),
                self.circle(n),
                self.sides,
                self.step_c.get(),
                self.step_d.get(),
            );
        }
        Ok(())
    }
}

/// `x87::circle_points` on cells: radius `r`, `out` three floats a point (the third left
/// alone), the angle's step `pi / c * d`.
fn circle_cells(r: f32, out: &[PlainFloat], count: u32, c: i32, d: f32) {
    let mut buf = [0f32; 3 * 6];
    let buf = &mut buf[..out.len()];
    for (b, f) in buf.iter_mut().zip(out) {
        *b = f.get();
    }
    x87::circle_points(r, buf, count, c, d);
    for (f, &b) in out.iter().zip(buf.iter()) {
        f.set(b);
    }
}

/// `effects::set_up_windscreen_drops`: once (then the done flag is set) four floats scaled;
/// then the windscreen drop items cleared, their curve and circles (6 points); 0x40 fans of
/// four triangles (words: n6, n6+1 .. n6+5); 0x40 records of six parts (0x20 bytes
/// apart: `MarksFloats.two_fifty_six`, -1 and 0xFF000000 in dwords 3, 4 and 5 of each);
/// `WindscreenDropVars.started` ..
/// `WindscreenDropVars.spare[1]` 0.
fn windscreen_drops_with(
    drops: &WindscreenDrops,
    plain: bool,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        let shapes = Shapes::windscreen_drops(drops);
        if !shapes.done.is_set() {
            for (to, by) in scaled(drops).iter() {
                let v = to.get();
                let v = if plain {
                    let by: Vec<f32> = by.iter().map(|b| b.get()).collect();
                    rc2k_core::effects::shapes::scaled(v, &by)
                } else {
                    match by.as_slice() {
                        [b] => x87::mul(v, *b),
                        [b, c] => x87::mul2(v, *b, *c),
                        _ => unreachable!(),
                    }
                };
                to.set(v);
            }
            shapes.done.set(true);
        }
        if plain {
            shapes.make_plain()?;
        } else {
            shapes.make()?;
        }
        let mut at = 0usize;
        for n in 0..FANS {
            for triangle in rc2k_core::effects::shapes::fan(n) {
                for (k, v) in triangle.into_iter().enumerate() {
                    gfx.card.windscreen_drop_batch.fans.write_at(
                        at + 2 * k,
                        &v.to_le_bytes(),
                        "a mark's field",
                    )?;
                }
                at += 6;
            }
        }
        let value = effects.setup.floats.two_fifty_six.to_bits();
        for n in 0..FANS as usize {
            for part in 0..6 {
                let p = 0xC0 * n + 0x20 * part;
                for (o, v) in [(0xC, value), (0x10, u32::MAX), (0x14, 0xFF00_0000)] {
                    gfx.card.windscreen_drop_batch.records.write_at(
                        p + o,
                        &v.to_le_bytes(),
                        "a mark's field",
                    )?;
                }
            }
        }
        drops.hex.started.set(0);
        drops.hex.moved.set(0);
        drops.hex.spare[0].set(0);
        drops.hex.spare[1].set(0);
    }
    Ok(())
}

/// `effects::set_up_windscreen_drops`' x87 version, for `effects::load_edges`.
pub(super) fn set_up_windscreen_drops(
    drops: &WindscreenDrops,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    windscreen_drops_with(drops, false, gfx, effects)
}

/// `effects::set_up_windscreen_flakes`: the square items cleared, their curve and circles
/// (4 points); then `effects::reset_pong`.
fn windscreen_flakes_with(
    pong: &crate::effects::pong_store::Pong,
    fl: &WindscreenFlakes,
    plain: bool,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    {
        if plain {
            squares::make_plain(fl);
        } else {
            squares::make(fl);
        }
        crate::effects::splines::reset_pong(pong, effects);
    }
    Ok(())
}

/// `effects::set_up_windscreen_flakes`' x87 version, for `effects::load_edges`.
pub(super) fn set_up_windscreen_flakes(
    pong: &crate::effects::pong_store::Pong,
    fl: &WindscreenFlakes,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    windscreen_flakes_with(pong, fl, false, effects)
}
