//! `scene::set_object_light` (0x45143A): the light that falls on the object as a whole,
//! to be added to each of its vertices.
//!
//! `dword_635068` (what `scene::vertex_light` adds) = 0, then by `g_lightKind`:
//! - 5: plain white;
//! - 1: `g_lightStrength` = 0; the lamps gathered; `g_lightColour` plus
//!   `dword_635054`;
//! - 2: the same, with `g_lightStrength` = 0 only while the key `byte_4F6B74` is down;
//! - 3: the same without touching the strength, and with `dword_639968` =
//!   `dword_60627C` while the lamps are gathered (a car is not lit by its own), -1
//!   afterwards;
//! - anything else: nothing more.
//!
//! "The lamps gathered" (`scene::gather_lights`, 0x451425, inlined): the object's place
//! `dword_63501C` .. to `g_litPlace`, `g_lightColour` = 0, `scene::add_car_lights`,
//! `scene::add_point_lights`. "Plus" is byte by byte without overflow, as in
//! `scene::vertex_light`. One caller, `scene::draw_object`.
//!
//! Converted 2026-10-21: the kind's way through `rc2k_core::scene::lamps::object_light`.

use crate::errors::Refused;
use crate::input::owner::input;
use crate::scene::add_car_light::add_car_light;
use crate::scene::add_car_lights::add_car_lights;
use crate::scene::add_point_lights::add_point_lights;
use crate::scene::vertex_light::plus;

// ---- game data ----

// ---- Rust version ----

/// `gather`: what stands for `scene::add_car_lights` and `scene::add_point_lights`.
// Converted: `rc2k_core::scene::lamps::object_light`
pub fn set_object_light(
    race: &crate::race::owner::Race,
    gather: &mut dyn FnMut() -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    use rc2k_core::scene::lamps::{ObjectLight, WHITE, object_light};
    light.light_added.set_bits(0u32);
    let (clear_strength, own_car) = match object_light(light.light_kind.bits(), || {
        input().devices.keys_now.get(0x18) != 0 // byte_4F6B74
    }) {
        ObjectLight::White => {
            light.light_added.set_bits(WHITE);
            return Ok(());
        }
        ObjectLight::Gathered {
            clear_strength,
            own_car,
        } => (clear_strength, own_car),
        ObjectLight::Nothing => return Ok(()),
    };
    if clear_strength {
        object.light_strength.set_bits(0u32);
    }
    crate::session::write_bits(
        &light.lit_place,
        crate::session::read_bits::<_, 3>(&object.place),
    );
    if own_car {
        race.physics
            .ground
            .drawn_car_light
            .set_bits(race.frame.view_and_records.view_vars.drawn_car.bits());
    }
    light.light_colour.set_bits(0u32);
    gather()?;
    if own_car {
        race.physics.ground.drawn_car_light.set_bits(u32::MAX);
    }
    let mmx = system.session.machine.has_mmx.get() != 0;
    light.light_added.set_bits(plus(
        light.light_colour.bits(),
        scene.drawing.object_work.other_light.bits(),
        mmx,
    ));
    Ok(())
}

/// With the lamps' Rust versions.
pub fn set_object_light_in_game(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    set_object_light(
        race,
        &mut || {
            add_car_lights(race, &mut |car, marks| {
                add_car_light(&race.course, car, marks, scene, &ctx.view.light, system);
                Ok(())
            })?;
            add_point_lights(ctx, system)?;
            Ok(())
        },
        scene,
        &ctx.view.light,
        &ctx.view.object,
        system,
    )
}
