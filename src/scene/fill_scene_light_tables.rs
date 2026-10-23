//! `scene::fill_scene_light_tables` (0x4433AD): the light tables from the scene's own
//! colours, whatever the switch says.
//!
//! In:  nothing
//! Out: as `scene::fill_light_tables` leaves them
//!
//! `dword_621480` is kept, set to 1 for `scene::fill_light_tables` (which then takes
//! the scene's colours) and put back.
//! Two callers.
//! Self-test: three settings of the switch on the game's tables, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::fill_light_tables::fill_light_tables;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

// ---- game data ----

const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480

// ---- Rust version ----

pub unsafe fn fill_scene_light_tables() -> Left {
    unsafe {
        let kept: u32 = game_read(OTHER_WAY_RVA);
        game_write(OTHER_WAY_RVA, 1u32);
        let left = fill_light_tables();
        game_write(OTHER_WAY_RVA, kept);
        left
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { fill_scene_light_tables() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
