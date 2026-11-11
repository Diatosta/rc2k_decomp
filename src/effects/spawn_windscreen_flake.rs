//! `effects::spawn_windscreen_flake` (0x4C0120): a windscreen flake item made live: of the 32 items from the
//! cursor on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (x, z as floats at +0x20, +0x24), its size from its kind (kind & 7, times a factor plus a
//! base, +0xC), a constant at +0x10, the scene clock (+4), +8 0 (the spare dwords are always 0 and not written); the cursor the item after it. With none free, the cursor
//! moves on 32.
//!
//! Takes: x, z, the kind
//! Returns: the kind & 7 when one was made
//!
//! Caller: `effects::pick_windscreen_flake`.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::expire_windscreen_flakes::{ITEM_LEN, ITEMS};
use crate::effects::item_walk::walk_to_free;
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
        fl.state.tile_offset_k[0].bits(),
        fl.state.tile_offset_k[1].bits(),
    ));
    e.texture_v.set(x87::loaded(&fl.state.tile_offset_k[2]));
    e.born.set(shared.physics_tables.scene_clock.bits());
    e.age.set(0);
    Ok(k)
}
