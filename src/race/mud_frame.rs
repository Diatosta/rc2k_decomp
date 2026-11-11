//! `race::mud_frame` (0x44A054): each frame drawn, the mud's frame count `mud_frames` one more;
//! at 0xE it is 0 again and the next of the four mud textures is worked on (`mud_texture`, kept to
//! 0 .. 3: `race::mud_texture_turn`, then one more, `and 3`).
//!
//! See `race/mud.rs`. Refuses what `mud_texture_turn` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn mud_frame(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{count_frame, texture_turn, next_texture}`.
    use crate::race::mud_texture_turn::mud_texture_turn;
    use rc2k_core::race::mud::{count_frame, next_texture, texture_turn};
    let (frames, due) = count_frame(race.shared.vars.mud_frames.get());
    race.shared.vars.mud_frames.set(frames);
    if !due {
        return Ok(());
    }
    race.shared
        .vars
        .mud_texture
        .set(texture_turn(race.shared.vars.mud_texture.get()));
    mud_texture_turn(race, scene)?;
    race.shared
        .vars
        .mud_texture
        .set(next_texture(race.shared.vars.mud_texture.get()));
    Ok(())
}
