//! The game's number and text formatting (from `strings::format_u32`,
//! `append_u32`, `append_clock`, `append_clock_padded`, `append_fraction`,
//! `append_percent`, `append_time`, `hex_append`, `bin_append`, `format_seal_code`,
//! `rot13_into_text`). Each gives the bytes the original appends (no NUL).

/// How a decimal number is padded (RAL.EXE 0x40204A's flags byte).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pad {
    /// No padding: the digits alone ("0" for zero).
    None,
    /// Exactly `width` characters, '0's before the digits (up to 15: past ten digits the
    /// original's buffer gives more '0's).
    Zeros(u8),
    /// `width` characters, spaces before the digits (a lone "0" for zero); a number wider
    /// than `width` is cut to its last `width` digits, as the original's.
    Spaces(u8),
}

impl Pad {
    /// From the original's flags byte: the low four bits the width, 0x10 zeros, 0x20 spaces
    /// (0x10 first); without either, no padding.
    pub fn from_flags(flags: u8) -> Self {
        let width = flags & 0x0F;
        if flags & 0x10 != 0 {
            Pad::Zeros(width)
        } else if flags & 0x20 != 0 {
            Pad::Spaces(width)
        } else {
            Pad::None
        }
    }
}

/// `value` in decimal, padded as `pad` says.
pub fn decimal(value: u32, pad: Pad) -> Vec<u8> {
    // The original's 16-byte buffer: five filler '0's, ten digits, the NUL.
    let mut buf = [b'0'; 16];
    buf[5..15].copy_from_slice(format!("{value:010}").as_bytes());
    match pad {
        Pad::Zeros(width) => buf[15 - (width as usize).min(15)..15].to_vec(),
        Pad::Spaces(width) => {
            let start = 15 - (width as usize).min(15);
            let mut s = start;
            while s < 15 && buf[s] == b'0' {
                buf[s] = b' ';
                s += 1;
            }
            if s == 15 && s > start {
                buf[14] = b'0'; // all zeros: keep a final '0'
            }
            buf[start..15].to_vec()
        }
        Pad::None => {
            let first = buf[5..15].iter().position(|&c| c != b'0').unwrap_or(9);
            buf[5 + first..15].to_vec()
        }
    }
}

/// Minutes and seconds (or hours and minutes) as "Q:RR".
pub fn clock(value: u32) -> Vec<u8> {
    let mut out = decimal(value / 60, Pad::None);
    out.push(b':');
    out.extend(decimal(value % 60, Pad::Zeros(2)));
    out
}

/// As `clock`, with a '0' before a quotient below 10 ("0Q:RR").
pub fn clock_padded(value: u32) -> Vec<u8> {
    let mut out = Vec::new();
    if ((value / 60) as i32) < 10 {
        out.push(b'0');
    }
    out.extend(clock(value));
    out
}

/// "first/second".
pub fn fraction(first: u32, second: u32) -> Vec<u8> {
    let mut out = decimal(first, Pad::None);
    out.push(b'/');
    out.extend(decimal(second, Pad::None));
    out
}

/// "n%".
pub fn percent(value: u32) -> Vec<u8> {
    let mut out = decimal(value, Pad::None);
    out.push(b'%');
    out
}

/// A time in hundredths of a second as "HH:MM:SS.hh", the hours only when asked or when
/// there are some.
pub fn time(hundredths: u32, show_hours: bool) -> Vec<u8> {
    const HOUR: u32 = 360_000;
    const MINUTE: u32 = 6_000;
    const SECOND: u32 = 100;
    let two = Pad::Zeros(2);
    let mut out = Vec::new();
    let mut rest = hundredths;
    if show_hours || rest >= HOUR {
        out.extend(decimal(rest / HOUR, two));
        out.push(b':');
        rest %= HOUR;
    }
    out.extend(decimal(rest / MINUTE, two));
    out.push(b':');
    rest %= MINUTE;
    out.extend(decimal(rest / SECOND, two));
    out.push(b'.');
    out.extend(decimal(rest % SECOND, two));
    out
}

/// The low `digits` hex digits of `value`, upper case, most significant first; `None` for
/// more than 8 (no caller asks for them: `hex_append`'s callers use 2 and 8; the
/// original's result for more is not modelled).
pub fn hex(value: u32, digits: u32) -> Option<Vec<u8>> {
    const DIGITS: &[u8; 16] = b"0123456789ABCDEF";
    (digits <= 8).then(|| {
        (0..digits)
            .rev()
            .map(|i| DIGITS[((value >> (i * 4)) & 0xF) as usize])
            .collect()
    })
}

