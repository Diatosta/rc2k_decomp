//! `marks::draw_overlays` (0x4C74DA): the race frame's layer of marks drawn before the car's
//! shadow (a reading): `scene::depth_buffer_on`, `scene::depth_test_on`, render states
//! 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 1, 0x1B = 1; `marks::draw_plane`,
//! `marks::squares_frame`, `marks::hexagons_top`; then the second shadow, the twin of
//! `marks::draw_shadow` without its depth calls (`draw_shadow`'s `SECOND`).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: the race's frame (0x45632B, just before `draw_shadow`). Hooked through a
//! private copy (a call at the entry). The calls through `part` (1 the depth buffer, 2 the
//! depth test, 3 a render state, 4 `draw_plane`, 5 `squares_frame`, 6 `hexagons_top`, 7
//! `draw_fan`); the shadow's vector helpers their Rust versions. Refuses the shadow's
//! refusals (null settings, a corner past `g_marksCorners`, an angle `fsincos` cannot
//! take), before anything. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose calls go to stand-ins (the vector helpers real), against the Rust version,
//! as `draw_shadow`'s on the second shadow's variables.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::marks::draw_shadow::{SECOND, draw_shadow_with};
use crate::marks::setup::SETTINGS_RVA;

const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The shadow's refusals, checked before anything is drawn.
fn shadow_refuses() -> Option<Refused> {
    let corner: u32 = unsafe { game_read(SECOND.corner()) };
    if corner == 0xFFFF_FFFF {
        return None;
    }
    if corner >= 0x100 {
        return Some(BadInput("a shadow corner past g_marksCorners").into());
    }
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Some(BadInput("no marks settings").into());
    }
    SECOND.angle_refuses()
}

/// The Rust version (see the header).
pub unsafe fn draw_overlays() -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    if let Some(e) = shadow_refuses() {
        return Err(e);
    }
    unsafe {
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_on::depth_test_on();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        crate::marks::draw_plane::draw_plane()?;
        crate::marks::squares_frame::squares_frame()?;
        crate::marks::hexagons_top::hexagons_top()?;
        draw_shadow_with(&SECOND, false)
    }
}
