//! `race::mud_frame` (0x44A054): each frame drawn, the mud's frame count `dword_623D58`
//! one more; at 0xE it is 0 again and the next of the four mud textures is worked on
//! (`dword_623D5C`, kept to 0 .. 3: `race::mud_texture_turn`, then one more, `and 3`).
//!
//! In:  nothing (the original reads no register at entry)
//! Out: nothing its one caller, `race::draw_race_scene`, reads
//!
//! See `race/mud.rs`. Refuses what `mud_texture_turn` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version (see the header).
pub fn mud_frame(race: &Race) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{count_frame, texture_turn, next_texture}`.
    use crate::race::mud_texture_turn::mud_texture_turn;
    use rc2k_core::race::mud::{count_frame, next_texture, texture_turn};
    let (frames, due) = count_frame(crate::race::owner::race().shared.vars.mud_frames.bits());
    crate::race::owner::race()
        .shared
        .vars
        .mud_frames
        .set_bits(frames);
    if !due {
        return Ok(());
    }
    crate::race::owner::race()
        .shared
        .vars
        .mud_texture
        .set_bits(texture_turn(
            crate::race::owner::race().shared.vars.mud_texture.bits(),
        ));
    mud_texture_turn(race)?;
    crate::race::owner::race()
        .shared
        .vars
        .mud_texture
        .set_bits(next_texture(
            crate::race::owner::race().shared.vars.mud_texture.bits(),
        ));
    Ok(())
}