/// The low `digits` bits of `value` as '0' and '1', most significant first; `None` for
/// more than 32 (the original reads below its buffer).
pub fn binary(value: u32, digits: u32) -> Option<Vec<u8>> {
    (digits <= 32).then(|| {
        (0..digits)
            .rev()
            .map(|i| b'0' + ((value >> i) & 1) as u8)
            .collect()
    })
}

/// A replay's seal code: two dwords as "AABB-CCDD-EEFF-GGHH", each dword's bytes lowest
/// first.
pub fn seal_code(a: u32, b: u32) -> Vec<u8> {
    let mut out = Vec::new();
    for (n, v) in [a, b].into_iter().enumerate() {
        if n > 0 {
            out.push(b'-');
        }
        for k in 0..4 {
            if k == 2 {
                out.push(b'-');
            }
            out.extend(hex(v >> (8 * k), 2).unwrap_or_default());
        }
    }
    out
}

/// One byte coded by rot13 (letters 13 on, the rest as they are).
pub fn rot13(b: u8) -> u8 {
    match b {
        b'A'..=b'Z' => (b - b'A' + 13) % 26 + b'A',
        b'a'..=b'z' => (b - b'a' + 13) % 26 + b'a',
        _ => b,
    }
}

/// The name of a controller axis as the controls screen appends it: " (+x)", " (-ry)"; the
/// 0x10 bit is the direction, 0 .. 2 are x, y, z and 3 .. 5 their turns.
pub fn axis_name(axis: u8) -> Vec<u8> {
    let (sign, rest) = if axis & 0x10 != 0 {
        (b'+', axis.wrapping_sub(0x10))
    } else {
        (b'-', axis)
    };
    let mut out = b" (".to_vec();
    out.push(sign);
    let rest = if rest >= 3 {
        out.push(b'r');
        rest.wrapping_sub(3)
    } else {
        rest
    };
    out.push(rest.wrapping_add(b'x'));
    out.push(b')');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: Vec<u8>) -> String {
        String::from_utf8(v).unwrap()
    }

    #[test]
    fn decimals() {
        assert_eq!(s(decimal(0, Pad::None)), "0");
        assert_eq!(s(decimal(4_294_967_295, Pad::None)), "4294967295");
        assert_eq!(s(decimal(7, Pad::Zeros(2))), "07");
        assert_eq!(s(decimal(123, Pad::Zeros(2))), "23");
        assert_eq!(s(decimal(5, Pad::Zeros(12))), "000000000005");
        assert_eq!(s(decimal(42, Pad::Spaces(5))), "   42");
        assert_eq!(s(decimal(0, Pad::Spaces(3))), "  0");
        assert_eq!(s(decimal(12345, Pad::Spaces(3))), "345");
        assert_eq!(Pad::from_flags(0x12), Pad::Zeros(2));
        assert_eq!(Pad::from_flags(0x34), Pad::Zeros(4));
        assert_eq!(Pad::from_flags(0x25), Pad::Spaces(5));
        assert_eq!(Pad::from_flags(0x05), Pad::None);
    }

    #[test]
    fn spaces_of_width_zero_give_nothing() {
        assert_eq!(decimal(0, Pad::Spaces(0)), b"");
        assert_eq!(decimal(9, Pad::Zeros(0)), b"");
    }

    #[test]
    fn clocks_and_times() {
        assert_eq!(s(clock(125)), "2:05");
        assert_eq!(s(clock_padded(125)), "02:05");
        assert_eq!(s(clock_padded(725)), "12:05");
        assert_eq!(s(time(12_345, false)), "02:03.45");
        assert_eq!(s(time(12_345, true)), "00:02:03.45");
        assert_eq!(s(time(366_100, false)), "01:01:01.00");
        assert_eq!(s(time(0, false)), "00:00.00");
    }

    #[test]
    fn others() {
        assert_eq!(s(fraction(3, 4)), "3/4");
        assert_eq!(s(percent(99)), "99%");
        assert_eq!(s(hex(0xBEEF, 4).unwrap()), "BEEF");
        assert_eq!(hex(1, 9), None);
        assert_eq!(s(binary(5, 4).unwrap()), "0101");
        assert_eq!(binary(1, 33), None);
        assert_eq!(
            s(seal_code(0x4433_2211, 0x8877_6655)),
            "1122-3344-5566-7788"
        );
        assert_eq!(rot13(b'a'), b'n');
        assert_eq!(rot13(b'N'), b'A');
        assert_eq!(rot13(b'!'), b'!');
        assert_eq!(s(axis_name(0x10)), " (+x)");
        assert_eq!(s(axis_name(4)), " (-ry)");
        assert_eq!(s(axis_name(0x12)), " (+z)");
    }
}
