//! `effects::draw_overlays` (0x4C74DA): the race frame's layer of marks drawn before the car's
//! shadow (the windscreen effects' overlay layer): `scene::depth_buffer_on`, `scene::depth_test_on`, render states
//! 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 1, 0x1B = 1; `effects::draw_plane`,
//! `effects::windscreen_flakes_frame`, `effects::windscreen_drops_top`; then the second shadow, the twin of
//! `effects::draw_shadow` without its depth calls (`draw_shadow`'s `second`).
//!
//! Caller: the race's frame (`race::draw_marks_overlays`, 0x45632B, just before `draw_shadow`). the
//! shadow's vector helpers their Rust versions. Refuses the shadow's refusals (null settings, a
//! corner past
//! `g_marksCorners`, an angle `fsincos` cannot take), before anything.

use crate::effects::draw_shadow::{draw_shadow_with, second};
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::{BadInput, Refused};

const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The shadow's refusals, checked before anything is drawn.
fn shadow_refuses(effects: &crate::effects::owner::Effects) -> Option<Refused> {
    let corner: u32 = second(effects).corner();
    if corner == 0xFFFF_FFFF {
        return None;
    }
    if corner >= 0x100 {
        return Some(BadInput("a shadow corner past g_marksCorners").into());
    }
    if effects.setup.settings.is_set.get() == 0 {
        return Some(BadInput("no marks settings").into());
    }
    second(effects).angle_refuses(effects)
}

pub fn draw_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    masks: &Masks,
    drops: &WindscreenDrops,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let buffer = ctx.part.vertices;
    let (race, scene) = (ctx.race, ctx.scene);
    use crate::scene::set_render_state::set_render_state;
    if let Some(e) = shadow_refuses(effects) {
        return Err(e);
    }
    {
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
        crate::scene::depth_test_on::depth_test_on(ctx.part);
        for (value, state) in STATES {
            set_render_state(ctx.part, value, state);
        }
        let fl = &effects.windscreen_flakes;
        crate::effects::draw_plane::draw_plane(ctx, drops, fl, effects)?;
        crate::effects::windscreen_flakes_frame::windscreen_flakes_frame(
            ctx,
            masks,
            drops,
            &effects.pong,
            fl,
            effects,
        )?;
        crate::effects::windscreen_drops_top::windscreen_drops_top(
            buffer,
            &race.frame,
            masks,
            drops,
            scene,
            gfx,
            effects,
        )?;
        draw_shadow_with(ctx, masks, &second(effects), false, effects)
    }
}
