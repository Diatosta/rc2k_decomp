//! `race::loading_bar_quad` (0x446861): the loading bar, a red (`dword_623DE8` 0xFF0000)
//! flat rectangle along the bottom: x from 0 to the screen's width times `dword_623700`
//! (held to 0 .. 100) over 100, y from 0x24D to 0x258 scaled from 600 to the screen's
//! height; `dword_6240C4` -1; `race::draw_screen_quad`.
//!
//! In:  nothing
//! Out: eax the right edge, ebx 100, ecx 0x258, edx the rest of its division; then as the
//!      quad leaves them (its callers return at once)
//!
//! Caller: `race::loading_bar_frame`. The quad through `part` (1), in the game its Rust
//! version. Shadow mode runs the original alone (it draws). Self-test: a copy whose quad
//! goes to a stand-in, against the Rust version, shares by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const QUAD_IN_RVA: usize = rva(0x623DE8); // colour; x at +0x28 / +0x2C, y at +0x38 / +0x3C
pub const SHARE_RVA: usize = rva(0x623700); // dword_623700: the share loaded, 0 .. 100
const SWITCH_RVA: usize = rva(0x6240C4); // dword_6240C4
const SCREEN_RVA: [usize; 2] = [rva(0x4F494C), rva(0x4F4950)];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn loading_bar_quad(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::loading::loading_bar`.
    use rc2k_core::race::loading::{BAR_COLOUR, loading_bar};
    let mut g = *entry;
    unsafe {
        let [w, h] = SCREEN_RVA.map(|r| game_read::<u32>(r));
        let q = |o: usize, v: u32| game_write(QUAD_IN_RVA + o, v);
        let bar = loading_bar(w, h, game_read(SHARE_RVA));
        q(0x28, bar.left);
        q(0, BAR_COLOUR);
        q(0x38, bar.top);
        q(0x3C, bar.bottom);
        g.eax = bar.right;
        q(0x2C, g.eax);
        [g.ebx, g.ecx, g.edx] = [0x64, 0x258, bar.rest];
        game_write(SWITCH_RVA, 0xFFFF_FFFFu32);
        crate::race::loading_bar_quad::game_part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `draw_screen_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::race::frame_draw::draw_screen_quad(r) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { loading_bar_quad(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
