//! `scene::set_object_light`: the light that falls on the object as a whole, to be added to
//! each of its vertices.
//!
//! The object's `light_added` (what `scene::vertex_light` adds) = 0, then by the light kind:
//! - 5: plain white;
//! - 1: the light strength = 0; the lamps gathered; the light colour plus `other_light`;
//! - 2: the same, with the strength = 0 only while the key (a debugging key) is down;
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

/// `gather`: what stands for `scene::add_car_lights` and `scene::add_point_lights`.
// Logic: `rc2k_core::scene::lamps::object_light`
pub fn set_object_light(
    gather: &mut dyn FnMut() -> Result<(), Refused>,
    input: &crate::input::owner::Input,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
    vars: &crate::race::view_vars_store::ViewVarsCopy,
) -> Result<(), Refused> {
    use rc2k_core::scene::lamps::{ObjectLight, WHITE, object_light};
    light.light_added.set(0u32);
    let (clear_strength, own_car) = match object_light(light.light_kind.get(), || {
        input.devices.keys_now.get(0x18) != 0
    }) {
        ObjectLight::White => {
            light.light_added.set(WHITE);
            return Ok(());
        }
        ObjectLight::Gathered {
            clear_strength,
            own_car,
        } => (clear_strength, own_car),
        ObjectLight::Nothing => return Ok(()),
    };
    if clear_strength {
        object.light_strength.set(0u32);
    }
    crate::memory::session::write_bits(
        &light.lit_place,
        crate::memory::session::read_bits::<_, 3>(&object.place),
    );
    if own_car {
        light.drawn_car_light.set(vars.drawn_car.get());
    }
    light.light_colour.set(0u32);
    gather()?;
    if own_car {
        light.drawn_car_light.set(u32::MAX);
    }
    light
        .light_added
        .set(plus(light.light_colour.get(), light.other_light.get()));
    Ok(())
}

pub fn set_object_light_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
) -> Result<(), Refused> {
    let race = ctx.race;
    set_object_light(
        &mut || {
            add_car_lights(
                ctx.reads,
                ctx.reads.net_race.get(),
                ctx.view.light.drawn_car_light.get(),
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
            Ok(())
        },
        ctx.input,
        &ctx.view.light,
        &ctx.view.object,
        &ctx.view.vars,
    )
}
