//! `effects::windscreen_drops_frame`: the windscreen drops' frame: the drift and set-up first, then
//! each live item (with the drift on, its motion eased toward the drift at its cell and its place
//! moved). An item dies off the map (x or z past 0x7FF) or on the edge mask; else its place is
//! turned and its six corners projected into the vertex ring (0xC0 bytes each; the ring wraps back
//! to its start, and the batch is drawn by `d3d::draw_marks_windscreen_drops`, the platform's).
//! Then, with fewer live than the settings' `windscreen_drops_wanted` and the drift on, new drops are spawned.
//!
//! Works on the effects settings.
//!
//! The float steps run on the x87 model (`math::x87_float`), rounding as the control word says.
//! Refuses, before anything, null settings and a live item of a kind past 7.

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars_store::WindscreenDrops;
use crate::effects::windscreen_drops::{ITEM_LEN, ITEMS};
use crate::errors::{BadInput, Refused};
use crate::math::cell_x87::CellX87;
use crate::math::x87;
use crate::race::owner_frame::Frame;
/// The vertex batch's bytes, past the 0x40 records of 0xC0 bytes.
const BUFFER_BYTES: usize = 0x3000;
const KINDS: u32 = 8;

/// An item's fields (byte offsets in the items buffer): live flag at 0, then the shape kind,
/// the place (x, z) and the motion eased toward the drift (x, z).
const ITEM_KIND: u32 = 4;
const ITEM_X: u32 = 0x14;
const ITEM_Z: u32 = 0x18;
const ITEM_MOTION_X: u32 = 0x1C;
const ITEM_MOTION_Z: u32 = 0x20;

/// The dword at byte `e` of the items.
fn get(drops: &WindscreenDrops, e: u32) -> u32 {
    drops.hex.borrow().item(e as usize)
}

fn put(drops: &WindscreenDrops, e: u32, v: u32) {
    drops.hex.borrow_mut().set_item(e as usize, v);
}

/// Why it refuses, if it does.
fn refusal(
    effects: &crate::effects::owner::Effects,
    drops: &WindscreenDrops,
) -> Result<Option<&'static str>, Refused> {
    Ok({
        if effects.settings().is_err() {
            return Ok(Some("no marks settings"));
        }
        let hex = drops.hex.borrow();
        let bad = (0..ITEMS as usize).any(|n| {
            let e = n * ITEM_LEN as usize;
            hex.item(e) == 1 && hex.item(e + ITEM_KIND as usize) >= KINDS
        });
        bad.then_some("a live windscreen drop of a kind past 7")
    })
}

/// The batch's draw (`d3d::draw_marks_windscreen_drops`; its result unread).
pub fn draw_batch(
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    crate::d3d::draw_marks_windscreen_drops::draw_marks_windscreen_drops(
        scene.scratch.game_scratch(),
        gfx,
    );
    Ok(())
}

