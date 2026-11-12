//! The windscreen flakes' items: made live (`spawn_windscreen_flake`), picked from the random
//! (`pick_windscreen_flake`), ended (`expire_windscreen_flakes`) and stepped
//! (`windscreen_flakes_step`, called by `effects::windscreen_flakes_frame`).
//!
//! Spawn: of the 32 items from the cursor on (wrapping at 0x800) the first free one (+0 0)
//! gets +0 1, its place (x, z as floats at +0x20, +0x24), its size from its kind (kind & 7,
//! times a factor plus a base, +0xC), a constant at +0x10 and the scene clock (+4); the
//! cursor the item after it. With none free, the cursor moves on 32. Returns the kind & 7.
//!
//! Pick: one step of the flakes' own random (times 0xC5 plus 0x5D) split into a place (bits
//! 0 .. 10, 11 .. 20) and a kind (bits 21 .. 23).
//!
//! Expire: each live item (0x800 of 0x3C bytes, +0 1 live) ends when its cell's byte in
//! mask A (or, with the mask run count above 1, in mask B) is the low byte of the first
//! shape's ease step (the cell from its place +0x20 / +0x24, rounded, >> 3).
//!
//! Step: a frame switch 0; for each step since the flakes were last looked at (their
//! last-seen clock less the picks' clock, low byte) `effects::pick_shapes` and expire, the
//! picks' clock on one (low byte); then the last-seen clock set to it. Refuses
//! `pick_shapes`' refusals.

use crate::effects::item_walk::walk_to_free;
use crate::effects::mask_store::Masks;
use crate::effects::pick_shapes::pick_shapes;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::math::x87;
use crate::race::owner_shared::RaceShared;

/// Ecx as it leaves it.
pub fn spawn_windscreen_flake(
    shared: &RaceShared,
    fl: &WindscreenFlakes,
    x: u32,
    z: u32,
    kind: u32,
) -> Result<u32, Refused> {
    let walk = walk_to_free(
        fl.tables.cursor.get(),
        ITEMS,
        ITEM_LEN,
        |at| fl.items.items[(at / ITEM_LEN) as usize].live.get() == 0,
        "a windscreen flake cursor off the items",
    )?;
    fl.tables.cursor.set(walk.cursor);
    let Some(at) = walk.free else {
        return Ok(kind);
    };
    let e = &fl.items.items[(at / ITEM_LEN) as usize];
    e.live.set(1);
    e.place[0].set(x87::from_int(x as i32));
    e.place[1].set(x87::from_int(z as i32));
    let k = kind & 7;
    e.texture_u.set(x87::int_times_plus(
        k as i32,
        &fl.state.tile_offset_k[0],
        &fl.state.tile_offset_k[1],
    ));
    e.texture_v.set(x87::loaded(&fl.state.tile_offset_k[2]));
    e.born.set(shared.physics_tables.scene_clock.get());
    e.age.set(0);
    Ok(k)
}

pub fn pick_windscreen_flake(shared: &RaceShared, fl: &WindscreenFlakes) -> Result<(), Refused> {
    {
        let s = fl
            .state
            .pick_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        fl.state.pick_random.set(s);
        spawn_windscreen_flake(shared, fl, s & 0x7FF, (s >> 11) & 0x3FF, (s >> 21) & 7)?;
    }
    Ok(())
}

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x3C;

// Converted: `rc2k_core::effects::expiry::{cell, square_ends}`
pub fn expire_windscreen_flakes(masks: &Masks, drops: &WindscreenDrops, fl: &WindscreenFlakes) {
    use rc2k_core::effects::expiry;

    let al = masks.vars.shapes[0].ease_step.get() as u8;
    for item in &fl.items.items {
        if item.live.get() != 1 {
            continue;
        }
        let c = expiry::cell(item.place[0].get(), item.place[1].get());
        drops.state.cell[1].set(c.z as u32);
        drops.state.cell[0].set(c.x as u32);
        if expiry::square_ends(
            al,
            || masks.vars.run_count.get() as i32,
            |n| [&masks.vars.mask_a, &masks.vars.mask_b][n].get(c.at as usize),
        ) {
            item.live.set(0);
        }
    }
}

// Converted: `rc2k_core::effects::steps::{steps, next_step}`
pub fn windscreen_flakes_step(
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
) -> Result<(), Refused> {
    use rc2k_core::effects::steps;
    {
        drops.hex.windscreen_flakes_cleared.set(0);
        let clock: u32 = drops.state.clock.get();
        let n = steps::steps(drops.picks.index.get(), clock);
        drops.picks.index.set(clock);
        for _ in 0..n {
            pick_shapes(masks, drops)?;
            expire_windscreen_flakes(masks, drops, fl);
            drops
                .picks
                .index
                .set(steps::next_step(drops.picks.index.get()));
        }
        drops.state.clock.set(drops.picks.index.get());
    }
    Ok(())
}
