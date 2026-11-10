//! One car's force feedback for one device (converted 2026-10-20 from
//! `input::update_car_effects` 0x40F347): from the car's physics (its view block) into its
//! sound object's feedback fields, and six effect requests, all "set" (`input::set_effect`
//! keeps every register, so each request's numbers are what this plans).

use crate::Invalid;
use crate::math::geometry::fistp64;

/// The sound object's feedback fields (their offsets in the object).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Feel {
    /// +0x14: the engine revs (the sound object's `revs`); their square >> 19 is the rumble when the car is nearly still.
    pub revs: u32,
    /// +0xAC: 1 while the car is off the ground, it seems (the view's +0x248).
    pub airborne: u32,
    /// +0xB0 .. +0xBC: the view's figures times 10, as whole numbers.
    pub figures: [u32; 4],
    /// +0xC0: the knock.
    pub knock: u32,
    /// +0xC4 .. +0xD0: a hit's figures (floats' bits), cleared each time.
    pub hit: [u32; 4],
    /// +0xD8: the strength the effects are set with.
    pub strength: u32,
    /// +0xDC: counts to 4 after a knock before the next.
    pub knock_count: u32,
    /// +0x278: 1 after a landing.
    pub landing: u32,
    /// +0x27C: +0xBC as it was last time.
    pub last_figure: u32,
    /// +0x284: the speed (signed).
    pub speed: u32,
    /// +0x294: the surface's number.
    pub surface: u32,
}

/// What is read of the view block: +0x248, and the floats at +0x1A4, +0x1AC, +0x1B8.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub airborne: u32,
    pub a4: f32,
    pub ac: f32,
    pub b8: f32,
}

/// The game's constants: 10, the angle's scale (162.97), 256, 0.1, and the effect kind of
/// each of the 16 surfaces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constants {
    pub ten: f32,
    pub angle_scale: f32,
    pub full: f32,
    pub tenth: f32,
    pub kinds: [u32; 16],
}

/// A request to set effect `effect` with the two numbers `a` and `b` (the parameters of
/// `input::set_effect`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub effect: u32,
    pub a: u32,
    pub b: u32,
}

/// The requests in order, and the knock's angle (the bits `g_soundScratch` gets) if one
/// was worked out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub requests: Vec<Request>,
    pub angle: Option<u32>,
}

/// `fpatan`: the angle of (x, y), as the stack keeps it.
fn fpatan(y: f32, x: f32) -> f64 {
    crate::math::portable::atan2(y as f64, x as f64)
}

