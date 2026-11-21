//! The force feedback's requests: each of a device record's six effects takes a request dword:
//! what to do in the low byte (2 start, 3 set, 4 stop) and two signed 12-bit numbers above it.
//! The input thread hands each waiting change to the effect's function.

use crate::Invalid;

/// The number of effects a record has.
pub const EFFECTS: u32 = 6;
/// What a request does: start, set or stop an effect.
pub const START: u32 = 2;
pub(crate) const SET: u32 = 3;
pub const STOP: u32 = 4;

/// A request's dword: `kind | (a & 0xFFF) << 8 | (b & 0xFFF) << 20` (the kind not
/// masked).
pub fn request(kind: u32, a: u32, b: u32) -> u32 {
    kind | (a & 0xFFF) << 8 | (b & 0xFFF) << 20
}

/// Whether a request is written for a record with effects `on`: "set" only while the race runs.
/// Refused: an effect past 5.
pub fn wanted(on: bool, race_on: bool, effect: u32, kind: u32) -> Result<bool, Invalid> {
    if !on || (!race_on && kind == SET) {
        return Ok(false);
    }
    if effect >= EFFECTS {
        return Err(Invalid("an effect past the record's six"));
    }
    Ok(true)
}

/// A change as its function gets it: the low byte, and the two numbers sign-extended.
pub fn change(request: u32) -> (u32, i32, i32) {
    (
        request & 0xFF,
        (request << 12) as i32 >> 20,
        request as i32 >> 20,
    )
}

/// One car's feedback on one device: the car (0 .. 7), the device's record, the view
/// block, and whether the device has effects (only then is it updated).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Load {
    pub car: u32,
    pub device: u32,
    pub view: u32,
    pub update: bool,
}

/// What decides the way: players A's and B's cars, one player, the two devices, the two
/// view blocks, whether the two devices are one player's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Players {
    pub cars: [u32; 2],
    pub one_player: bool,
    pub devices: [u32; 2],
    pub views: [u32; 2],
    pub one_players_devices: bool,
}

/// The loads, in order: car A on the first device; then car A again on the second when
/// the first has effects and both are one player's; else, unless one player, car B on the
/// second with its view. Refused (before any is used): a car past the eight sound objects,
/// a null device on the way taken.
pub fn loads(p: &Players, has_effects: impl Fn(u32) -> bool) -> Result<Vec<Load>, Invalid> {
    let checked = |car: u32, device: u32, view: u32| {
        if car >= 8 {
            return Err(Invalid("a car number past the eight sound objects"));
        }
        if device == 0 {
            return Err(Invalid("a null device on the way taken"));
        }
        Ok(Load {
            car,
            device,
            view,
            update: has_effects(device),
        })
    };
    let first = checked(p.cars[0], p.devices[0], p.views[0])?;
    let mut out = vec![first];
    if first.update && p.one_players_devices {
        out.push(checked(p.cars[0], p.devices[1], p.views[0])?);
    } else if !p.one_player {
        out.push(checked(p.cars[1], p.devices[1], p.views[1])?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_and_loads() {
        assert_eq!(request(SET, 0x1FFF, 0xABC), 3 | 0xFFF << 8 | 0xABC << 20);
        assert_eq!(change(request(SET, 0xFFF, 0x7FF)), (3, -1, 0x7FF));
        assert_eq!(wanted(true, false, 2, SET), Ok(false));
        assert_eq!(wanted(true, false, 2, START), Ok(true));
        assert!(wanted(true, true, 6, START).is_err());
        let p = Players {
            cars: [1, 2],
            devices: [10, 20],
            views: [5, 6],
            ..Players::default()
        };
        let l = loads(&p, |d| d == 10).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(
            (l[1].car, l[1].device, l[1].view, l[1].update),
            (2, 20, 6, false)
        );
        let l = loads(
            &Players {
                one_players_devices: true,
                ..p
            },
            |_| true,
        )
        .unwrap();
        assert_eq!((l[1].car, l[1].view), (1, 5));
        assert_eq!(
            loads(
                &Players {
                    one_player: true,
                    ..p
                },
                |_| false
            )
            .unwrap()
            .len(),
            1
        );
        assert!(loads(&Players { cars: [8, 0], ..p }, |_| true).is_err());
    }
}
