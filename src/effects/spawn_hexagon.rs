//! `effects::spawn_hexagon` (0x4BC22B): a hexagon item made live: of the 32 items from the
//! cursor `dword_81517E` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x14, +0x18), its kind ecx (+4) and the rest 0; the cursor
//! the item after it. With none free, the cursor moves on 32.
//!
//! In:  eax x, ebx z, ecx the kind
//! Out: every register kept
//!
//! Caller: `effects::pick_hexagon`.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::expire_hexagons::{ITEM_LEN, ITEMS};
use crate::effects::hexagon_state::HEXAGON_WORK;
use crate::effects::hexagon_vars::HEX;
use crate::errors::Refused;
use crate::math::x87;
use crate::pointers::GamePtr;
const WINDOW: u32 = 32;

/// The Rust version (see the header).
pub fn spawn_hexagon(x: u32, z: u32, kind: u32) -> Result<(), Refused> {
    {
        let base = HEX.items.addr();
        let end = ITEMS * ITEM_LEN;
        let mut at = HEXAGON_WORK.cursor.bits();
        for _ in 0..WINDOW {
            let e = base.wrapping_add(at);
            if GamePtr::new(e).read::<u32>("a mark's field")? == 0 {
                let put = |o: u32, v: u32| GamePtr::new(e + o).write::<u32>(v, "a mark's field");
                put(0, 1)?;
                put(0x14, x87::int_to_float(x as i32))?;
                put(0x18, x87::int_to_float(z as i32))?;
                put(4, kind)?;
                for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
                    put(o, 0)?;
                }
                at += ITEM_LEN;
                if at == end {
                    at = 0;
                }
                HEXAGON_WORK.cursor.set_bits(at);
                return Ok(());
            }
            at += ITEM_LEN;
            if at == end {
                at = 0;
            }
        }
        HEXAGON_WORK.cursor.set_bits(at);
    }
    Ok(())
}
