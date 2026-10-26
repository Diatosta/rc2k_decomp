//! `marks::spawn_square` (0x4C0120): a square item made live: of the 32 items from the
//! cursor `dword_89D166` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x20, +0x24), its size from its kind (ecx & 7, times
//! `flt_8BC23A` plus `flt_8BC23E`, +0xC), `flt_8BC242` at +0x10, the clock (`g_sceneClock`,
//! +4), +8 0 and the rest 0; the cursor the item after it. With none free, the cursor
//! moves on 32.
//!
//! In:  eax x, ebx z, ecx the kind
//! Out: ecx the kind & 7 when one was made; the rest kept
//!
//! Caller: `marks::pick_square`.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::errors::Refused;
use crate::marks::expire_squares::{ITEM_LEN, ITEMS};
use crate::marks::squares_store::{SQUARE_ITEMS, SQUARE_STATE, SQUARE_TABLES};
use crate::math::x87;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;

const WINDOW: u32 = 32;

/// The Rust version (see the header): ecx as it leaves it.
pub fn spawn_square(x: u32, z: u32, kind: u32) -> Result<u32, Refused> {
    Ok({
        let base = crate::session::address_of(&SQUARE_ITEMS.items);
        let end = ITEMS * ITEM_LEN;
        let mut at = SQUARE_TABLES.cursor.get();
        let step = |at: &mut u32| {
            *at += ITEM_LEN;
            if *at == end {
                *at = 0;
            }
        };
        for _ in 0..WINDOW {
            let e = base.wrapping_add(at);
            if GamePtr::new(e).read::<u32>("a mark's field")? == 0 {
                let put = |o: u32, v: u32| GamePtr::new(e + o).write::<u32>(v, "a mark's field");
                put(0, 1)?;
                put(0x20, x87::int_to_float(x as i32))?;
                put(0x24, x87::int_to_float(z as i32))?;
                let k = kind & 7;
                put(
                    0xC,
                    x87::int_times_plus(
                        k as i32,
                        SQUARE_STATE.size_k[0].bits(),
                        SQUARE_STATE.size_k[1].bits(),
                    ),
                )?;
                put(0x10, x87::loaded(SQUARE_STATE.size_k[2].bits()))?;
                for o in [0x28, 0x2C, 0x38, 0x14, 0x18, 0x1C, 0x30, 0x34] {
                    put(o, 0)?;
                }
                put(4, PHYSICS.scene_clock.bits())?;
                put(8, 0)?;
                step(&mut at);
                SQUARE_TABLES.cursor.set(at);
                return Ok(k);
            }
            step(&mut at);
        }
        SQUARE_TABLES.cursor.set(at);
        kind
    })
}
