//! The texts the game's log writes (converted 2026-10-20 from `logfile::log_int` 0x401440,
//! `log_int_fmt` 0x40148B, `log_float` 0x4014D8, `date_text` 0x401D74; the entry's gate of
//! `logfile::log_write` 0x4013CB converted 2026-10-21).

use crate::strings::format::{Pad, decimal};
use crate::strings::table::nth_string;

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

/// What `logfile::log_write` does with a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// Logging is off (`g_logEnabled` 0 or less): nothing changes.
    Off,
    /// The log level is 0: the original's lock function returns straight to the caller's
    /// caller (eax left the code address after its call).
    Skipped,
    /// The lock taken and the line written.
    Written,
}

/// `logfile::log_write`'s gate: `enabled` (signed) first, then the level (asked only
/// then).
pub fn entry(enabled: i32, level: impl FnOnce() -> u32) -> Entry {
    if enabled <= 0 {
        Entry::Off
    } else if level() == 0 {
        Entry::Skipped
    } else {
        Entry::Written
    }
}

/// The most bytes a log line keeps: bytes are stored while the line is no longer (the line
/// buffer 0x93E080 .. 0x93E42C).
pub const LINE_LIMIT: usize = 0x93E42C - 0x93E080;

/// The line `logfile::write_line` (0x40132C) builds and the new line-start flag: at a
/// line's start (flag 1) `indent` spaces (0 .. 80) first, then the message (cut where the
/// line passes `LINE_LIMIT`), then "\r\n" (a new line starts) or a space.
pub fn format_line(msg: &[u8], newline: bool, indent: i32, line_start: u32) -> (Vec<u8>, u32) {
    let mut out = Vec::new();
    let mut flag = line_start;
    if flag == 1 {
        flag = 0;
        out.resize(indent.clamp(0, 80) as usize, b' ');
    }
    for &c in msg {
        if out.len() > LINE_LIMIT {
            break;
        }
        out.push(c);
    }
    if newline {
        out.extend_from_slice(b"\r\n");
        flag = 1;
    } else {
        out.push(b' ');
    }
    (out, flag)
}

/// One of the log's calls: a text, a signed number (`log_int`), eight hex digits
/// (`log_hex8`); each with its newline flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Text(Vec<u8>),
    Int(i32),
    Hex8(u32),
}

/// What `start` has found out by the log's start (`logfile::start_log` 0x401135).
#[derive(Clone, Debug, Default)]
pub struct Machine {
    /// `logfile::date_text`'s text, the version (`byte_4E0050`) and a second text if any.
    pub date: Vec<u8>,
    pub version: Vec<u8>,
    pub second: Option<Vec<u8>>,
    /// The processor's family, flags, name; clock, FPU, MMX, 3DNow, SSE.
    pub cpu_family: i32,
    pub cpu_flags: u32,
    pub cpu_name: Vec<u8>,
    pub clock: i32,
    pub fpu: i32,
    pub mmx: i32,
    pub amd_3d: i32,
    pub katmai: i32,
    /// The keyboard type.
    pub machine_type: i32,
    /// Windows' version: its platform, major, minor, build (16 bits) and text.
    pub platform: [i32; 4],
    pub platform_text: Vec<u8>,
    /// The memory in bytes: physical available and total, page file available and total.
    pub memory: [u32; 4],
    /// The current drive's free space in Kb (0xFFFFFFFF unknown).
    pub free_kb: u32,
}

/// The log's head, in the original's calls.
pub fn head(m: &Machine) -> Vec<(Message, bool)> {
    use Message::*;
    let t = |s: &[u8]| Text(s.to_vec());
    let mut out = vec![
        (t(b"Log started"), false),
        (Text(m.date.clone()), true),
        (Text(m.version.clone()), true),
    ];
    if let Some(second) = &m.second {
        out.push((Text(second.clone()), true));
    }
    out.push((t(b""), true));
    out.extend([
        (t(b"cpu="), false),
        (Int(m.cpu_family), false),
        (Hex8(m.cpu_flags), false),
        (Text(m.cpu_name.clone()), true),
    ]);
    for (name, v) in [
        (&b"clk="[..], m.clock),
        (b"fpu=", m.fpu),
        (b"mmx=", m.mmx),
        (b"amdk3d=", m.amd_3d),
        (b"katmai=", m.katmai),
        (b"machinetype=", m.machine_type),
    ] {
        out.extend([(t(name), false), (Int(v), true)]);
    }
    out.push((t(b"platform="), false));
    out.extend(m.platform.iter().map(|&v| (Int(v), false)));
    out.push((Text(m.platform_text.clone()), true));
    let kb = |b: u32| Int((b >> 10) as i32);
    out.extend([
        (t(b"ram avail/total:"), false),
        (kb(m.memory[0]), false),
        (kb(m.memory[1]), false),
        (t(b"Kb"), true),
        (t(b"swp avail/total:"), false),
        (kb(m.memory[2]), false),
        (kb(m.memory[3]), false),
        (t(b"Kb"), true),
        (t(b"free space on partition="), false),
        (Int(m.free_kb as i32), false),
        (t(b"Kb"), true),
        (t(b""), true),
        (t(b""), true),
    ]);
    out
}

