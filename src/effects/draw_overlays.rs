//! `effects::draw_overlays` (0x4C74DA): the race frame's layer of marks drawn before the car's
//! shadow (a reading): `scene::depth_buffer_on`, `scene::depth_test_on`, render states
//! 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 1, 0x1B = 1; `effects::draw_plane`,
//! `effects::windscreen_flakes_frame`, `effects::windscreen_drops_top`; then the second shadow, the twin of
//! `effects::draw_shadow` without its depth calls (`draw_shadow`'s `SECOND`).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: the race's frame (0x45632B, just before `draw_shadow`). the shadow's vector
//! helpers their Rust versions. Refuses the shadow's refusals (null settings, a corner past
//! `g_marksCorners`, an angle `fsincos` cannot take), before anything.

use crate::effects::draw_shadow::{SECOND, draw_shadow_with};
use crate::effects::mask_store::Masks;
use crate::effects::owner::effects;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;

const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The shadow's refusals, checked before anything is drawn.
fn shadow_refuses() -> Option<Refused> {
    let corner: u32 = SECOND.corner();
    if corner == 0xFFFF_FFFF {
        return None;
    }
    if corner >= 0x100 {
        return Some(BadInput("a shadow corner past g_marksCorners").into());
    }
    if effects().setup.settings.is_set.get() == 0 {
        return Some(BadInput("no marks settings").into());
    }
    SECOND.angle_refuses()
}

/// The Rust version (see the header).
pub fn draw_overlays(
    race: &Race,
    masks: &Masks,
    drops: &WindscreenDrops,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::scene::set_render_state::set_render_state;
    if let Some(e) = shadow_refuses() {
        return Err(e);
    }
    {
        crate::scene::depth_buffer_on::depth_buffer(true);
        crate::scene::depth_test_on::depth_test_on();
        for (value, state) in STATES {
            set_render_state(value, state);
        }
        let effects = effects();
        let fl = &effects.windscreen_flakes;
        crate::effects::draw_plane::draw_plane(race, drops, fl, scene)?;
        crate::effects::windscreen_flakes_frame::windscreen_flakes_frame(
            &race.frame,
            masks,
            drops,
            &effects.pong,
            fl,
            scene,
        )?;
        crate::effects::windscreen_drops_top::windscreen_drops_top(
            &race.frame,
            masks,
            drops,
            scene,
        )?;
        draw_shadow_with(race, masks, &SECOND, false)
    }
}
