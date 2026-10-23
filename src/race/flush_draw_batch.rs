//! `race::flush_draw_batch` (0x45FD1D): the scene's draw batch handed to the device: with
//! indices in it (`dword_6D3CEC`, edx), `sub_41ADF5` (the platform's `DrawIndexedPrimitive`:
//! eax the vertex count `dword_6D3CF0`, ebx the vertices `unk_6D3CF4`, ecx the indices
//! `unk_6D7CF4`); a failure (eax not 0) logged as "drawtri" and its code (level 1, each
//! inside `pusha`); then both counts 0.
//!
//! In:  nothing
//! Out: edx = 0 with an empty batch; else the registers as the device leaves them (ebx the
//!      vertices); no flags read
//!
//! Callers: `race::draw_sky` (0x4555E5), `sub_4557AB` (twice), `sub_4559A6`, 0x45FCB4.
//! Its calls through `part`: 3 the device, 4 the log's text (eax), 5 its code (eax); in the
//! game all three through the game (section 15, as `race/frame_flush.rs`). Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose calls go to stand-ins (the
//! device failing or not), against the Rust version, batches empty and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

const INDEX_COUNT_RVA: usize = rva(0x6D3CEC); // dword_6D3CEC
const VERTEX_COUNT_RVA: usize = rva(0x6D3CF0); // dword_6D3CF0
const VERTICES_RVA: usize = rva(0x6D3CF4); // unk_6D3CF4
const INDICES_RVA: usize = rva(0x6D7CF4); // unk_6D7CF4
const DRAWTRI_RVA: usize = rva(0x4DDCA4); // "drawtri"

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn flush_draw_batch(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        r.edx = game_read(INDEX_COUNT_RVA);
        if r.edx != 0 {
            r.ecx = game_addr(INDICES_RVA) as u32;
            r.eax = game_read(VERTEX_COUNT_RVA);
            r.ebx = game_addr(VERTICES_RVA) as u32;
            crate::race::flush_draw_batch::game_part(3, &mut r)?;
            if r.eax != 0 {
                let mut log = Regs {
                    eax: game_addr(DRAWTRI_RVA) as u32,
                    ..r
                };
                crate::race::flush_draw_batch::game_part(4, &mut log)?;
                let mut log = r;
                crate::race::flush_draw_batch::game_part(5, &mut log)?;
            }
        }
        game_write(INDEX_COUNT_RVA, 0u32);
        game_write(VERTEX_COUNT_RVA, 0u32);
    }
    Ok(r)
}

/// In the game: the device and the log through the game (`frame_flush::batch_part`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::race::frame_flush::batch_part(place, r)
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn flush_draw_batch_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { flush_draw_batch(r) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { flush_draw_batch(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