pub fn windscreen_drops_frame(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    frame: &Frame,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    if let Some(why) = refusal(effects, drops)? {
        return Err(BadInput(why).into());
    }
    {
        if setup.kind_floats.borrow().running != 0 {
            crate::effects::windscreen_drops_drift::windscreen_drops_drift(frame, drops, effects)?;
        }
        crate::effects::windscreen_drops_setup::windscreen_drops_setup(buffer, drops, scene);
        drops.state.borrow_mut().live = 0;
        for n in 0..ITEMS {
            let e = n * ITEM_LEN;
            if get(drops, e) != 1 {
                continue;
            }
            {
                let value = drops.state.borrow().live.wrapping_add(1);
                drops.state.borrow_mut().live = value;
            }
            if setup.kind_floats.borrow().running != 0 {
                let x = x87::to_int(get(drops, e + ITEM_X));
                setup.work.borrow_mut().scratch = x as u32;
                drift_cell_x87(drops, 4 * (((x as u32) >> 3) & 0xFF) as usize);
                drift_move_x87(drops, 4 * get(drops, e + ITEM_KIND) as usize, e as usize);
            }
            let x = x87::to_int(get(drops, e + ITEM_X));
            let z = x87::to_int(get(drops, e + ITEM_Z));
            drops.work.borrow_mut().whole = [x as u32, z as u32];
            if (x as u32 | (z as u32).wrapping_add(z as u32)) & 0x800 != 0 {
                put(drops, e, 0);
                continue;
            }
            let shape = get(drops, e + ITEM_KIND);
            let cell = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            if masks.edge_mask.borrow().mask[cell as usize] != 0 {
                put(drops, e, 0);
                continue;
            }
            sprite_depth_x87(drops);
            // The depth's bits below 4096.0's.
            if !crate::math::x87::bits_at_or_below(4096.0, drops.work.borrow().cell[2]) {
                continue;
            }
            let inverse_depth = sprite_corners_x87(
                scene.scratch.front_end_polygon(),
                drops,
                shape.wrapping_mul(0x48) as usize,
            );
            let buffer_at = drops.hex.borrow().buffer;
            sprite_project_x87(
                scene.scratch.front_end_polygon(),
                drops,
                gfx,
                effects,
                buffer_at as usize,
                inverse_depth,
            );
            let next = buffer_at.wrapping_add(0xC0);
            drops.hex.borrow_mut().buffer = next;
            if next as usize == BUFFER_BYTES {
                drops.hex.borrow_mut().buffer = 0;
                gfx.card.windscreen_drop_batch.index_count.set(0x300);
                gfx.card.windscreen_drop_batch.vertex_count.set(0x180);
                draw_batch(scene, gfx)?;
            }
        }
        let marks = effects.settings()?;
        let live: u32 = drops.state.borrow().live;
        if (live as i32) < marks.windscreen_drops_wanted.get() as i32
            && setup.kind_floats.borrow().running != 0
        {
            crate::effects::windscreen_drops_spawn::windscreen_drops_spawn(drops, effects)?;
        }
    }
    Ok(())
}

use crate::effects::plain::{ld_at, st_at};
use crate::math::x87_float::{self as bh, stf};

/// The drift at the item's cell (`curve[k / 4]`) into `drift`: x from the shape tilt's point,
/// y its second coordinate.
fn drift_cell_x87(drops: &WindscreenDrops, k: usize) {
    let tilt = drops.tilt.borrow();
    let mut work = drops.work.borrow_mut();
    let x = -(bh::ld(work.curve[k / 4]) * bh::ld(tilt.point[2])) + bh::ld(tilt.point[0]);
    work.drift[0] = stf(x);
    let y = bh::ld(tilt.point[1]);
    work.drift[1] = stf(y);
}

/// The item at byte `s` of the items has its motion eased toward the drift (scaled by its
/// kind's radius `radii[k / 4]`) and its place moved by the new motion.
fn drift_move_x87(drops: &WindscreenDrops, k: usize, s: usize) {
    let mut hex = drops.hex.borrow_mut();
    let work = drops.work.borrow();
    let ease = bh::ld(hex.ease);
    for (motion, drift) in [(ITEM_MOTION_X, 0), (ITEM_MOTION_Z, 1)] {
        let toward = bh::ld(work.drift[drift]) * bh::ld(work.radii[k / 4]);
        let at = s + motion as usize;
        let eased = (toward - hex.ld_item(at)) * ease + hex.ld_item(at);
        hex.st_item(at, eased);
    }
    let x = hex.ld_item(s + ITEM_X as usize) + hex.ld_item(s + ITEM_MOTION_X as usize);
    let z = hex.ld_item(s + ITEM_Z as usize) + hex.ld_item(s + ITEM_MOTION_Z as usize);
    hex.st_item(s + ITEM_Z as usize, z);
    hex.st_item(s + ITEM_X as usize, x);
}

/// The dword at `offset` of the record batch, as the x87 loads it.
fn ld_record(offset: usize, gfx: &crate::gfx::owner::Gfx) -> bh::X {
    ld_at(&gfx.card.windscreen_drop_batch.records, offset)
}

