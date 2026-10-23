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
use crate::framework::{game_read, game_write, rva};
use crate::hud::draw_view_strip::TEXTURE_RVA;
use crate::hud::overlay_pass::shown_layout;

const OFF_RVA: usize = rva(0x5BEC94); // dword_5BEC94
const BLOCKS_RVA: usize = rva(0x63DBB0); // g_viewBlocks

/// Whether the block at `block`'s strip is shown (see the header).
unsafe fn flashing(block: u32) -> bool {
    use rc2k_core::hud::strips::strip;
    strip(|o| unsafe { ((block + o) as *const u32).read_unaligned() }).shown()
}

/// The Rust version (see the header). What the original leaves in the registers is read
/// by none of its callers (register liveness, 2026-10-23).
// Converted: `rc2k_core::hud::strips`
pub unsafe fn draw_view_strips() -> Result<(), Refused> {
    use crate::hud::draw_view_strip::{draw_view_strip, overlay_tail};
    use rc2k_core::hud::strips::{SECOND_VIEW, TEXTURES};
    unsafe {
        if game_read::<u32>(OFF_RVA) == 0 {
            if flashing(game_read(BLOCKS_RVA)) {
                game_write(TEXTURE_RVA, TEXTURES[0]);
                draw_view_strip(shown_layout())?;
            }
            if shown_layout() != 0 && flashing(game_read(BLOCKS_RVA + 4)) {
                game_write(TEXTURE_RVA, TEXTURES[1]);
                draw_view_strip(shown_layout().wrapping_add(SECOND_VIEW))?;
            }
        }
        overlay_tail()
    }
}
