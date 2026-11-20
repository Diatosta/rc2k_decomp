//! The lamps that light an object: the scene's lamps by their distance, which cars' lamps
//! count, which colours the light tables are made from, and the light by the light's kind.

use crate::scene::distance::approximate;

/// Two colours added byte by byte without overflow.
pub fn brighter(a: u32, b: u32) -> u32 {
    (0..4).fold(a, |out, n| {
        let at = n * 8;
        let sum = ((a >> at & 0xFF) + (b >> at & 0xFF)).min(0xFF);
        out & !(0xFF << at) | sum << at
    })
}

/// A lamp of the scene: a point of the scene, a strength, a colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lamp {
    pub point: u32,
    pub strength: u32,
    pub colour: u32,
}

/// What a lamp gives at a distance: (0x20_0000 less it) times the strength over 0x20000,
/// held to 0 ..= 255.
pub fn amount(distance: u32, strength: u32) -> u32 {
    let near = 0x20_0000i32.wrapping_sub(distance as i32);
    (near.wrapping_mul(strength as i32) >> 17).clamp(0, 0xFF) as u32
}

/// The amount in each of the colour's three parts, each times the part over 256.
pub fn tinted(amount: u32, colour: u32) -> u32 {
    [0, 8, 16].into_iter().fold(0, |out, shift| {
        out | ((amount * (colour >> shift & 0xFF)) >> 8) << shift
    })
}

/// What `add_point_lights` leaves: the last lamp's strength, the last lit lamp's colour and
/// distance, and the light; None where nothing was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointLights {
    pub strength: Option<u32>,
    pub colour: Option<u32>,
    pub distance: Option<u32>,
    pub light: Option<u32>,
}

/// Each lamp of a strength not 0 adds its tinted amount at its approximate distance from
/// `place` (its point's place `point(n)`, asked only for those) to the light, `brighter`.
pub fn add_point_lights(
    lamps: impl IntoIterator<Item = Lamp>,
    place: [u32; 3],
    point: impl Fn(u32) -> [u32; 3],
    mut light: u32,
) -> PointLights {
    let mut out = PointLights::default();
    for lamp in lamps {
        out.strength = Some(lamp.strength);
        if lamp.strength == 0 {
            continue;
        }
        out.colour = Some(lamp.colour);
        let at = point(lamp.point);
        let distance =
            approximate(std::array::from_fn(|k| at[k].wrapping_sub(place[k]) as i32)).length;
        out.distance = Some(distance);
        light = brighter(tinted(amount(distance, lamp.strength), lamp.colour), light);
        out.light = Some(light);
    }
    out
}

/// What `add_car_lights` reads, each asked only where needed.
pub trait CarLights {
    /// Whether only one car's lamps are used.
    fn one_car(&self) -> bool;
    /// That one car.
    fn the_car(&self) -> u32;
    /// Whether every car's lamps are used.
    fn all_cars(&self) -> bool;
    /// The cars near.
    fn near(&self) -> u32;
    /// The view.
    fn view(&self) -> u32;
    /// The car being drawn.
    fn drawn(&self) -> u32;
    /// The car viewed.
    fn viewed(&self) -> u32;
}

/// Whether a car's lamps light the object: with one car, that car (or every car while all
/// count); else the cars near (or all) but not the one drawn, and of those all if one is
/// near or the view is 0 or 2, else only the car viewed.
pub fn car_lights(c: &impl CarLights, car: u32) -> bool {
    if c.one_car() {
        return car == c.the_car() || c.all_cars();
    }
    let near = c.near();
    let view = c.view();
    (c.all_cars() || car < near)
        && car != c.drawn()
        && (near == 1 || view == 0 || view == 2 || car == c.viewed())
}

/// The plain colours (both switches 0) or the scene's.
pub fn plain_light(switch: u32, other_switch: impl FnOnce() -> u32) -> bool {
    switch == 0 && other_switch() == 0
}

/// What the object's light does by the light's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectLight {
    /// 5: plain white (`WHITE`) added.
    White,
    /// 1, 2, 3: the lamps gathered and their colour plus the base added; the strength is
    /// cleared first (1 always, 2 while the key is down); with `own_car` (3) the drawn car is
    /// set to this one while they are gathered, -1 afterwards.
    Gathered { clear_strength: bool, own_car: bool },
    /// Anything else: nothing more.
    Nothing,
}

/// The white `ObjectLight::White` adds.
pub const WHITE: u32 = 0x00FF_FFFF;

/// The light's kind's way; `key_down` is asked for kind 2 only.
pub fn object_light(kind: u32, key_down: impl FnOnce() -> bool) -> ObjectLight {
    match kind {
        5 => ObjectLight::White,
        1 => ObjectLight::Gathered {
            clear_strength: true,
            own_car: false,
        },
        2 => ObjectLight::Gathered {
            clear_strength: key_down(),
            own_car: false,
        },
        3 => ObjectLight::Gathered {
            clear_strength: false,
            own_car: true,
        },
        _ => ObjectLight::Nothing,
    }
}

#[cfg(test)]
mod tests;
