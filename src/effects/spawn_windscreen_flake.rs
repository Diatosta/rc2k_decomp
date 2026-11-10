//! `effects::spawn_windscreen_flake` (0x4C0120): a windscreen flake item made live: of the 32 items from the
//! cursor `dword_89D166` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x20, +0x24), its size from its kind (ecx & 7, times
//! `flt_8BC23A` plus `flt_8BC23E`, +0xC), `flt_8BC242` at +0x10, the clock (`g_sceneClock`,
//! +4), +8 0 and the rest 0; the cursor the item after it. With none free, the cursor
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
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::race::owner_shared::Shared;

use crate::errors::{BadInput, Refused};
use crate::math::x87;

const WINDOW: u32 = 32;

/// Ecx as it leaves it.
pub fn spawn_windscreen_flake(
    shared: &Shared,
    fl: &WindscreenFlakes,
    x: u32,
    z: u32,
    kind: u32,
) -> Result<u32, Refused> {
    Ok({
        let end = ITEMS * ITEM_LEN;
        let mut at = fl.tables.cursor.get();
        let step = |at: &mut u32| {
            *at += ITEM_LEN;
            if *at == end {
                *at = 0;
            }
        };
        for _ in 0..WINDOW {
            // The cursor steps by whole items and wraps at the end (only its own writes move
            // it); one off that grid would read past the items.
            if !at.is_multiple_of(ITEM_LEN) || at >= end {
                return Err(BadInput("a windscreen flake cursor off the items").into());
            }
            let e = &fl.items.items[(at / ITEM_LEN) as usize];
            if e.live.get() == 0 {
                e.live.set(1);
                e.place[0].set_bits(x87::int_to_float(x as i32));
                e.place[1].set_bits(x87::int_to_float(z as i32));
                let k = kind & 7;
                e.texture_u.set_bits(x87::int_times_plus(
                    k as i32,
                    fl.state.tile_offset_k[0].bits(),
                    fl.state.tile_offset_k[1].bits(),
                ));
                e.texture_v
                    .set_bits(x87::loaded(fl.state.tile_offset_k[2].bits()));
                for d in e.spare_28.iter().chain(&e.spare_14) {
                    d.set(0);
                }
                e.born.set(shared.physics_tables.scene_clock.bits());
                e.age.set(0);
                step(&mut at);
                fl.tables.cursor.set(at);
                return Ok(k);
            }
            step(&mut at);
        }
        fl.tables.cursor.set(at);
        kind
    })
}
