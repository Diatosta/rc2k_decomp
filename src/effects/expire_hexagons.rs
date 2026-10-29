//! `effects::expire_hexagons` (0x4BCB1D): each live hexagon item (`g_marksHexagonItems`, 0x800
//! of 0x30 bytes, +0 1 live) ends one time in 0x800 (its own random `dword_82E2B6`, times
//! 0xC5 plus 0x5D, bits 16 .. 26 0), or when its cell's byte in `g_marksMaskA` or
//! `g_marksMaskB` (the cell from its place +0x14 / +0x18, `fistp`ed through `dword_81471C` /
//! `dword_814720`, >> 3) is the low byte of one of the last `dword_81622E` steps' values
//! (`dword_844A10`[(`dword_84460C` + k) & 0xFF], the latest first).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::hexagons_step` (0x4BCAF1).

use crate::effects::hexagon_state::HEXAGON_STATE;
use crate::effects::hexagon_vars::HEX;
use crate::effects::mask_picks::PICKS;
use crate::effects::mask_vars::MASKS;
use crate::errors::Refused;
use crate::pointers::GamePtr;

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x30;

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::expiry::{cell, hexagon_draw, hexagon_hit}`
pub fn expire_hexagons() -> Result<(), Refused> {
    use rc2k_core::effects::expiry;
    {
        let base = HEX.items.addr();
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if GamePtr::new(e).read::<u32>("a mark's field")? != 1 {
                continue;
            }
            let f = |o: u32| -> Result<_, Refused> {
                Ok(f32::from_bits(
                    GamePtr::new(e + o).read::<u32>("a mark's field")?,
                ))
            };
            let c = expiry::cell(f(0x14)?, f(0x18)?);
            HEXAGON_STATE.cell[1].set(c.z as u32);
            HEXAGON_STATE.cell[0].set(c.x as u32);
            let (r, draw) = expiry::hexagon_draw(HEX.random.get());
            HEX.random.set(r);
            if draw == 0 {
                GamePtr::new(e).write::<u32>(0, "a mark's field")?;
            }
            let (hit, _) = expiry::hexagon_hit(
                HEX.steps.get() as i32,
                |k| {
                    let at = PICKS.index.get().wrapping_add(k) & 0xFF;
                    PICKS.values[at as usize].get()
                },
                |n| [&MASKS.mask_a, &MASKS.mask_b][n].get(c.at as usize),
            );
            if hit {
                GamePtr::new(e).write::<u32>(0, "a mark's field")?;
            }
        }
    }
    Ok(())
}