/// The feedback of one call. Refused (before anything): a surface number past the 16.
pub fn update_car_effects(feel: &mut Feel, view: &View, k: &Constants) -> Result<Plan, Invalid> {
    if feel.surface >= 16 {
        return Err(Invalid("a surface number past the effect kinds' 16"));
    }
    feel.airborne = view.airborne;
    // On the stack throughout: `f64`, rounded by `fistp`.
    let (a4, ac, b8, ten) = (view.a4 as f64, view.ac as f64, view.b8 as f64, k.ten as f64);
    feel.figures = [
        fistp64(a4 * ten) as u32,
        fistp64((b8 - ac) * ten) as u32,
        fistp64(a4 * ten) as u32,
        fistp64(ac * ten) as u32,
    ];
    let mut angle = None;
    let [c4, _, cc, d0] = feel.hit;
    // A knock when +0xD0 and +0xC4 are not plus or minus zero.
    if d0 & 0x7FFF_FFFF != 0 && c4 & 0x7FFF_FFFF != 0 {
        let a = fpatan(f32::from_bits(cc), f32::from_bits(c4));
        // `fst`: the angle stored rounded; the stack goes on with the whole one.
        angle = Some((a as f32).to_bits());
        let scaled = a.abs() * k.angle_scale as f64;
        let d = f32::from_bits(d0) as f64;
        feel.knock = fistp64((k.full as f64 - scaled) * d * k.tenth as f64) as u32;
    } else {
        feel.knock = 0;
    }
    feel.hit = [0; 4];

    // The surface's effect kind and the rumble's strength (the speed, or at a crawl the
    // low-speed rumble).
    let mut kind = k.kinds[feel.surface as usize];
    let mut rumble = feel.speed;
    if (rumble as i32) < 0 {
        rumble = rumble.wrapping_neg();
    }
    if rumble <= 5 {
        kind = 0;
        rumble = feel.revs.wrapping_mul(feel.revs) >> 19;
    }
    rumble = rumble.min(0x100);

    let mut requests = Vec::new();
    // Each request gives back its `b`, which the next may take on.
    let mut set = |effect: u32, a: u32, b: u32| {
        requests.push(Request { effect, a, b });
        b
    };
    let d8 = feel.strength;
    set(1, kind, rumble);
    let mut landed_now = false;
    // Each way's requests; the last one's `b` goes on to the next.
    let mut last = match kind {
        5 | 9 | 0 => {
            set(4, 0x100, d8);
            set(0, 0, d8)
        }
        7 => {
            set(4, 0x55, d8);
            set(0, 0x55, d8)
        }
        8 => {
            set(4, 0, d8);
            set(0, 0, d8)
        }
        _ => {
            set(4, 0, d8);
            if feel.airborne != 0 {
                feel.landing = 1;
                set(0, 0, d8);
                set(1, 0, 0x100);
                landed_now = true;
                set(4, 0, d8)
            } else if feel.landing == 1 {
                feel.landing = 0;
                let before = set(0, 0x100, d8);
                set(2, 0x100, before)
            } else {
                let mut a = (feel.figures[0] << 1).wrapping_add(feel.figures[1]) << 1;
                if (a as i32) < 0 {
                    a = a.wrapping_neg();
                }
                if (a as i32) > 0x14 {
                    set(0, 0x100 - (a - 0x14).min(0x100), d8)
                } else {
                    set(0, 0x100, d8)
                }
            }
        }
    };
    if !landed_now {
        // The knock.
        let mut counted = false;
        if feel.knock != 0 && feel.airborne == 0 {
            if feel.knock_count == 0 {
                let sign = if (feel.knock as i32) < 0 {
                    0xFFFF_FF00
                } else {
                    0x100
                };
                last = set(3, sign, last);
                feel.knock_count = feel.knock_count.wrapping_add(1);
                counted = true;
            } else {
                feel.knock = 0;
            }
        }
        if !counted && feel.knock_count != 0 {
            feel.knock_count = feel.knock_count.wrapping_add(1);
            if feel.knock_count >= 4 {
                feel.knock_count = 0;
            }
        }
    }
    // The change of +0xBC since last time, held to 0x200 either way, halved.
    let now = feel.figures[3];
    let change = (now.wrapping_sub(feel.last_figure) as i32).clamp(-0x200, 0x200) >> 1;
    feel.last_figure = now;
    set(5, change as u32, last);
    Ok(Plan { requests, angle })
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: Constants = Constants {
        ten: 10.0,
        angle_scale: 162.97,
        full: 256.0,
        tenth: 0.1,
        kinds: [1, 0, 5, 7, 8, 9, 2, 3, 4, 6, 1, 1, 1, 1, 1, 1],
    };

    #[test]
    fn plans() {
        let view = View {
            airborne: 0,
            a4: 1.5,
            ac: 2.0,
            b8: 3.0,
        };
        let mut feel = Feel {
            speed: (-50i32) as u32,
            strength: 0x80,
            surface: 3,
            ..Feel::default()
        };
        let plan = update_car_effects(&mut feel, &view, &K).unwrap();
        assert_eq!(feel.figures, [15, 10, 15, 20]);
        let effects: Vec<u32> = plan.requests.iter().map(|r| r.effect).collect();
        assert_eq!(effects, [1, 4, 0, 5]);
        assert_eq!(
            plan.requests[0],
            Request {
                effect: 1,
                a: 7,
                b: 50
            }
        );
        assert_eq!(
            plan.requests[3],
            Request {
                effect: 5,
                a: 10,
                b: 0x80
            }
        );
        // A knock: straight ahead, full strength 0.1 of 256 * d.
        feel.hit = [1.0f32.to_bits(), 0, 0, 10.0f32.to_bits()];
        let plan = update_car_effects(&mut feel, &view, &K).unwrap();
        assert_eq!(plan.angle, Some(0));
        assert_eq!(feel.knock, 256);
        assert_eq!(feel.hit, [0; 4]);
        assert!(plan.requests.iter().any(|r| r.effect == 3 && r.a == 0x100));
        feel.surface = 16;
        assert!(update_car_effects(&mut feel, &view, &K).is_err());
    }
}
