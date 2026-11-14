//! The game's settings texts (`var\ral.cfg`, `var\name.cfg`): lines of `key=value` or `key value`.
//! Places are offsets into the text; a text ends at its NUL.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::case::CaseTable;

fn at(text: &(impl Bytes + ?Sized), place: usize) -> Result<u8, Invalid> {
    text.byte(place)
        .ok_or(Invalid("a settings text without its NUL"))
}

/// Where key `key`'s value starts, or `None` (`find_key`). A line is looked at from its first
/// character above a space; the key (given in lower case, compared through `case`) must be
/// there and be followed by `=` or a character of a space or less. The value starts at the first
/// character above a space after that, an `=` between them skipped once; a line that ends first
/// has its (empty) value at its line feed. A text that ends with the key itself is refused.
pub fn find_key(
    text: &(impl Bytes + ?Sized),
    key: &[u8],
    case: &CaseTable,
) -> Result<Option<usize>, Invalid> {
    let mut place = 0usize;
    let mut next = || -> Result<u8, Invalid> {
        let c = at(text, place)?;
        place += 1;
        Ok(c)
    };
    // Where the character `next` gave last was.
    macro_rules! last {
        () => {
            place - 1
        };
    }
    'line: loop {
        let mut c = loop {
            match next()? {
                0 => return Ok(None),
                c if c > b' ' => break c,
                _ => {}
            }
        };
        let mut k_at = 0usize;
        let key_left = loop {
            let k = key.get(k_at).copied().unwrap_or(0);
            k_at += 1;
            c = case.lower(c);
            if c != k {
                break k;
            }
            if k == 0 {
                return Err(Invalid("the settings text ends with the key"));
            }
            c = next()?;
        };
        let mut after_equals = c == b'=';
        if key_left != 0 || (!after_equals && c > b' ') {
            loop {
                match c {
                    0 => return Ok(None),
                    b'\n' => continue 'line,
                    _ => c = next()?,
                }
            }
        }
        while !after_equals {
            match c {
                0 => return Ok(None),
                b'\n' => return Ok(Some(last!())),
                b'=' => after_equals = true,
                c if c > b' ' => return Ok(Some(last!())),
                _ => c = next()?,
            }
        }
        loop {
            match next()? {
                0 => return Ok(None),
                c if c == b'\n' || c > b' ' => return Ok(Some(last!())),
                _ => {}
            }
        }
    }
}

/// The value at `place` (`copy_value`): up to a NUL, a carriage return, a line feed or a
/// tab; one that starts with `"` without it and up to the next `"` in place of a tab.
pub fn value(text: &(impl Bytes + ?Sized), place: usize) -> Result<Vec<u8>, Invalid> {
    let mut place = place;
    let mut end = b'\t';
    if at(text, place)? == b'"' {
        end = b'"';
        place += 1;
    }
    let mut out = Vec::new();
    loop {
        let c = at(text, place)?;
        place += 1;
        if c == 0 || c == b'\r' || c == b'\n' || c == end {
            return Ok(out);
        }
        out.push(c);
    }
}

/// The number at `place` (`parse_number`): an optional `+` or `-`, then hexadecimal after
/// `$` or `0x` (upper or lower case), decimal otherwise, up to the first character that is
/// no digit. It wraps.
pub fn number(text: &(impl Bytes + ?Sized), place: usize) -> Result<u32, Invalid> {
    let mut place = place;
    let negative = at(text, place)? == b'-';
    if negative || at(text, place)? == b'+' {
        place += 1;
    }
    let mut n = 0u32;
    let prefix = if at(text, place)? == b'$' {
        Some(1)
    } else if at(text, place)? == b'0' && at(text, place + 1)? == b'x' {
        Some(2)
    } else {
        None
    };
    match prefix {
        None => loop {
            let digit = at(text, place)?.wrapping_sub(b'0');
            place += 1;
            if digit >= 10 {
                break;
            }
            n = n.wrapping_mul(10).wrapping_add(digit as u32);
        },
        Some(prefix) => {
            place += prefix;
            loop {
                let digit = match at(text, place)? {
                    c @ b'0'..=b'9' => c - b'0',
                    c @ b'A'..=b'F' => c - b'A' + 10,
                    c @ b'a'..=b'f' => c - b'a' + 10,
                    _ => break,
                };
                place += 1;
                n = n << 4 | digit as u32;
            }
        }
    }
    Ok(if negative { n.wrapping_neg() } else { n })
}

