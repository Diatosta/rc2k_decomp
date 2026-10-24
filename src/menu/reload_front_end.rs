//! `menu::reload_front_end` (0x48140D): when `dword_7587E8` is 1 (the scene was let go for
//! a race), the front end's scene set up again: `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! 36 callers (the menus after their races). The three calls through `part` (1 the engine,
//! 2 the scene), in the game their Rust versions (`set_up_engine_in_game`,
//! `set_up_front_end_scene_in_game`). Shadow mode runs the original alone (it loads the
//! scene). Self-test: a copy whose calls go to stand-ins, against the Rust version, with
//! the switch 0 and 1.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, rva};

const SWITCH_RVA: usize = rva(0x7587E8); // dword_7587E8

/// The Rust version (see the header).
pub unsafe fn reload_front_end() -> Result<(), Refused> {
    if unsafe { game_read::<u32>(SWITCH_RVA) } == 1 {
        unsafe {
            crate::scene::set_up_engine::set_up_engine()?;
            crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
            crate::scene::set_up_engine::set_up_engine()?;
        }
    }
    Ok(())
}
