//! `effects::spawn_windscreen_drop` (0x4BC22B): a windscreen drop item made live: of the 32 items from the
//! cursor `dword_81517E` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x14, +0x18), its kind ecx (+4) and the rest 0; the cursor
//! the item after it. With none free, the cursor moves on 32.
//!
//! In:  eax x, ebx z, ecx the kind
//! Out: every register kept
//!
//! Caller: `effects::pick_windscreen_drop`.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::expire_windscreen_drops::{ITEM_LEN, ITEMS};
use crate::effects::windscreen_drop_state::WINDSCREEN_DROP_WORK;
use crate::effects::windscreen_drop_vars::HEX;
use crate::errors::BadInput;
use crate::errors::Refused;
use crate::math::x87;
const WINDOW: u32 = 32;

/// The Rust version (see the header).
pub fn spawn_windscreen_drop(x: u32, z: u32, kind: u32) -> Result<(), Refused> {
    {
        let items = &HEX.items;
        let end = ITEMS * ITEM_LEN;
        let mut at = WINDSCREEN_DROP_WORK.cursor.bits();
        for _ in 0..WINDOW {
            // The cursor steps by whole items and wraps at the end; one off that grid
            // (only its own writes keep it there) would read past the items.
            if !at.is_multiple_of(ITEM_LEN) || at >= end {
                return Err(BadInput("a windscreen drop cursor off the items").into());
            }
            let e = at as usize;
            if items.get_u32(e) == 0 {
                let put = |o: usize, v: u32| items.set_u32(e + o, v);
                put(0, 1);
                put(0x14, x87::int_to_float(x as i32));
                put(0x18, x87::int_to_float(z as i32));
                put(4, kind);
                for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
                    put(o, 0);
                }
                at += ITEM_LEN;
                if at == end {
                    at = 0;
                }
                WINDSCREEN_DROP_WORK.cursor.set_bits(at);
                return Ok(());
            }
            at += ITEM_LEN;
            if at == end {
                at = 0;
            }
        }
        WINDSCREEN_DROP_WORK.cursor.set_bits(at);
    }
    Ok(())
}
