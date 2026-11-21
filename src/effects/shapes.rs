//! Two of `effects::load_edges`'s steps: the shape tables of two kinds of marks' items, named here
//! by their shapes (what draws them is not read yet). Each clears its 0x800 items, makes a
//! quarter-circle curve of 0x100 floats and 8 circles of points (by the radii after the curve), and
//! sets up what follows.

use crate::effects::windscreen_drop_state_store::WindscreenDropWork;
use crate::effects::windscreen_drop_vars_store::{WindscreenDropVars, WindscreenDrops};
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;

/// The windscreen drops' tables: the items (`ITEMS` of `ITEM_LEN` bytes), the curve, the radii
/// and the circles' points (three floats a point, `SIDES` points a circle).
pub struct Shapes<'a> {
    hex: &'a mut WindscreenDropVars,
    work: &'a mut WindscreenDropWork,
}

pub const ITEMS: u32 = 0x800;
const ITEM_LEN: usize = 0x30;
const SIDES: u32 = 6;
const CURVE: u32 = 0x100;
const RADII: u32 = 8;

/// The squares' side of `Shapes`: the same steps over the squares' typed
/// variables (`effects::windscreen_flakes_store`), four points to a circle.
mod squares {
    use super::{CURVE, RADII};
    use crate::effects::windscreen_flakes_store::WindscreenFlakes;

    use crate::math::x87;

    pub const SIDES: usize = 4;

    /// The items cleared.
    fn clear_items(fl: &WindscreenFlakes) {
        for item in &mut fl.items.borrow_mut().items {
            item.live = 0;
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

const FANS: u32 = 0x40;

impl Shapes<'_> {
    /// The four floats `set_up_windscreen_drops` scales once, by one or two factors each.
    fn scale_once(&mut self, plain: bool) {
        let hex = &mut *self.hex;
        if hex.done != 0 {
            return;
        }
        let factors: [&[f32]; 4] = [
            &[hex.factors[0]],
            &[hex.factors[1]],
            &[hex.factors[2], hex.factors[3]],
            &[hex.factors[4], hex.factors[5]],
        ];
        for (n, by) in factors.into_iter().enumerate() {
            let v = hex.scale[n];
            hex.scale[n] = if plain {
                rc2k_core::effects::shapes::scaled(v, by)
            } else {
                match by {
                    [b] => x87::mul(v, *b),
                    [b, c] => x87::mul2(v, *b, *c),
                    _ => unreachable!(),
                }
            };
        }
        hex.done = 1;
    }

    /// The items' first dwords cleared (each item's live mark).
    fn clear_items(&mut self) {
        for n in 0..ITEMS as usize {
            self.hex.set_item(n * ITEM_LEN, 0);
        }
    }

    /// Circle `n`'s points.
    fn circle(&mut self, n: usize) -> &mut [f32] {
        let len = 3 * SIDES as usize;
        &mut self.work.points[len * n..len * (n + 1)]
    }

    /// The items cleared, the curve, the circles (plain floats: `rc2k_core::effects::shapes`).
    fn make_plain(&mut self) {
        use rc2k_core::effects::shapes as core;
        self.clear_items();
        let curve = core::curve(self.hex.curve_a, self.hex.curve_b);
        for (cell, v) in self.work.curve.iter_mut().zip(curve) {
            *cell = v;
        }
        let (c, d) = (self.hex.step_c, self.hex.step_d);
        for n in 0..RADII as usize {
            let r = self.work.radii[n];
            let to = self.circle(n);
            for (k, [x, y]) in core::circle(r, SIDES, c, d).into_iter().enumerate() {
                to[3 * k] = x;
                to[3 * k + 1] = y;
            }
        }
    }

    /// The items cleared, the curve, the circles.
    fn make(&mut self) {
        self.clear_items();
        let (a, b) = (self.hex.curve_a, self.hex.curve_b);
        for (n, cell) in self.work.curve.iter_mut().enumerate().take(CURVE as usize) {
            let i = 8 * n as i32;
            *cell = x87::circle_curve(i, a, b, i < 0x400);
        }
        let (c, d) = (self.hex.step_c, self.hex.step_d);
        for n in 0..RADII as usize {
            let r = self.work.radii[n];
            x87::circle_points(r, self.circle(n), SIDES, c, d);
        }
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
        let mut hex = drops.hex.borrow_mut();
        let mut work = drops.work.borrow_mut();
        let mut shapes = Shapes {
            hex: &mut hex,
            work: &mut work,
        };
        shapes.scale_once(plain);
        if plain {
            shapes.make_plain();
        } else {
            shapes.make();
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
        hex.started = 0;
        hex.moved = 0;
        hex.spare = [0; 2];
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
