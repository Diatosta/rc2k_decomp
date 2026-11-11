//! `effects::spawn_windscreen_drop` (0x4BC22B): a windscreen drop item made live: of the 32 items from the
//! cursor on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (x, z as floats at +0x14, +0x18), its kind (+4) and the rest 0; the cursor
//! the item after it. With none free, the cursor moves on 32.
//!
//! Takes: x, z, the kind
//!
//! Caller: `effects::pick_windscreen_drop`.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::expire_windscreen_drops::{ITEM_LEN, ITEMS};
use crate::effects::item_walk::walk_to_free;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::math::x87;

pub fn spawn_windscreen_drop(
    drops: &WindscreenDrops,
    x: u32,
    z: u32,
    kind: u32,
) -> Result<(), Refused> {
    let items = &drops.hex.items;
    let walk = walk_to_free(
        drops.work.cursor.bits(),
        ITEMS,
        ITEM_LEN,
        |at| items.get_u32(at as usize) == 0,
        "a windscreen drop cursor off the items",
    )?;
    if let Some(at) = walk.free {
        let e = at as usize;
        let put = |o: usize, v: u32| items.set_u32(e + o, v);
        put(0, 1);
        put(0x14, x87::from_int(x as i32));
        put(0x18, x87::from_int(z as i32));
        put(4, kind);
        for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
            put(o, 0);
        }
    }
    drops.work.cursor.set_bits(walk.cursor);
    Ok(())
}
