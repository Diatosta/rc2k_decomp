//! `race::splash_mud` (0x44A35F): one splash of mud onto the texture worked on
//! (`dword_623D5C`, n): its count `dword_623D90[n]` one more; a random place (x the low
//! byte, y the next of `math::next_random`); unless the mud map's strength there (as
//! `race::muddy_texel`'s) is below 0x20, or a random byte is above it, the 4 x 4 texels
//! from that place are faded (`gfx::fade_colour`, by each colour's top byte) toward one of
//! 16 patterns of `g_mudColours` (4 x 4 colours each, picked at random), their top bytes set,
//! and put back (`race::mud_texel`, `race::put_mud_texel`; untouched without the top bit).
//! The columns and rows counted in `dword_623D38`, `dword_623D3C` (4 each after).
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! See `race/mud.rs`. One caller, `race::update_mud_texture`, once per splash. Refuses what
//! `muddy_texel` refuses before anything, and `put_mud_texel`'s later refusals (the counts, the
//! counters and the random state put back).

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::gfx::fade_colour::fade_colour;
use crate::math::next_random::next_random;
use crate::race::mud_texel::mud_texel;
use crate::race::muddy_texel::{strength_at, texture_and_mud};
use crate::race::put_mud_texel::put_mud_texel;
use crate::race::vars_store::RACE_VARS;

pub const SPLASHED_RVA: usize = rva(0x623D90); // dword_623D90: each texture's count
pub const LAST_SPLASHED: u32 = 0x1000;

/// The Rust version (see the header).
pub unsafe fn splash_mud() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::mud::{splash_place, splash_lands, splash_pattern}`.
    use rc2k_core::race::mud::{
        MUD_TOP, SPLASH_SIDE, muddy, splash_lands, splash_pattern, splash_place,
    };
    let (n, mud) = texture_and_mud()?;
    unsafe {
        let count_at = SPLASHED_RVA + 4 * n as usize;
        game_write(count_at, game_read::<u32>(count_at).wrapping_add(1));
        let (mut x, mut y) = splash_place(next_random());
        let strength = u32::from(*(strength_at(mud, n, x, y) as *const u8));
        if !splash_lands(strength, || next_random() & 0xFF) {
            return Ok(());
        }
        // eax carries from texel to texel: `mud_texel` and `put_mud_texel` hand it back
        // when they skip a place.
        let mut eax = splash_pattern(next_random());
        let mut colours = (RACE_VARS.mud_colours[0].addr() as u32).wrapping_add(eax);
        RACE_VARS.mud_row.set_bits(0u32);
        for row in 0..SPLASH_SIDE {
            RACE_VARS.mud_column.set_bits(0u32);
            for column in 0..SPLASH_SIDE {
                eax = mud_texel(eax, x, y);
                if muddy(eax) {
                    let colour = (colours as *const u32).read_unaligned();
                    let faded = fade_colour(eax, colour, colour >> 24);
                    eax = put_mud_texel(faded | MUD_TOP, x, y)?;
                }
                colours = colours.wrapping_add(4);
                x = x.wrapping_add(1);
                RACE_VARS.mud_column.set_bits(column + 1);
            }
            x = x.wrapping_sub(4);
            y = y.wrapping_add(1);
            RACE_VARS.mud_row.set_bits(row + 1);
        }
        Ok(())
    }
}
