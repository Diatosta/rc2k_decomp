//! `hud::draw_view_strips` (0x45B7B1): unless `dword_5BEC94`, for each player's view block
//! (`g_viewBlocks`, `+4` for the second when `dword_6238A4` is not 0) whose +0xFA8 is 0 and
//! +0x728 is set: while its flash is on (+0x730 at most 0, or +0x72C modulo +0x730 below
//! half of +0x730), `hud::draw_view_strip` for view `dword_6238A4` (the second's plus 3)
//! with texture `dword_6240BC` 0xC4 (0xC5); then the strips' shared tail (the original
//! jumps to 0x45B9AA).
//!
//! In:  nothing
//! Out: as the path leaves them (esi the last block looked at)
//!
//! Caller: `sub_45A893` (0x45A92E). The strips and the tail through `part` (1 the strip,
//! 2 the tail), in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose strip calls and tail jump go to stand-ins, against the
//! Rust version, blocks by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::draw_view_strip::TEXTURE_RVA;

const OFF_RVA: usize = rva(0x5BEC94); // dword_5BEC94
const BLOCKS_RVA: usize = rva(0x63DBB0); // g_viewBlocks
const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4

/// Whether esi's block's strip is shown (see the header); eax / edx / ebx as the test
/// leaves them.
unsafe fn flashing(g: &mut Regs) -> bool {
    use rc2k_core::hud::strips::{Strip, strip};
    let b = g.esi;
    let s = strip(|o| unsafe { ((b + o) as *const u32).read_unaligned() });
    if let Strip::Flashing(f) = s {
        [g.eax, g.edx, g.ebx] = [f.quotient, f.remainder, f.half];
    }
    s.shown()
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::strips`
pub unsafe fn draw_view_strips(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::hud::strips::{SECOND_VIEW, TEXTURES};
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(OFF_RVA) == 0 {
            g.esi = game_read(BLOCKS_RVA);
            if flashing(&mut g) {
                g.ebp = game_read(PLAYERS_RVA);
                game_write(TEXTURE_RVA, TEXTURES[0]);
                crate::hud::draw_view_strips::game_part(1, &mut g)?;
            }
            g.ebp = game_read(PLAYERS_RVA);
            if g.ebp != 0 {
                g.esi = game_read(BLOCKS_RVA + 4);
                if flashing(&mut g) {
                    g.ebp = game_read::<u32>(PLAYERS_RVA).wrapping_add(SECOND_VIEW);
                    game_write(TEXTURE_RVA, TEXTURES[1]);
                    crate::hud::draw_view_strips::game_part(1, &mut g)?;
                }
            }
        }
        crate::hud::draw_view_strips::game_part(2, &mut g)?;
    }
    Ok(g)
}

/// In the game: the strip's and the tail's Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::draw_view_strip as vs;
    unsafe {
        match place {
            1 => *r = vs::draw_view_strip(r, &mut vs::game_part)?,
            _ => vs::overlay_tail(r, &mut (&mut vs::game_part as vs::Part))?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_view_strips(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
