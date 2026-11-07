//! `race::muddy_texel` (0x44A2E1): one texel of the mud texture being worked on
//! (`dword_623D5C`, 0 .. 3) brought toward the mud's colour: the place is the word
//! `g_shuffledWords[eax & 0xFFFF]` (x its low byte, y its high one), the strength the mud
//! map's byte at (x / 2, y / 2) of the texture's 128 x 128 (`g_mud` + 0x4000 n), kept in
//! `dword_62400C`. When `race::mud_texel` there has its top bit set, it is tinted by
//! `race::tint_to_mud_colour` (ebp = (strength + 1) / 2), the top byte set, and put back
//! (`race::put_mud_texel`).
//!
//! In:  the place's number (eax)
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24: the registers it left
//!      went; `throw_mud_texel`'s results are read by nobody)
//!
//! See `race/mud.rs`. Callers: `race::throw_mud_texel` (twice). Refuses a texture past 3 (the
//! map would be read past its 64 KB) and no mud, before anything; what `put_mud_texel` refuses
//! (`dword_62400C` put back).

use crate::errors::{BadInput, Refused};
use crate::race::mud_texel::mud_texel;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::tint_to_mud_colour::tint_to_mud_colour;

// g_surface.format

/// The place (x, y) of number `n` (its low 16 bits).
pub fn shuffled_place(n: u32) -> (u32, u32) {
    let word: u16 = crate::scene::engine_tables_store::ENGINE_TABLES.shuffled_words
        [(n & 0xFFFF) as usize]
        .get();
    rc2k_core::race::mud::shuffled_place(word)
}

/// The texture worked on, or the refusal (also without the mud).
pub fn texture_and_mud(shared: &Shared) -> Result<u32, Refused> {
    let n: u32 = shared.vars.mud_texture.bits();
    if n > rc2k_core::race::mud::MOST_TEXTURE {
        return Err(BadInput("a mud texture past 3 (the map would be read past its end)").into());
    }
    if !crate::race::mud::MUD.is_taken() {
        return Err(BadInput("no mud").into());
    }
    Ok(n)
}

/// The mud's strength for texture `n` at (x, y).
pub fn strength_at(n: u32, x: u32, y: u32) -> Result<u32, BadInput> {
    let at = rc2k_core::race::mud::strength_index(n, x, y) as usize;
    crate::race::mud::MUD
        .with(|m| m.get(at).copied())?
        .map(u32::from)
        .ok_or(BadInput("a mud strength past the mud"))
}

/// The Rust version (see the header): the texel of place number `number` (its low 16
/// bits) muddied.
pub fn muddy_texel(
    race: &Race,
    number: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    let drawing = &scene.drawing;
    // Converted: `rc2k_core::race::mud::{shuffled_place, strength_index, muddy,
    // tint_amount}`.
    use rc2k_core::race::mud::{MUD_TOP, muddy, strength_row, tint_amount};
    let n = texture_and_mud(&race.shared)?;
    let (x, y) = shuffled_place(number);
    let strength = strength_at(n, x, y)?;
    let kept: u32 = drawing.scene.scratch.bits();
    drawing.scene.scratch.set_bits(strength);
    let texel = { mud_texel(strength_row(y), x, y) }?;
    if !muddy(texel) {
        return Ok(());
    }
    let tinted = tint_to_mud_colour(course, texel, tint_amount(strength));
    if let Err(e) = { put_mud_texel(tinted | MUD_TOP, x, y) } {
        drawing.scene.scratch.set_bits(kept);
        return Err(e);
    }
    Ok(())
}
