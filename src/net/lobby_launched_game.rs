//! `net::lobby_launched_game` (0x418DF4): the game as a lobby launched it (`g_lobbyLaunched`
//! 1, instead of the front end): `sub_4154CD` (the front end's set-up, agent F's), the
//! network game (`net::start_link`, mode 3), then the 3D tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`, `race::release_textures`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Caller: `system::main_loop` (whose Rust version calls this one). Its calls through `part`
//! (`sub_4154CD` through the game). The whole game runs inside it: ungated. Shadow mode runs
//! the original alone. Self-test: a copy whose calls go to stand-ins, against the Rust
//! version, launched or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    LOBBY_LAUNCHED, NULLSUB_6, OBJECTS_TIDY, Part, RELEASE_TEXTURES, RENDER3D_TIDY,
    START_FRONT_END, g, lobby_entry,
};

const START_LINK: u32 = 0x416F50; // net::start_link

/// The Rust version: the registers it leaves (the entry's).
pub fn lobby_launched_game(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    if g(LOBBY_LAUNCHED) == 1 {
        part(START_FRONT_END, &mut r)?;
        r.eax = 3;
        for place in [
            START_LINK,
            RENDER3D_TIDY,
            OBJECTS_TIDY,
            NULLSUB_6,
            RELEASE_TEXTURES,
        ] {
            part(place, &mut r)?;
        }
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, lobby_launched_game)
}
