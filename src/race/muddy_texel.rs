//! `race::muddy_texel`: one texel of the mud texture being worked on (`mud_texture`, 0
//! .. 3) brought toward the mud's colour: the place is the word `shuffled_words[n & 0xFFFF]` (x
//! its low byte, y its high one), the strength the mud map's byte at (x / 2, y / 2) of the
//! texture's 128 x 128 (`mud` + 0x4000 n), kept in the scratch dword. When `race::mud_texel` there
//! has its top bit set, it is tinted by `race::tint_to_mud_colour`, the top byte set, and put back
//! (`race::put_mud_texel`).
//!
//! See `race/mud.rs`. Callers: `race::throw_mud_texel` (twice). Refuses a texture past 3 (the
//! map would be read past its 64 KB) and no mud, before anything; what `put_mud_texel` refuses
//! (`scratch` put back).

use crate::errors::{BadInput, Refused};
use crate::race::mud_texel::mud_texel;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::tint_to_mud_colour::tint_to_mud_colour;

/// The place (x, y) of number `n` (its low 16 bits).
pub fn shuffled_place(race: &Race, n: u32) -> (u32, u32) {
    let word: u16 = race.engine_tables().shuffled_words[(n & 0xFFFF) as usize].get();
    rc2k_core::race::mud::shuffled_place(word)
}

/// The texture worked on, or the refusal (also without the mud).
pub fn texture_and_mud(
    shared: &RaceShared,
    mud: &crate::memory::owned::Owned<Vec<u8>>,
) -> Result<u32, Refused> {
    let n: u32 = shared.vars.get().mud_texture.get();
    if n > rc2k_core::race::mud::MOST_TEXTURE {
        return Err(BadInput("a mud texture past 3 (the map would be read past its end)").into());
    }
    if !mud.is_taken() {
        return Err(BadInput("no mud").into());
    }
    Ok(n)
}

/// The mud's strength for texture `n` at (x, y).
pub fn strength_at(
    mud: &crate::memory::owned::Owned<Vec<u8>>,
    n: u32,
    x: u32,
    y: u32,
) -> Result<u32, BadInput> {
    let at = rc2k_core::race::mud::strength_index(n, x, y) as usize;
    mud.with(|m| m.get(at).copied())?
        .map(u32::from)
        .ok_or(BadInput("a mud strength past the mud"))
}

/// The texel of place number `number` (its low 16 bits) muddied.
pub fn muddy_texel(
    race: &Race,
    number: u32,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let course = &race.course;
    // Logic in `rc2k_core::race::mud::{shuffled_place, strength_index, muddy,
    // tint_amount}`.
    use rc2k_core::race::mud::{MUD_TOP, muddy, strength_row, tint_amount};
    let n = texture_and_mud(&race.shared, &course.stage.mud)?;
    let (x, y) = shuffled_place(race, number);
    let strength = strength_at(&course.stage.mud, n, x, y)?;
    let texel = { mud_texel(strength_row(y), x, y, scene, gfx, system) }?;
    if !muddy(texel) {
        return Ok(());
    }
    let tinted = tint_to_mud_colour(course, texel, tint_amount(strength));
    put_mud_texel(tinted | MUD_TOP, x, y, scene, gfx, system)?;
    Ok(())
}
