//! The texts the game's log writes (converted 2026-10-20 from `logfile::log_int` 0x401440,
//! `log_int_fmt` 0x40148B, `log_float` 0x4014D8, `date_text` 0x401D74).

use crate::format::{Pad, decimal};
use crate::strings::nth_string;

/// A signed number: a minus sign before the magnitude, padded as `pad` says.
pub fn signed(value: i32, pad: Pad) -> Vec<u8> {
    let mut text = Vec::new();
    if value < 0 {
        text.push(b'-');
    }
    text.extend(decimal(value.unsigned_abs(), pad));
    text
}

/// A float's text as the log makes it: the bytes, and where its NUL goes (after
/// `decimals` digits of the fraction, which may be past the bytes: the original's buffer
/// then shows what it held before).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloatText {
    pub bytes: Vec<u8>,
    pub nul_at: usize,
}

/// A float (its bits) with `decimals` digits: always a sign, "+" or "-", then the whole
/// part, a dot and nine digits, cut at `decimals`. Taken apart by hand as the original
/// does: the mantissa shifted into a 32.32 fixed-point value, the fraction's digits the
/// sum of 500000000 >> n for each set bit n (each rounded down, so the last digits can be a
/// little low). Zero of either sign is "=0." and the digits; 2^32 or more the sign and
/// ">=2^32"; then the sign and "infinity" or "nan".
pub fn float(bits: u32, decimals: u32) -> FloatText {
    let complete = |bytes: Vec<u8>| FloatText {
        nul_at: bytes.len(),
        bytes,
    };
    let (lead, whole, fraction) = if bits & 0x7FFF_FFFF == 0 {
        (b'=', 0, 0)
    } else {
        let sign = if (bits as i32) < 0 { b'-' } else { b'+' };
        let special = |text: &[u8]| {
            let mut out = vec![sign];
            out.extend_from_slice(text);
            complete(out)
        };
        let exponent = (bits >> 23) & 0xFF;
        let mut whole = (exponent != 0) as u32; // the hidden bit; none for a denormal
        let mut fraction = (bits & 0x7F_FFFF) << 9;
        if exponent == 0xFF {
            return special(if fraction == 0 { b"infinity" } else { b"nan" });
        }
        let shift = exponent as i32 - 0x7F;
        for _ in 0..shift.max(0) {
            let carry = whole >> 31;
            whole = whole << 1 | fraction >> 31;
            fraction <<= 1;
            if carry != 0 {
                return special(b">=2^32");
            }
        }
        for _ in 0..(-shift).max(0) {
            fraction = fraction >> 1 | whole << 31;
            whole >>= 1;
        }
        (sign, whole, fraction)
    };
    let mut bytes = vec![lead];
    bytes.extend(decimal(whole, Pad::None));
    bytes.push(b'.');
    let nul_at = bytes.len().wrapping_add(decimals as usize);
    bytes.extend(decimal(fraction_digits(fraction), Pad::Zeros(9)));
    FloatText { bytes, nul_at }
}

/// Nine decimal digits for a 32-bit binary fraction, as the original adds them up.
fn fraction_digits(mut fraction: u32) -> u32 {
    let (mut step, mut sum) = (500_000_000u32, 0u32);
    while fraction != 0 {
        if fraction & 0x8000_0000 != 0 {
            sum += step;
        }
        step >>= 1;
        fraction <<= 1;
    }
    sum
}

/// The moment the game started (`GetLocalTime`'s, as the game keeps it at 0x51839F).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Started {
    /// 0 = Sunday.
    pub weekday: u8,
    pub day: u8,
    /// 1 = January.
    pub month: u8,
    pub year: u16,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub hundredths: u8,
}

/// RAL.EXE's tables at 0x401E23 .. 0x401EDE, one after another: for each day of the month
/// which ending it has, the endings, the months' names (from an empty one) and the
/// weekdays'.
const ENDING_OF_DAY: [u8; 32] = [
    3, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 0,
];
const ENDINGS: &[u8] = b"st\0nd\0rd\0th\0";
const MONTHS: &[u8] = b"\0January\0February\0March\0April\0May\0June\0July\0August\0September\0October\0November\0December\0";
const WEEKDAYS: &[u8] = b"Sunday\0Monday\0Tuesday\0Wednesday\0Thursday\0Friday\0Saturday\0";

/// "Monday 12th October 2026 14:49:56.46" (`date_text`); `None` for a weekday past 6, a
/// day past 31 or a month past 12 (the original reads on into what follows its tables).
pub fn date(at: &Started) -> Option<Vec<u8>> {
    if at.weekday > 6 || at.day > 31 || at.month > 12 {
        return None;
    }
    let two = Pad::Zeros(2);
    let mut text = nth_string(WEEKDAYS, at.weekday as i32).to_vec();
    text.push(b' ');
    text.extend(decimal(at.day as u32, Pad::None));
    text.extend(nth_string(ENDINGS, ENDING_OF_DAY[at.day as usize] as i32));
    text.push(b' ');
    text.extend(nth_string(MONTHS, at.month as i32));
    text.push(b' ');
    text.extend(decimal(at.year as u32, Pad::None));
    text.push(b' ');
    for (part, after) in [
        (at.hour, &b":"[..]),
        (at.minute, b":"),
        (at.second, b"."),
        (at.hundredths, b""),
    ] {
        text.extend(decimal(part as u32, two));
        text.extend_from_slice(after);
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[u8]) -> &str {
        std::str::from_utf8(v).unwrap()
    }

    #[test]
    fn numbers() {
        assert_eq!(signed(-42, Pad::None), b"-42");
        assert_eq!(signed(i32::MIN, Pad::None), b"-2147483648");
        assert_eq!(signed(7, Pad::Zeros(3)), b"007");
        let f = float(1.5f32.to_bits(), 2);
        assert_eq!(s(&f.bytes[..f.nul_at]), "+1.50");
        let f = float((-0.25f32).to_bits(), 3);
        assert_eq!(s(&f.bytes[..f.nul_at]), "-0.250");
        assert_eq!(float(0x8000_0000, 1).bytes[..3], *b"=0.");
        assert_eq!(float(f32::INFINITY.to_bits(), 2).bytes, b"+infinity");
        assert_eq!(float(f32::NAN.to_bits(), 2).bytes, b"+nan");
        assert_eq!(float(5e9f32.to_bits(), 2).bytes, b"+>=2^32");
        assert_eq!(float(1.0f32.to_bits(), 12).nul_at, 15);
    }

    #[test]
    fn dates() {
        let at = Started {
            weekday: 1,
            day: 12,
            month: 10,
            year: 2026,
            hour: 14,
            minute: 49,
            second: 56,
            hundredths: 46,
        };
        assert_eq!(
            s(&date(&at).unwrap()),
            "Monday 12th October 2026 14:49:56.46"
        );
        let first = Started { day: 1, ..at };
        assert!(s(&date(&first).unwrap()).contains("1st"));
        let late = Started { day: 23, ..at };
        assert!(s(&date(&late).unwrap()).contains("23rd"));
        assert_eq!(date(&Started { month: 13, ..at }), None);
        let zero = Started {
            day: 0,
            month: 0,
            ..at
        };
        assert!(s(&date(&zero).unwrap()).starts_with("Monday 0th  2026"));
    }
}
