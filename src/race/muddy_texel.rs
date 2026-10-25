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
use crate::framework::rva;
use crate::race::mud::MUD_RVA;
use crate::race::mud_texel::mud_texel;
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::tint_to_mud_colour::tint_to_mud_colour;
use crate::race::vars_store::RACE_VARS;
use crate::scene::draw_store::DRAW;
use crate::store::game_read;

pub const SHUFFLED_RVA: usize = rva(0xA6BB54); // g_shuffledWords, 0x10000 words
// g_surface.format

/// The place (x, y) of number `n` (its low 16 bits).
pub fn shuffled_place(n: u32) -> (u32, u32) {
    let word: u16 = unsafe { game_read(SHUFFLED_RVA + 2 * (n & 0xFFFF) as usize) };
    rc2k_core::race::mud::shuffled_place(word)
}

/// The texture worked on and the mud, or the refusal.
pub fn texture_and_mud() -> Result<(u32, u32), Refused> {
    let n: u32 = RACE_VARS.mud_texture.bits();
    if n > rc2k_core::race::mud::MOST_TEXTURE {
        return Err(BadInput("a mud texture past 3 (the map would be read past its end)").into());
    }
    let mud: u32 = unsafe { game_read(MUD_RVA) };
    if mud == 0 {
        return Err(BadInput("no mud").into());
    }
    Ok((n, mud))
}

/// The strength's address for texture `n` at (x, y).
pub fn strength_at(mud: u32, n: u32, x: u32, y: u32) -> u32 {
    mud.wrapping_add(rc2k_core::race::mud::strength_index(n, x, y))
}

/// The Rust version (see the header): the texel of place number `number` (its low 16
/// bits) muddied.
pub unsafe fn muddy_texel(number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{shuffled_place, strength_index, muddy,
    // tint_amount}`.
    use rc2k_core::race::mud::{MUD_TOP, muddy, strength_row, tint_amount};
    let (n, mud) = texture_and_mud()?;
    let (x, y) = shuffled_place(number);
    let strength = u32::from(unsafe { *(strength_at(mud, n, x, y) as *const u8) });
    let kept: u32 = DRAW.scratch.bits();
    DRAW.scratch.set_bits(strength);
    let texel = unsafe { mud_texel(strength_row(y), x, y) };
    if !muddy(texel) {
        return Ok(());
    }
    let tinted = tint_to_mud_colour(texel, tint_amount(strength));
    if let Err(e) = unsafe { put_mud_texel(tinted | MUD_TOP, x, y) } {
        DRAW.scratch.set_bits(kept);
        return Err(e);
    }
    Ok(())
}
