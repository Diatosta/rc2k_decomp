//! `scene::add_point_lights`: what the scene's lamps add to the light on an object.
//!
//! Each lamp (0x10 bytes: a point of the scene, a strength, a colour; strength 0 is skipped)
//! gives (0x200000 less its distance from the place being lit, by `scene::approximate_distance`)
//! times the strength over 0x20000, held to 0 ..= 255, in all three colour places, each times
//! the lamp's colour's over 256. That is added to the light colour byte by byte without
//! overflow (all four bytes with MMX, the low three without, as in `scene::vertex_light`).
//! Refuses more than 0x100 lamps and lamps without the
//! scene's points.

use crate::errors::{BadInput, Refused};

// ---- game data ----
const MOST: u32 = 0x100;

// Logic: `rc2k_core::scene::lamps::add_point_lights`
pub fn add_point_lights(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    use rc2k_core::scene::lamps::{self, Lamp};

    let lamps: u32 = ctx.view.lamps.lamp_count.get();
    if lamps == 0 {
        return Ok(());
    }
    if lamps > MOST || !crate::race::scene_points::held(&ctx.reads.stage.course) {
        return Err(BadInput("more lamps than the list holds, or no points").into());
    }
    let mmx = ctx.reads.has_mmx;
    let place: [u32; 3] = crate::session::read_bits(&ctx.view.light.lit_place);
    let reading = crate::scene::reading::Reading::new();
    let list = (0..lamps as usize).map(|n| {
        // The list holds 16 lamps, the refusal allows `MOST`; the only write of the count
        // is a 0 (`race::frame_draw::panel`), so more than 16 is never met.
        let [point, strength, colour] = [0, 1, 2].map(|k| {
            reading.keep(
                ctx.view
                    .lamps
                    .lamps
                    .get(4 * n + k)
                    .map(crate::session::PlainDword::get)
                    .ok_or(BadInput("more lamps than the 16 of the level")),
            )
        });
        Lamp {
            point,
            strength,
            colour,
        }
    });
    let at = |point: u32| {
        reading.keep(crate::race::scene_points::point_number(
            ctx.reads.course(),
            &ctx.part.points,
            point,
        ))
    };
    let out = lamps::add_point_lights(list, place, at, ctx.view.light.light_colour.get(), mmx);
    reading.finish()?;
    for (to, value) in [
        (&ctx.view.light.lamp_tint, out.colour),
        (&ctx.view.light.light_colour, out.light),
    ] {
        if let Some(v) = value {
            to.set(v);
        }
    }
    Ok(())
}
