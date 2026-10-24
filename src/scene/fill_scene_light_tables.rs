//! `scene::fill_scene_light_tables` (0x4433AD): the light tables from the scene's own
//! colours, whatever the switch says.
//!
//! In:  nothing
//! Out: as `scene::fill_light_tables` leaves them
//!
//! `dword_621480` is kept, set to 1 for `scene::fill_light_tables` (which then takes the
//! scene's colours) and put back. Two callers.

use crate::framework::{game_read, game_write, rva};
use crate::scene::fill_light_tables::fill_light_tables;

// ---- game data ----

const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480

// ---- Rust version ----

pub unsafe fn fill_scene_light_tables() {
    unsafe {
        let kept: u32 = game_read(OTHER_WAY_RVA);
        game_write(OTHER_WAY_RVA, 1u32);
        fill_light_tables();
        game_write(OTHER_WAY_RVA, kept);
    }
}
