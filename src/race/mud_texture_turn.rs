//! `race::mud_texture_turn` (0x44A087): the mud's colour packed into `dword_623D34`
//! (red `dword_60AD00` << 16, its second byte replaced by green's low byte and its low one by
//! blue's: `shl eax, 10h; mov ah, bl; mov al, cl`), `dword_62390C` into `dword_623910` and
//! `dword_623918` into `dword_62391C`, both 0, and `race::update_mud_texture`.
//!
//! In:  nothing
//! Out: the registers as `update_mud_texture` leaves them; no flags read
//!
//! See `race/mud.rs`. One caller, `race::mud_frame`. Its call through `part` (0, in the
//! game the Rust version with its own calls). Refuses what `update_mud_texture` refuses,
//! before anything (its checks first). Shadow mode runs the original alone (the device is
//! told). Self-test: a copy whose call goes to a stand-in, against the Rust version, on
//! colours and values by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::race::muddy_texel::texture_and_mud;
use crate::race::tint_to_mud_colour::mud_colour;

const PACKED_RVA: usize = rva(0x623D34); // dword_623D34
const MOVED: [(usize, usize); 2] = [
    (rva(0x62390C), rva(0x623910)), // dword_62390C into dword_623910
    (rva(0x623918), rva(0x62391C)), // dword_623918 into dword_62391C
];

/// A call by place: 0 `race::update_mud_texture`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn mud_texture_turn(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    texture_and_mud()?;
    let mut r = *entry;
    let [red, green, blue] = mud_colour();
    // Converted: `rc2k_core::race::mud::packed_colour`.
    r.eax = rc2k_core::race::mud::packed_colour([red, green, blue]);
    r.ebx = green;
    r.ecx = blue;
    unsafe {
        game_write(PACKED_RVA, r.eax);
        for (from, to) in MOVED {
            game_write(to, game_read::<u32>(from));
            game_write(from, 0u32);
        }
    }
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: `update_mud_texture`'s Rust version with its own calls.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::update_mud_texture::{game_part, update_mud_texture};
    *r = unsafe { update_mud_texture(r, &mut game_part) }?;
    Ok(())
}
