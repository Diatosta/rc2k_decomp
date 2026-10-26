//! The cars of a network race (converted 2026-10-21 from `net::car_list` 0x4AAAF7,
//! `net::set_hud_options` 0x463D3F, `net::reset_cars` 0x463E52 and
//! `net::clear_car_state` 0x463EB6): the car list offered, the HUD's bits, and the cars'
//! blocks made ready (offsets into a car's block).

use crate::Invalid;

/// The longest list looked through before it is taken to have no end.
const MOST: u32 = 0x100;
/// A list's end.
pub const END: u32 = u32::MAX;

/// Whether one of the cars won (`won`, a list ended by -1, its first entry looked at
/// whatever it is) is in the secret cars' list (`secret`, likewise; each won car compared
/// with its first entry before its end is looked for). The lists by index. Refused: a list
/// with no end in 0x100 entries.
pub fn any_won(secret: impl Fn(u32) -> u32, won: impl Fn(u32) -> u32) -> Result<bool, Invalid> {
    let mut w = 0u32;
    for _ in 0..MOST {
        let car = won(w);
        let mut s = 0u32;
        let mut ended = false;
        for _ in 0..MOST {
            if secret(s) == car {
                return Ok(true);
            }
            s += 1;
            if secret(s) == END {
                ended = true;
                break;
            }
        }
        if !ended {
            return Err(Invalid("a secret cars' list with no end"));
        }
        w += 1;
        if won(w) == END {
            return Ok(false);
        }
    }
    Err(Invalid("a won cars' list with no end"))
}

/// The HUD's bits kept from the last race; this value: none kept.
pub const NOT_KEPT: u32 = u32::MAX;

/// The HUD's bits each of the player's nine switches gives when not 0, in the original's
/// order (`dword_75ADF8`, `75ADFC`, `75AE00`, `75AE08`, `75AE14`, `75AE04`, `75AE10`,
/// `75AE0C`, `75AE18`).
pub const HUD_SWITCH_BITS: [u32; 9] = [0x10, 8, 1, 4, 2, 0x100, 0x40, 0x20, 0x80];

/// The HUD's bits made from the switches' values.
pub fn hud_bits(switches: [u32; 9]) -> u32 {
    switches
        .iter()
        .zip(HUD_SWITCH_BITS)
        .filter(|(on, _)| **on != 0)
        .fold(0, |bits, (_, bit)| bits | bit)
}

/// A car's block's dwords set before it is cleared (`reset_cars`), in order: +0xAA8 the
/// car's number + 1, +0x684 4, +0x28 2, +0x34, +0x30, +0x20, +0x3C 0x80.
pub fn reset_fields(car: u32) -> [(u32, u32); 7] {
    [
        (0xAA8, car + 1),
        (0x684, 4),
        (0x28, 2),
        (0x34, 0x80),
        (0x30, 0x80),
        (0x20, 0x80),
        (0x3C, 0x80),
    ]
}

/// The cars reset.
pub const CARS: u32 = 8;

/// A car's block's dwords cleared (`clear_car_state`), in the original's order, and what
/// goes in them: 51 of them 0, +0x298 -1.
pub const CLEARED: [(u32, u32); 52] = [
    (0x2B8, 0),
    (0x2BC, 0),
    (0x310, 0),
    (0x2C0, 0),
    (0x2C4, 0),
    (0x330, 0),
    (0x2C8, 0),
    (0x2CC, 0),
    (0x344, 0),
    (0x364, 0),
    (0x350, 0),
    (0x32C, 0),
    (0x2F4, 0),
    (0x30C, 0),
    (0x2D0, 0),
    (0x2D8, 0),
    (0x2DC, 0),
    (0x2E0, 0),
    (0x2E4, 0),
    (0x2E8, 0),
    (0x2EC, 0),
    (0x2F0, 0),
    (0x2F8, 0),
    (0x2FC, 0),
    (0x300, 0),
    (0x304, 0),
    (0x308, 0),
    (0x314, 0),
    (0x318, 0),
    (0x31C, 0),
    (0x320, 0),
    (0x324, 0),
    (0x328, 0),
    (0x334, 0),
    (0x338, 0),
    (0x33C, 0),
    (0x340, 0),
    (0x348, 0),
    (0x34C, 0),
    (0x354, 0),
    (0x358, 0),
    (0x35C, 0),
    (0x360, 0),
    (0x64, 0),
    (0x68, 0),
    (0x6C, 0),
    (0x70, 0),
    (0x74, 0),
    (0x78, 0),
    (0x7C, 0),
    (0x80, 0),
    (0x298, 0xFFFF_FFFF),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cars() {
        let secret = [5u32, 9, END];
        let list = |l: &'static [u32]| move |n: u32| l[n as usize];
        assert_eq!(any_won(list(&[5, 9, END]), list(&[1, 9, END])), Ok(true));
        assert_eq!(
            any_won(|n| secret[n as usize], list(&[1, 2, END])),
            Ok(false)
        );
        assert_eq!(
            any_won(|n| secret[n as usize], list(&[END, END])),
            Ok(false)
        );
        assert!(any_won(|_| 0, |_| 1).is_err());
        assert_eq!(
            hud_bits([1, 0, 1, 0, 0, 1, 0, 0, 1]),
            0x10 | 1 | 0x100 | 0x80
        );
        assert_eq!(hud_bits([0; 9]), 0);
        assert_eq!(reset_fields(3)[0], (0xAA8, 4));
        assert_eq!(CLEARED.iter().filter(|(_, v)| *v == 0).count(), 51);
    }
}