/// How a setting's variable sits in the game's memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A whole aligned dword.
    Dword,
    /// A dword that straddles two cells, so its owner stores the four bytes itself.
    Unaligned,
}

/// A setting of `var\ral.cfg`: its place in [`SETTINGS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setting(pub usize);

impl Setting {
    /// The key the setting is written under.
    pub fn name(self) -> &'static [u8] {
        SETTINGS[self.0].0
    }

    /// How its variable is stored.
    pub fn kind(self) -> Kind {
        SETTINGS[self.0].1
    }
}

/// The 77 settings in the order they are applied: key (lower case) and kind. Two keys may name
/// one variable (`noaudio` / `nosound`, `allsecretcars` / `secretcar`); the later one wins.
pub const SETTINGS: [(&[u8], Kind); 77] = [
    (b"notaskswitch", Kind::Dword),
    (b"noprichange", Kind::Dword),
    (b"ignoremmx", Kind::Dword),
    (b"ignorekatmai", Kind::Dword),
    (b"ignoreamdk3d", Kind::Dword),
    (b"altmemenable", Kind::Dword),
    (b"altmemheapsize", Kind::Dword),
    (b"noaudio", Kind::Dword),
    (b"nosound", Kind::Dword),
    (b"nocdaudio", Kind::Dword),
    (b"nomusic", Kind::Dword),
    (b"audiobasefreq", Kind::Dword),
    (b"ignoreds3d", Kind::Dword),
    (b"ignoredsv", Kind::Dword),
    (b"ignoreeax", Kind::Dword),
    (b"ignorea3d", Kind::Dword),
    (b"wavheapsize", Kind::Dword),
    (b"bundleheapsize", Kind::Dword),
    (b"imageheapsize", Kind::Dword),
    (b"gamegfxheapsize", Kind::Dword),
    (b"forcedx5", Kind::Dword),
    (b"forceno3dcard", Kind::Dword),
    (b"ignoreprimary", Kind::Dword),
    (b"ignoresecondary", Kind::Dword),
    (b"agpmode", Kind::Dword),
    (b"force3dtrans", Kind::Dword),
    (b"force3dalpha", Kind::Dword),
    (b"force3dshadow", Kind::Dword),
    (b"force3dfogging", Kind::Dword),
    (b"textflashmode", Kind::Dword),
    (b"videomodemain", Kind::Dword),
    (b"videomodegame", Kind::Dword),
    (b"v3driverfix", Kind::Dword),
    (b"nopngdecode", Kind::Dword),
    (b"nojoy", Kind::Dword),
    (b"joymode", Kind::Dword),
    (b"joycode", Kind::Dword),
    (b"joylowerdz", Kind::Dword),
    (b"joyupperdz", Kind::Dword),
    (b"noforcefeedback", Kind::Dword),
    (b"joyffgain", Kind::Dword),
    (b"mergejoy", Kind::Dword),
    (b"nonetwork", Kind::Dword),
    (b"inetport", Kind::Dword),
    (b"screensnapshot", Kind::Dword),
    (b"framecounter", Kind::Dword),
    (b"noreplay", Kind::Dword),
    (b"replaykeys", Kind::Dword),
    (b"logfile", Kind::Dword),
    (b"nofmv", Kind::Dword),
    (b"key", Kind::Dword),
    (b"allsecretcars", Kind::Dword),
    (b"secretcar", Kind::Dword),
    (b"usesavegamekeys", Kind::Dword),
    (b"sensiblesaving", Kind::Dword),
    (b"noprecipitation", Kind::Dword),
    (b"seasoncode", Kind::Dword),
    (b"noautoneutral", Kind::Dword),
    (b"nonlinearsteering", Kind::Dword),
    (b"wrreplayenable", Kind::Unaligned),
    (b"wrreplayrate", Kind::Unaligned),
    (b"alt2axcode", Kind::Dword),
    (b"fizpatch", Kind::Dword),
    (b"patchmaxdrawdistance", Kind::Dword),
    (b"patchstickygrass", Kind::Dword),
    (b"patchgravity", Kind::Dword),
    (b"patchdamages", Kind::Dword),
    (b"patchbrakegrip", Kind::Dword),
    (b"patchaccelgrip", Kind::Dword),
    (b"patchslidegrip", Kind::Dword),
    (b"patchgripgrass", Kind::Dword),
    (b"patchgriptarmac", Kind::Dword),
    (b"patchgripsnow", Kind::Dword),
    (b"patchgripmud", Kind::Dword),
    (b"patchgripgravel", Kind::Dword),
    (b"lexcontrol", Kind::Dword),
    (b"sparksmode", Kind::Dword),
];

