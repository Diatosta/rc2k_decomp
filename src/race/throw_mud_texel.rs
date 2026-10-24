//! `race::throw_mud_texel` (0x44A2A2): one throw of mud onto the texture worked on
//! (`dword_623D5C`, n): while its count `dword_623D80[n]` is below 0x60000, the texel of
//! that number (`race::muddy_texel`, the places in their shuffled order) and the count
//! one more; then `dword_623D30` + 0x10000000, and when that carries (every 16th) a texel
//! at a random number too (`math::next_random`).
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per thrown unit.
//! Refuses what `muddy_texel` refuses (a texture past 3, no mud) before anything, and its
//! later refusals (the counts and the random state put back). Self-test: the original and
//! the Rust version on the mud tests' set-up, counts at and below the end, sums that carry
//! and not; the surface, the counts and the random state compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::math::next_random::next_random;
use crate::race::muddy_texel::{muddy_texel, texture_and_mud};

pub const THROWN_RVA: usize = rva(0x623D80); // dword_623D80: each texture's count
pub use rc2k_core::race::mud::LAST_THROWN;
pub const EXTRA_SUM_RVA: usize = rva(0x623D30); // dword_623D30

/// The Rust version (see the header).
pub unsafe fn throw_mud_texel() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{LAST_THROWN, extra_throw}`.
    let (n, _) = texture_and_mud()?;
    unsafe {
        let count_at = THROWN_RVA + 4 * n as usize;
        let count: u32 = game_read(count_at);
        if count >= LAST_THROWN {
            return Ok(());
        }
        game_write(count_at, count + 1);
        muddy_texel(count)?;
        let (sum, carried) = rc2k_core::race::mud::extra_throw(game_read(EXTRA_SUM_RVA));
        game_write(EXTRA_SUM_RVA, sum);
        if !carried {
            return Ok(());
        }
        muddy_texel(next_random())
    }
}
