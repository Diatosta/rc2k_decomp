//! `race::mud_texture_turn` (0x44A087): the mud's colour packed into `dword_623D34`
//! (red `dword_60AD00` << 16, its second byte replaced by green's low byte and its low one by
//! blue's: `shl eax, 10h; mov ah, bl; mov al, cl`), `dword_62390C` into `dword_623910` and
//! `dword_623918` into `dword_62391C`, both 0, and `race::update_mud_texture`.
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! See `race/mud.rs`. One caller, `race::mud_frame`. Refuses what `update_mud_texture` refuses,
//! before anything (its checks first).

use crate::errors::Refused;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::owner::Race;
use crate::race::tint_to_mud_colour::mud_colour;

/// The Rust version (see the header).
pub fn mud_texture_turn(race: &Race) -> Result<(), Refused> {
    texture_and_mud()?;
    let [red, green, blue] = mud_colour(&race.course);
    // Converted: `rc2k_core::race::mud::packed_colour`.
    crate::race::owner::race()
        .shared
        .vars
        .mud_packed
        .set(rc2k_core::race::mud::packed_colour([red, green, blue]));
    for turn in [
        &crate::race::owner::race().shared.vars.mud_turn_a,
        &crate::race::owner::race().shared.vars.mud_turn_b,
    ] {
        turn[1].set(turn[0].get());
        turn[0].set(0);
    }
    crate::race::update_mud_texture::update_mud_texture(race)
}
