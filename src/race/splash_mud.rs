//! `race::splash_mud` (0x44A35F): one splash of mud onto the texture worked on (`mud_texture`, n):
//! its count `mud_splash_counts`[n]` one more; a random place (x the low byte, y the next of the
//! drawing's random numbers, `next_drawing_random`); unless the mud map's strength there (as
//! `race::muddy_texel`'s) is below 0x20, or a random byte is above it, the 4 x 4 texels from that
//! place are faded (`gfx::fade_colour`, by each colour's top byte) toward one of 16 patterns of
//! `g_mudColours` (4 x 4 colours each, picked at random), their top bytes set, and put back
//! (`race::mud_texel`, `race::put_mud_texel`; untouched without the top bit). The columns and rows
//! counted in `mud_column`, `mud_row` (4 each after).
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per splash. Refuses what
//! `muddy_texel` refuses before anything, and `put_mud_texel`'s later refusals (the counts, the
//! counters and the random state put back).

use crate::errors::{BadInput, Refused};
use crate::gfx::fade_colour::fade_colour;
use crate::math::next_random::next_drawing_random;
use crate::race::mud_texel::mud_texel;
use crate::race::muddy_texel::{strength_at, texture_and_mud};
use crate::race::owner_shared::RaceShared;
use crate::race::put_mud_texel::put_mud_texel;

pub const LAST_SPLASHED: u32 = 0x1000;

pub fn splash_mud(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{splash_place, splash_lands, splash_pattern}`.
    use rc2k_core::race::mud::{
        MUD_TOP, SPLASH_SIDE, muddy, splash_lands, splash_pattern, splash_place,
    };
    let n = texture_and_mud(shared)?;
    // `mud_splash_counts`: each texture's count (the texture is 0 .. 3).
    let count_cell = &shared.vars.mud_splash_counts[n as usize];
    count_cell.set(count_cell.get().wrapping_add(1));
    let (mut x, mut y) = splash_place(next_drawing_random());
    let strength = strength_at(n, x, y)?;
    if !splash_lands(strength, || next_drawing_random() & 0xFF) {
        return Ok(());
    }
    // The pattern carries from texel to texel: `mud_texel` and `put_mud_texel` hand it back
    // when they skip a place.
    let mut eax = splash_pattern(next_drawing_random());
    let mut colours = (eax >> 2) as usize; // the pattern's place: whole dwords
    shared.vars.mud_row.set(0u32);
    for row in 0..SPLASH_SIDE {
        shared.vars.mud_column.set(0u32);
        for column in 0..SPLASH_SIDE {
            eax = mud_texel(eax, x, y, scene, gfx, system)?;
            if muddy(eax) {
                // The pattern's place is 16 x (0 .. 15) and the splash steps it 16 times:
                // at most 0xFF, inside the 256 colours.
                let colour = shared
                    .vars
                    .mud_colours
                    .get(colours)
                    .ok_or(BadInput("a mud colour past the 256"))?
                    .get();
                let faded = fade_colour(eax, colour, colour >> 24);
                eax = put_mud_texel(faded | MUD_TOP, x, y, scene, gfx, system)?;
            }
            colours += 1;
            x = x.wrapping_add(1);
            shared.vars.mud_column.set(column + 1);
        }
        x = x.wrapping_sub(4);
        y = y.wrapping_add(1);
        shared.vars.mud_row.set(row + 1);
    }
    Ok(())
}
