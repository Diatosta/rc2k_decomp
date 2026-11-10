//! `effects::expire_windscreen_drops` (0x4BCB1D): each live windscreen drop item (`g_marksWindscreenDropItems`, 0x800
//! of 0x30 bytes, +0 1 live) ends one time in 0x800 (its own random `dword_82E2B6`, times
//! 0xC5 plus 0x5D, bits 16 .. 26 0), or when its cell's byte in `g_marksMaskA` or
//! `g_marksMaskB` (the cell from its place +0x14 / +0x18, `fistp`ed through `dword_81471C` /
//! `dword_814720`, >> 3) is the low byte of one of the last `dword_81622E` steps' values
//! (`dword_844A10`[(`dword_84460C` + k) & 0xFF], the latest first).
//!
//! Caller: `effects::windscreen_drops_step` (0x4BCAF1).

use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;

pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x30;

// Converted: `rc2k_core::effects::expiry::{cell, windscreen_drop_draw, windscreen_drop_hit}`
pub fn expire_windscreen_drops(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    use rc2k_core::effects::expiry;
    {
        let items = &drops.hex.items;
        for n in 0..ITEMS {
            let e = (n * ITEM_LEN) as usize;
            if items.get_u32(e) != 1 {
                continue;
            }
            let f = |o: usize| f32::from_bits(items.get_u32(e + o));
            let c = expiry::cell(f(0x14), f(0x18));
            drops.state.cell[1].set(c.z as u32);
            drops.state.cell[0].set(c.x as u32);
            let (r, draw) = expiry::windscreen_drop_draw(drops.hex.random.get());
            drops.hex.random.set(r);
            if draw == 0 {
                items.set_u32(e, 0);
            }
            let (hit, _) = expiry::windscreen_drop_hit(
                drops.hex.steps.get() as i32,
                |k| {
                    let at = drops.picks.index.get().wrapping_add(k) & 0xFF;
                    drops.picks.values[at as usize].get()
                },
                |n| [&masks.vars.mask_a, &masks.vars.mask_b][n].get(c.at as usize),
            );
            if hit {
                items.set_u32(e, 0);
            }
        }
    }
    Ok(())
}