/// The item's whole-number place turned by the view into `cell` (and the depth, which is
/// `cell[2]` too). Same order and rounding as the x87.
fn sprite_depth_x87(drops: &WindscreenDrops) {
    let hex = drops.hex.borrow();
    let mut work = drops.work.borrow_mut();
    let x = bh::int(work.whole[0] as i32);
    let z = bh::ld(hex.frame_k) - bh::int(work.whole[1] as i32);
    let z = z * bh::ld(work.shares[1]);
    let x = x * bh::ld(work.shares[0]);
    // Output i: x times the first axes row, z times the second, plus the view's place. The
    // last one adds the products the other way round and is also
    // the depth.
    for i in 0..3 {
        let from_x = x * bh::ld(work.axes[i]);
        let from_z = z * bh::ld(work.axes[3 + i]);
        let sum = if i < 2 {
            from_x + from_z
        } else {
            from_z + from_x
        };
        let out = sum + bh::ld(work.place[i]);
        if i == 2 {
            drops.state.borrow_mut().depth = stf(out);
        }
        work.cell[i] = stf(out);
    }
}

/// The six corners of the kind's shape (its offsets from `scaled[c / 4]`) placed around the
/// item's `cell` into `corners`. Returns the inverse depth the projection takes
/// (`view_scale / cell[2]`).
fn sprite_corners_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    drops: &WindscreenDrops,
    c: usize,
) -> bh::X {
    let mut work = drops.work.borrow_mut();
    let inverse_depth = bh::ld(front.window.view_scale.get()) / bh::ld(work.cell[2]);
    // Axis by axis (x, y, z), each of the six corners.
    for axis in 0..3 {
        for corner in 0..6 {
            let i = 3 * corner + axis;
            let sum = bh::ld(work.cell[axis]) + bh::ld(work.scaled[c / 4 + i]);
            work.corners[i] = stf(sum);
        }
    }
    inverse_depth
}

/// `sprite_project`: the centre and the six corners projected to the screen into the six
/// vertices at `e` of the batch's records (0x20 bytes each; the depth in the third field), `inverse_depth`
/// being `view_scale / depth`. Same order and rounding as the x87.
fn sprite_project_x87(
    front: &crate::scene::polygon_work_store::PolygonWork,
    drops: &WindscreenDrops,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    e: usize,
    inverse_depth: bh::X,
) {
    let hex = drops.hex.borrow();
    let mut work = drops.work.borrow_mut();
    let records = &gfx.card.windscreen_drop_batch.records;
    let centre = |axis: usize| front.window.view_centre[axis].ld();
    work.cell[2] = stf(inverse_depth);
    // The centre.
    let up = -(bh::ld(work.cell[1]) * bh::ld(work.cell[2]));
    let across = bh::ld(work.cell[0]) * bh::ld(work.cell[2]) + centre(0);
    work.cell[0] = stf(across);
    work.cell[1] = stf(up + centre(1));
    // The corners: x and y written both at the vertex's screen-place fields and at its first two fields.
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = bh::ld(work.corners[3 * v]) * bh::ld(work.cell[2]);
        let y = -(bh::ld(work.corners[3 * v + 1]) * bh::ld(work.cell[2]));
        let y = y + centre(1);
        st_at(records, at + 0x1C, y);
        st_at(records, at + 0x4, y);
        let x = x + centre(0);
        st_at(records, at + 0x18, x);
        st_at(records, at, x);
    }
    // The vertices' screen places moved around the centre by the scales.
    let screen_x = bh::ld(work.cell[0]) * bh::ld(hex.scale[0]);
    let screen_y = bh::ld(work.cell[1]) * bh::ld(hex.scale[1]);
    work.screen[0] = stf(screen_x);
    work.screen[1] = stf(screen_y);
    for v in 0..6 {
        let at = e + 0x20 * v;
        let x = (ld_record(at + 0x18, gfx) - bh::ld(work.cell[0])) * bh::ld(hex.scale[2])
            + bh::ld(work.screen[0]);
        st_at(records, at + 0x18, x);
        let y = (ld_record(at + 0x1C, gfx) - bh::ld(work.cell[1])) * bh::ld(hex.scale[3])
            + bh::ld(work.screen[1]);
        st_at(records, at + 0x1C, y);
    }
    // Every vertex's depth.
    let depth = (bh::ld(drops.state.borrow().depth)
        + bh::ld(effects.setup.kind_floats.borrow().kind_float))
        * bh::ld(work.inverse);
    for v in 0..6 {
        st_at(records, e + 0x20 * v + 0x8, depth);
    }
}

#[cfg(test)]
mod tests;
