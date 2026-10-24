//! `net::lobby_launched_game` (0x418DF4): the game as a lobby launched it (`g_lobbyLaunched`
//! 1, instead of the front end): `sub_4154CD` (the front end's set-up, agent F's), the
//! network game (`net::start_link`, mode 3), then the 3D tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`, `race::release_textures`).
//!
//! In:  nothing
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Caller: `system::main_loop` (whose Rust version calls this one).

use crate::errors::Refused;
use crate::net::lobby::{LOBBY_LAUNCHED, g};

/// The Rust version.
pub fn lobby_launched_game() -> Result<(), Refused> {
    if g(LOBBY_LAUNCHED) == 1 {
        crate::scene::rebuild_front_end::rebuild_front_end()?;
        crate::net::start_link::start_link(3)?;
        unsafe {
            crate::race::stage_free::render3d_tidy()?;
            crate::race::stage_free::objects_tidy()?;
            crate::race::race_end::release_textures();
        }
    }
    Ok(())
}
