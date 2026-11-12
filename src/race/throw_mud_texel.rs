//! `race::throw_mud_texel` (0x44A2A2): one throw of mud onto the texture worked on (`mud_texture`,
//! n): while its count `mud_thrown_counts`[n]` is below 0x60000, the texel of that number
//! (`race::muddy_texel`, the places in their shuffled order) and the count one more; then
//! `mud_extra_sum` + 0x10000000, and when that carries (every 16th) a texel at a random number too
//! (`math::next_random::next_drawing_random`, the drawing's numbers).
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per thrown unit. Refuses
//! what `muddy_texel` refuses (a texture past 3, no mud) before anything, and its later
//! refusals (the counts and the random state put back).

use crate::errors::Refused;
use crate::math::next_random::next_drawing_random;
use crate::race::muddy_texel::{muddy_texel, texture_and_mud};
use crate::race::owner::Race;

pub use rc2k_core::race::mud::LAST_THROWN;

pub fn throw_mud_texel(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let shared = &race.shared;
    // Converted: `rc2k_core::race::mud::{LAST_THROWN, extra_throw}`.
    let n = texture_and_mud(&race.shared)?;
    // `mud_thrown_counts`: each texture's count (the texture is 0 .. 3).
    let count_cell = &shared.vars.get().mud_thrown_counts[n as usize];
    let count: u32 = count_cell.get();
    if count >= LAST_THROWN {
        return Ok(());
    }
    count_cell.set(count + 1);
    muddy_texel(race, count, scene, gfx, system)?;
    let (sum, carried) = rc2k_core::race::mud::extra_throw(shared.vars.get().mud_extra_sum.get());
    shared.vars.get().mud_extra_sum.set(sum);
    if !carried {
        return Ok(());
    }
    muddy_texel(race, next_drawing_random(), scene, gfx, system)
}