/// What a settings text sets: each setting it has, in [`SETTINGS`] order, with the number of its
/// value (0 for a value that is no number). A key the text lacks sets nothing.
pub fn parse_settings(
    text: &(impl Bytes + ?Sized),
    case: &CaseTable,
) -> Result<Vec<(Setting, u32)>, Invalid> {
    let mut found = Vec::new();
    for (n, (key, _)) in SETTINGS.iter().enumerate() {
        if let Some(place) = find_key(text, key, case)? {
            found.push((Setting(n), number(text, place)?));
        }
    }
    Ok(found)
}

/// The room a settings text is read into, its NUL after it.
pub const TEXT_ROOM: u32 = 0xFFFF;

/// `settings::read_text`: a read (whether it worked, the bytes it read) -> where the NUL
/// goes (the text's length), or None when there is no text.
pub fn text_read(ok: bool, bytes: Option<u32>) -> Option<u32> {
    match (ok, bytes) {
        (true, Some(bytes)) => Some(bytes),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_read() {
        assert_eq!(text_read(true, Some(5)), Some(5));
        assert_eq!(
            (text_read(false, Some(5)), text_read(true, None)),
            (None, None)
        );
    }

    #[test]
    fn settings_text_is_parsed_in_table_order() {
        let case = CaseTable::ascii();
        let name = |s: Setting| String::from_utf8_lossy(s.name()).into_owned();
        let parsed = |text: &[u8]| {
            parse_settings(text, &case)
                .unwrap()
                .into_iter()
                .map(|(s, n)| (name(s), n))
                .collect::<Vec<_>>()
        };
        assert_eq!(SETTINGS.len(), 77);
        // Both forms, a key without a number, an unknown key, lines in any order.
        assert_eq!(
            parsed(b"sparksmode 3\r\nbogus=9\r\nIgnoreMMX=7\r\nignorekatmai\r\n\0"),
            [
                ("ignoremmx".to_string(), 7),
                ("ignorekatmai".to_string(), 0),
                ("sparksmode".to_string(), 3)
            ]
        );
        // Keys sharing a variable are both reported, the later in the table last.
        assert_eq!(
            parsed(b"nosound=1\nnoaudio=2\n\0"),
            [("noaudio".to_string(), 2), ("nosound".to_string(), 1)]
        );
        assert_eq!(
            parsed(b"hex=$1F\nkey = -0x10\n\0")[0],
            ("key".to_string(), 0x10u32.wrapping_neg())
        );
        assert_eq!(parsed(b"\0"), []);
        assert!(parse_settings(&b"no nul"[..], &case).is_err());
        assert_eq!(Setting(0).kind(), Kind::Dword);
        let unaligned: Vec<_> = (0..77)
            .filter(|&n| Setting(n).kind() == Kind::Unaligned)
            .map(|n| name(Setting(n)))
            .collect();
        assert_eq!(unaligned, ["wrreplayenable", "wrreplayrate"]);
    }

    const TEXT: &[u8] = b"; ral.cfg\r\n  LogFile=10\r\nscreen 800\r\nempty\r\nname000 \"A B\"\tx\r\nhex=$1F\r\nneg = -0x10\r\nlast\0";

    fn find(key: &[u8]) -> Option<usize> {
        find_key(TEXT, key, &CaseTable::ascii()).unwrap()
    }

    #[test]
    fn keys_values_numbers() {
        let at = find(b"logfile").unwrap();
        assert_eq!(number(TEXT, at).unwrap(), 10);
        assert_eq!(number(TEXT, find(b"screen").unwrap()).unwrap(), 800);
        assert_eq!(TEXT[find(b"empty").unwrap()], b'\n');
        assert_eq!(value(TEXT, find(b"name000").unwrap()).unwrap(), b"A B");
        assert_eq!(number(TEXT, find(b"hex").unwrap()).unwrap(), 0x1F);
        assert_eq!(
            number(TEXT, find(b"neg").unwrap()).unwrap(),
            0x10u32.wrapping_neg()
        );
        assert_eq!(find(b"log"), None);
        assert_eq!(find(b"missing"), None);
        assert!(find_key(TEXT, b"last", &CaseTable::ascii()).is_err());
        assert!(find_key(&b"no nul"[..], b"x", &CaseTable::ascii()).is_err());
        assert_eq!(value(b"plain\tnext\0", 0).unwrap(), b"plain");
        assert_eq!(number(b"12x\0", 0).unwrap(), 12);
    }
}

/// What `var\name.cfg` gives a driver: key `name000` .. `name179`, then up to three words: a name
/// (at most 30 characters kept), a number under 0x28 for the driver record's +0, one under 0x22 for
/// its +4; a number too large is left out, a missing word ends the entry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriverName {
    pub name: Option<Vec<u8>>,
    pub first: Option<u32>,
    pub second: Option<u32>,
}

