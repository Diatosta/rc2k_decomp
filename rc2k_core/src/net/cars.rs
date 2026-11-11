//! The cars of a network race (from `net::car_list` 0x4AAAF7,
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

/// The cars reset.
pub const CARS: u32 = 8;

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
    }
}