/// The free space in Kb from the drive's numbers (`files::drive_free_space_kb`): the cluster's
/// bytes as 32 bits times the free clusters; 0xFFFFFFFF from 2^42 bytes on.
pub fn free_kb(cluster_sectors: u32, sector_bytes: u32, free_clusters: u32) -> u32 {
    let bytes = cluster_sectors.wrapping_mul(sector_bytes) as u64 * free_clusters as u64;
    if bytes >> 42 != 0 {
        u32::MAX
    } else {
        (bytes >> 10) as u32
    }
}

#[cfg(test)]
mod head_tests {
    use super::*;

    #[test]
    fn lines() {
        assert_eq!(format_line(b"ab", true, 2, 1), (b"  ab\r\n".to_vec(), 1));
        assert_eq!(format_line(b"ab", false, 2, 0), (b"ab ".to_vec(), 0));
        let h = head(&Machine::default());
        assert_eq!(h[0], (Message::Text(b"Log started".to_vec()), false));
        assert_eq!(h.len(), 39);
        let second = Machine {
            second: Some(b"x".to_vec()),
            ..Machine::default()
        };
        assert_eq!(head(&second)[3], (Message::Text(b"x".to_vec()), true));
        assert_eq!(free_kb(8, 512, 4), 16);
        assert_eq!(free_kb(0x1000, 0x1000, 0x40000), u32::MAX);
        // The cluster's bytes wrap in 32 bits, as the original's.
        assert_eq!(free_kb(0x10000, 0x10000, 0x400), 0);
    }

    #[test]
    fn gate() {
        assert_eq!(entry(0, || unreachable!()), Entry::Off);
        assert_eq!(entry(-1, || unreachable!()), Entry::Off);
        assert_eq!(entry(1, || 0), Entry::Skipped);
        assert_eq!(entry(10, || 2), Entry::Written);
    }
}

/// The log's state: the game keeps it in `g_logEnabled`, `g_logLevel`, `g_logIndent` and
/// `g_logAtLineStart` (0x4E0094 .. 0x4E00A4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogState {
    /// On while above 0 (signed).
    pub enabled: i32,
    /// The highest level logged; 0 logs nothing.
    pub level: u32,
    /// Spaces before a line (0 .. 80 used).
    pub indent: i32,
    /// 1 when the next text starts a line.
    pub line_start: u32,
}

/// What one call of the log does with a text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Logged {
    /// Logging is off or its level is 0: the lock not taken, nothing changed.
    Off,
    /// The text's level is above the log's: the lock taken and given back, nothing else.
    Above,
    /// The line to append to the log's file (the state's line-start flag set for the next).
    Line(Vec<u8>),
}

/// One text logged at `level` (1 or 2; the original adds 1 to `level - 1` as a byte, so 0
/// is 256), with a line break after it if `newline`: `log_write` and `write_line` together.
pub fn log_line(state: &mut LogState, text: &[u8], newline: bool, level: u8) -> Logged {
    if entry(state.enabled, || state.level) != Entry::Written {
        return Logged::Off;
    }
    let level = match level {
        0 => 256,
        n => n as u32,
    };
    if state.level < level {
        return Logged::Above;
    }
    let (line, flag) = format_line(text, newline, state.indent, state.line_start);
    state.line_start = flag;
    Logged::Line(line)
}

#[cfg(test)]
mod line_tests {
    use super::*;

    fn on(level: u32) -> LogState {
        LogState {
            enabled: 1,
            level,
            indent: 2,
            line_start: 1,
        }
    }

    #[test]
    fn gates() {
        let mut off = LogState {
            enabled: 0,
            ..on(2)
        };
        assert_eq!(log_line(&mut off, b"x", true, 1), Logged::Off);
        let mut negative = LogState {
            enabled: -1,
            ..on(2)
        };
        assert_eq!(log_line(&mut negative, b"x", true, 1), Logged::Off);
        let mut silent = on(0);
        assert_eq!(log_line(&mut silent, b"x", true, 1), Logged::Off);
        let mut one = on(1);
        assert_eq!(log_line(&mut one, b"x", true, 2), Logged::Above);
        assert_eq!(one, on(1));
        // level 0 is 256: above any log level the game sets
        assert_eq!(log_line(&mut on(255), b"x", true, 0), Logged::Above);
    }

    #[test]
    fn lines_and_indent() {
        let mut s = on(2);
        assert_eq!(
            log_line(&mut s, b"cpu=", false, 1),
            Logged::Line(b"  cpu= ".to_vec())
        );
        assert_eq!(s.line_start, 0);
        assert_eq!(
            log_line(&mut s, b"6", true, 2),
            Logged::Line(b"6\r\n".to_vec())
        );
        assert_eq!(s.line_start, 1);
        assert_eq!(
            log_line(&mut s, b"", true, 1),
            Logged::Line(b"  \r\n".to_vec())
        );
    }
}