/// The names file's entries, one per driver (`None` for a driver without its key).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriverNames {
    pub entries: Vec<Option<DriverName>>,
}

/// The names file read from its text.
pub fn read_driver_names(
    text: &(impl Bytes + ?Sized),
    case: &CaseTable,
) -> Result<DriverNames, Invalid> {
    Ok(DriverNames {
        entries: driver_names(text, case)?,
    })
}

/// The drivers' entries.
pub const DRIVERS: usize = 180;
/// The most of a name kept.
pub const NAME_MAX: usize = 30;

/// Each driver's entry (`None` for a driver without its key).
pub fn driver_names(
    text: &(impl Bytes + ?Sized),
    case: &CaseTable,
) -> Result<Vec<Option<DriverName>>, Invalid> {
    use crate::strings::format::{Pad, decimal};
    use crate::strings::words::next_argument;
    let mut out = Vec::with_capacity(DRIVERS);
    for driver in 0..DRIVERS as u32 {
        let mut key = b"name".to_vec();
        key.extend(decimal(driver, Pad::Zeros(3)));
        let Some(value) = find_key(text, &key, case)? else {
            out.push(None);
            continue;
        };
        let mut entry = DriverName::default();
        let w = next_argument(text, value)?;
        if w.len != 0 {
            let len = w.len.min(NAME_MAX);
            entry.name = Some(
                (0..len)
                    .map(|n| at(text, w.start + n))
                    .collect::<Result<_, _>>()?,
            );
        }
        let w = next_argument(text, w.next)?;
        if w.len != 0 {
            let n = number(text, w.start)?;
            entry.first = (n < 0x28).then_some(n);
        }
        let w = next_argument(text, w.next)?;
        if w.len != 0 {
            let n = number(text, w.start)?;
            entry.second = (n < 0x22).then_some(n);
        }
        out.push(Some(entry));
    }
    Ok(out)
}

/// Which of the processor's extensions the game uses: MMX, SSE, 3DNow each when the processor has
/// it (bit 0) and its ignore setting is 0.
pub fn feature_switches(has: [u32; 3], ignore: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|n| ((ignore[n] == 0) as u32) & has[n])
}
