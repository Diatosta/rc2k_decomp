//! `hud::flush_batch` (0x4B74EF): unless the batch holds no indices, drawn by the device
//! (`sub_45A869`: `DrawIndexedPrimitive` through `g_d3dDevice`, the platform's) and
//! emptied.
//!
//! In:  nothing
//! Out: as the device's call leaves the registers when drawn; else kept
//!
//! Callers: `sub_4B1CD0` (the HUD's end), `hud::add_polygon`. The device through `part`
//! (1), in the game the original's call (section 15). Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose device call goes to a stand-in, against the Rust
//! version, the batch empty and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::batch::{INDEX_COUNT_RVA, VERTEX_COUNT_RVA};

const DEVICE_CALL_RVA: usize = rva(0x45A869); // sub_45A869

/// A call by place: 1 the device's draw.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::flush_due`
pub unsafe fn flush_batch(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if !rc2k_core::hud::batch::flush_due(game_read(INDEX_COUNT_RVA)) {
            return Ok(g);
        }
        part(1, &mut g)?;
        game_write(VERTEX_COUNT_RVA, 0u32);
        game_write(INDEX_COUNT_RVA, 0u32);
    }
    Ok(g)
}

/// In the game: the device's draw through the original's call.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(DEVICE_CALL_RVA, r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { flush_batch(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
