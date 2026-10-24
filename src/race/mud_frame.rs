//! `race::mud_frame` (0x44A054): each frame drawn, the mud's frame count `dword_623D58`
//! one more; at 0xE it is 0 again and the next of the four mud textures is worked on
//! (`dword_623D5C`, kept to 0 .. 3: `race::mud_texture_turn`, then one more, `and 3`).
//!
//! In:  nothing (the original reads no register at entry)
//! Out: nothing its one caller, `race::draw_race_scene`, reads
//!
//! See `race/mud.rs`. Refuses what `mud_texture_turn` refuses.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::race::muddy_texel::TEXTURE_RVA;

const FRAMES_RVA: usize = rva(0x623D58); // dword_623D58

/// The Rust version (see the header).
pub unsafe fn mud_frame() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{count_frame, texture_turn, next_texture}`.
    use crate::race::mud_texture_turn::mud_texture_turn;
    use rc2k_core::race::mud::{count_frame, next_texture, texture_turn};
    unsafe {
        let (frames, due) = count_frame(game_read(FRAMES_RVA));
        game_write(FRAMES_RVA, frames);
        if !due {
            return Ok(());
        }
        game_write(TEXTURE_RVA, texture_turn(game_read(TEXTURE_RVA)));
        mud_texture_turn()?;
        game_write(TEXTURE_RVA, next_texture(game_read(TEXTURE_RVA)));
        Ok(())
    }
}
