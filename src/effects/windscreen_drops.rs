//! The windscreen drops' items: made live (`spawn_windscreen_drop`), picked from the random
//! (`pick_windscreen_drop`), ended (`expire_windscreen_drops`) and stepped
//! (`windscreen_drops_step`, called by `effects::windscreen_drops_top`).
//!
//! Spawn: of the 32 items from the cursor on (wrapping at 0x800) the first free one (live
//! word 0) made live, with its place (x, z as floats), its kind and the rest 0; the
//! cursor the item after it. With none free, the cursor moves on 32.
//!
//! Pick: one step of the marks' random (times 0x9D plus 0x47) split into a place (bits
//! 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24).
//!
//! Expire: each live item (0x800 of 0x30 bytes, live word 1) ends one time in 0x800 (its own
//! random, times 0xC5 plus 0x5D, bits 16 .. 26 0), or when its cell's byte in mask A or mask
//! B (the cell from its place, rounded, >> 3) is the low byte of one of the
//! last few steps' values (the picks' values at (the shape index + k) & 0xFF, the latest
//! first).
//!
//! Step: the steps since the drops were last looked at (the last-seen clock less the picks'
//! clock, low byte) kept, the picks' clock the last-seen clock; expire; then the picks' clock
//! moved on by those steps (low byte) and back into the last-seen clock.

use crate::effects::item_walk::walk_to_free;
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars_store::{WindscreenDropVars, WindscreenDrops};
use crate::errors::Refused;
use crate::math::x87;

pub(super) fn spawn_windscreen_drop(
    drops: &WindscreenDrops,
    x: u32,
    z: u32,
    kind: u32,
) -> Result<(), Refused> {
    let mut hex = drops.hex.borrow_mut();
    let mut work = drops.work.borrow_mut();
    let walk = walk_to_free(
        work.cursor,
        ITEMS,
        ITEM_LEN,
        |at| hex.item(at as usize) == 0,
        "a windscreen drop cursor off the items",
    )?;
    if let Some(at) = walk.free {
        let e = at as usize;
        let mut put = |o: usize, v: u32| hex.set_item(e + o, v);
        put(0, 1);
        put(0x14, x87::from_int(x as i32).to_bits());
        put(0x18, x87::from_int(z as i32).to_bits());
        put(4, kind);
        for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
            put(o, 0);
        }
    }
    work.cursor = walk.cursor;
    Ok(())
}

pub fn pick_windscreen_drop(
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let s = setup
            .work
            .borrow()
            .seed
            .wrapping_mul(0x9D)
            .wrapping_add(0x47);
        setup.work.borrow_mut().seed = s;
        spawn_windscreen_drop(
            drops,
            s & 0x7FF,
            s.rotate_right(11) & 0x3FF,
            s.rotate_right(22) & 7,
        )?;
    }
    Ok(())
}

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x30;

// The logic is in `rc2k_core::effects::expiry::{cell, windscreen_drop_draw, windscreen_drop_hit}`
pub(super) fn expire_windscreen_drops(
    masks: &Masks,
    drops: &WindscreenDrops,
) -> Result<(), Refused> {
    use rc2k_core::effects::expiry;
    {
        let mut hex = drops.hex.borrow_mut();
        for n in 0..ITEMS {
            let e = (n * ITEM_LEN) as usize;
            if hex.item(e) != 1 {
                continue;
            }
            let f = |hex: &WindscreenDropVars, o: usize| f32::from_bits(hex.item(e + o));
            let c = expiry::cell(f(&hex, 0x14), f(&hex, 0x18));
            drops.state.borrow_mut().cell[1] = c.z as u32;
            drops.state.borrow_mut().cell[0] = c.x as u32;
            let (r, draw) = expiry::windscreen_drop_draw(hex.random);
            hex.random = r;
            if draw == 0 {
                hex.set_item(e, 0);
            }
            let (hit, _) = expiry::windscreen_drop_hit(
                hex.steps as i32,
                |k| {
                    let at = drops.picks.borrow().index.wrapping_add(k) & 0xFF;
                    drops.picks.borrow().values[at as usize]
                },
                |n| [&masks.vars.mask_a, &masks.vars.mask_b][n].get(c.at as usize),
            );
            if hit {
                hex.set_item(e, 0);
            }
        }
    }
    Ok(())
}

// The logic is in `rc2k_core::effects::steps::steps`
pub(super) fn windscreen_drops_step(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let clock: u32 = drops.state.borrow().clock;
        let steps = rc2k_core::effects::steps::steps(drops.picks.borrow().index, clock);
        drops.picks.borrow_mut().index = clock;
        drops.hex.borrow_mut().steps = steps;
        expire_windscreen_drops(masks, drops)?;
        let latest = drops
            .picks
            .borrow()
            .index
            .wrapping_add(drops.hex.borrow().steps);
        drops.picks.borrow_mut().index = latest & 0xFF;
        drops.state.borrow_mut().clock = latest & 0xFF;
    }
    Ok(())
}
