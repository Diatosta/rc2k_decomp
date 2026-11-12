//! `scene::add_point_lights` (0x44F471): what the scene's lamps add to the light on an
//! object.
//!
//! For each of the scene's lamps (0x10 bytes each: a point of the scene, a strength, a colour;
//! a strength of 0 is skipped): the strength and the colour kept; the lamp's distance from the
//! place being lit by `scene::approximate_distance` (the original kept it in the shared work
//! dword, which nothing read: not kept); (0x200000 less the distance) times the strength over
//! 0x20000, held to 0 .. 255, in all three colour places, each times the lamp's colour's over
//! 256; and that added to the light colour byte by byte without overflow (all four bytes with
//! MMX, the low three without, as in `scene::vertex_light`).
//! One caller, `scene::gather_lights`.
//! Refuses more than 0x100 lamps and lamps without the scene's points.

use crate::errors::{BadInput, Refused};

// ---- game data ----
const MOST: u32 = 0x100;

// ---- Rust version ----

// Converted: `rc2k_core::scene::lamps::add_point_lights`
pub fn add_point_lights(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    use rc2k_core::scene::lamps::{self, Lamp};

    let lamps: u32 = ctx.view.lamps.lamp_count.get();
    if lamps == 0 {
        return Ok(());
    }
    let points: u32 = ctx.reads.course_work.scene_points.get();
    if lamps > MOST || points == 0 {
        return Err(BadInput("more lamps than the list holds, or no points").into());
    }
    let mmx = ctx.reads.has_mmx;
    let place: [u32; 3] = crate::session::read_bits(&ctx.view.light.lit_place);
    let reading = crate::scene::reading::Reading::new();
    let list = (0..lamps as usize).map(|n| {
        // The list holds 16 lamps, the refusal allows `MOST`: read by address.
        let [point, strength, colour] = [0, 1, 2].map(|k| {
            reading.keep(crate::session::read_on_in(
                &ctx.view.lamps,
                &ctx.view.lamps.lamps,
                4 * n + k,
                "a lamp of the level",
            ))
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
