//! `race::throw_mud_texel` (0x44A2A2): one throw of mud onto the texture worked on
//! (`dword_623D5C`, n): while its count `dword_623D80[n]` is below 0x60000, the texel of
//! that number (`race::muddy_texel`, the places in their shuffled order) and the count
//! one more; then `dword_623D30` + 0x10000000, and when that carries (every 16th) a texel
//! at a random number too (`math::next_random`).
//!
//! In:  nothing
//! Out: ebx = n when the count is used up; else the registers as the last `muddy_texel`
//!      leaves them (ebx n again after the first when there is no second); no flags read
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
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::next_random::{STATE_RVA as RANDOM_RVA, next_random};
use crate::physics::flow::guard;
use crate::race::muddy_texel::{STRENGTH_RVA, muddy_texel, texture_and_mud};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub const THROWN_RVA: usize = rva(0x623D80); // dword_623D80: each texture's count
pub use rc2k_core::race::mud::LAST_THROWN;
pub const EXTRA_SUM_RVA: usize = rva(0x623D30); // dword_623D30

/// The counts, the sum and the random state: what a throw writes besides the texels.
pub fn throw_regions() -> Vec<Region> {
    vec![
        (game_addr(THROWN_RVA) as u32, 0x10),
        at(EXTRA_SUM_RVA),
        at(RANDOM_RVA),
        at(STRENGTH_RVA),
    ]
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn throw_mud_texel(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::mud::{LAST_THROWN, extra_throw}`.
    let (n, _) = texture_and_mud()?;
    guard(&throw_regions(), || unsafe {
        let mut r = Regs { ebx: n, ..*entry };
        let count_at = THROWN_RVA + 4 * n as usize;
        let count: u32 = game_read(count_at);
        if count >= LAST_THROWN {
            return Ok(r);
        }
        game_write(count_at, count + 1);
        r.eax = count;
        r = muddy_texel(&r)?;
        r.ebx = n;
        let (sum, carried) = rc2k_core::race::mud::extra_throw(game_read(EXTRA_SUM_RVA));
        game_write(EXTRA_SUM_RVA, sum);
        if !carried {
            return Ok(r);
        }
        r.eax = next_random();
        muddy_texel(&r)
    })
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { throw_mud_texel(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
