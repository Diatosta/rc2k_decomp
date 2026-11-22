//! `scene::set_object_light`: the light that falls on the object as a whole, to be added to
//! each of its vertices.
//!
//! The object's `light_added` (what `scene::vertex_light` adds) = 0, then by the light kind:
//! - 5: plain white;
//! - 1: the light strength = 0; the lamps gathered; the light colour plus `other_light`;
//! - 2: the same, with the strength = 0 only while the debug window's switch
//!   (`CLEAR_KIND_2_STRENGTH`) is on;
//! - 3: the same without touching the strength, and with `drawn_car_light` = the drawn car
//!   while the lamps are gathered (a car is not lit by its own), -1 afterwards;
//! - anything else: nothing more.
//!
//! "The lamps gathered" (`scene::gather_lights`, inlined): the object's `place` to the lit place,
//! the light colour = 0, `scene::add_car_lights`, `scene::add_point_lights`. "Plus" is byte by byte
//! without overflow, as in `scene::vertex_light`.
//!
//! The kind's way is in `rc2k_core::scene::lamps::object_light`.

use crate::errors::Refused;
use crate::scene::add_car_light::add_car_light;
use crate::scene::add_car_lights::add_car_lights;
use crate::scene::add_point_lights::add_point_lights;
use crate::scene::vertex_light::plus;

// ---- game data ----

/// The developers' switch for light kind 2: its objects' light strength cleared, as kind 1's
/// always is. Set from the debug window's switches tab; off otherwise. (It was a key held in
/// the race.)
pub static CLEAR_KIND_2_STRENGTH: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// Logic: `rc2k_core::scene::lamps::object_light`
pub fn set_object_light(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), Refused> {
    let race = ctx.race;
    let (light, object, vars) = (&ctx.view.light, &ctx.view.object, &ctx.view.vars);
    use rc2k_core::scene::lamps::{ObjectLight, WHITE, object_light};
    light.borrow_mut().light_added = 0u32;
    let kind = light.borrow().light_kind;
    let (clear_strength, own_car) = match object_light(kind, || {
        CLEAR_KIND_2_STRENGTH.load(std::sync::atomic::Ordering::Relaxed)
    }) {
        ObjectLight::White => {
            light.borrow_mut().light_added = WHITE;
            return Ok(());
        }
        ObjectLight::Gathered {
            clear_strength,
            own_car,
        } => (clear_strength, own_car),
        ObjectLight::Nothing => return Ok(()),
    };
    if clear_strength {
        object.borrow_mut().light_strength = 0u32;
    }
    let place = object.borrow().place;
    light.borrow_mut().lit_place = place;
    if own_car {
        light.borrow_mut().drawn_car_light = vars.borrow().drawn_car;
    }
    light.borrow_mut().light_colour = 0u32;
    let drawn_car = ctx.view.light.borrow().drawn_car_light;
    add_car_lights(
        ctx.reads,
        ctx.reads.net_race.get(),
        drawn_car,
        ctx.view.drawn_view(race),
        &mut |car, marks| {
            add_car_light(
                ctx.reads.course(),
                car,
                marks,
                ctx.reads.light_tables,
                &ctx.view.light,
            );
            Ok(())
        },
    )?;
    add_point_lights(ctx)?;
    if own_car {
        light.borrow_mut().drawn_car_light = u32::MAX;
    }
    {
        let value = plus(light.borrow().light_colour, light.borrow().other_light);
        light.borrow_mut().light_added = value;
    };
    Ok(())
}
