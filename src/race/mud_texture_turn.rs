//! `race::mud_texture_turn` (0x44A087): the mud's colour packed into `mud_packed`, `mud_turn_a`
//! into `RaceVars.mud_turn_a[1]` and `mud_turn_b` into `RaceVars.mud_turn_b[1]`, both 0, and
//! `race::update_mud_texture`.
//!
//! See `race/mud.rs`. One caller, `race::mud_frame`. Refuses what `update_mud_texture` refuses,
//! before anything (its checks first).

use crate::errors::Refused;
use crate::race::muddy_texel::texture_and_mud;
use crate::race::owner::Race;
use crate::race::tint_to_mud_colour::mud_colour;

pub fn mud_texture_turn(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    texture_and_mud(&race.shared)?;
    let [red, green, blue] = mud_colour(&race.course);
    // Converted: `rc2k_core::race::mud::packed_colour`.
    race.shared
        .vars
        .mud_packed
        .set(rc2k_core::race::mud::packed_colour([red, green, blue]));
    for turn in [&race.shared.vars.mud_turn_a, &race.shared.vars.mud_turn_b] {
        turn[1].set(turn[0].get());
        turn[0].set(0);
    }
    crate::race::update_mud_texture::update_mud_texture(race, scene, gfx)
}
