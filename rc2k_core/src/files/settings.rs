//! The game's settings texts (`var\ral.cfg`, `var\name.cfg`): lines of `key=value` or
//! `key value` (converted 2026-10-20 from `settings::find_key` 0x41157D,
//! `settings::copy_value` 0x4115F5, `settings::parse_number` 0x411623). Places are offsets
//! into the text; a text ends at its NUL.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::case::CaseTable;

fn at(text: &(impl Bytes + ?Sized), place: usize) -> Result<u8, Invalid> {
    text.byte(place)
        .ok_or(Invalid("a settings text without its NUL"))
}

/// Where key `key`'s value starts, or `None` (`find_key`). A line is looked at from its
/// first character above a space; the key must be there in any case (the text is lowered
/// through `case`; the key is given in lower case) and be followed by `=` or a character
/// of a space or less. The value starts at the first character above a space after that,
/// an `=` between them skipped once; a line that ends first has its value at its line feed
/// (empty). A text that ends with the key itself is refused (the original compares on
/// past both ends).
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
/// no digit. It wraps, as the original's.
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

/// The game's settings table (0x518C09: for each a key, its NUL, then a dword naming its
/// variable), read up to the empty key; `limit` the bytes it may take (refused past it).
/// The dword is the caller's to interpret.
pub fn table(bytes: &(impl Bytes + ?Sized), limit: usize) -> Result<Vec<(Vec<u8>, u32)>, Invalid> {
    const END: Invalid = Invalid("the settings table has no end");
    let mut entries = Vec::new();
    let mut at = 0usize;
    while bytes.byte(at).ok_or(END)? != 0 {
        let mut key = Vec::new();
        while let byte @ 1.. = bytes.byte(at).ok_or(END)? {
            key.push(byte);
            at += 1;
        }
        entries.push((key, bytes.u32_le(at + 1).ok_or(END)?));
        at += 5;
        if at > limit {
            return Err(END);
        }
    }
    Ok(entries)
}

/// What a settings text sets (`settings::apply_settings`, 0x4114FF): for each key it
/// has, in the table's order, the key's index and the number of its value (0 for a value
/// that is no number). A key the text lacks sets nothing; of two keys with one variable
/// the later one wins, as the caller stores in order.
pub fn apply(
    text: &(impl Bytes + ?Sized),
    keys: &[&[u8]],
    case: &CaseTable,
) -> Result<Vec<(usize, u32)>, Invalid> {
    let mut found = Vec::new();
    for (n, key) in keys.iter().enumerate() {
        if let Some(place) = find_key(text, key, case)? {
            found.push((n, number(text, place)?));
        }
    }
    Ok(found)
}

/// The room a settings text is read into (`settings::read_text` 0x41155A, converted
/// 2026-10-21), its NUL after it.
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
    fn tables_and_applying() {
        let raw = b"one\0\x01\0\0\0two\0\x02\0\0\0\0";
        let t = table(&raw[..], raw.len()).unwrap();
        assert_eq!(t, [(b"one".to_vec(), 1), (b"two".to_vec(), 2)]);
        assert!(table(&raw[..], 4).is_err());
        let keys: Vec<&[u8]> = t.iter().map(|(k, _)| k.as_slice()).collect();
        let found = apply(b"two\t5\r\n\0", &keys, &CaseTable::ascii()).unwrap();
        assert_eq!(found, [(1, 5)]);
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

/// What `var\name.cfg` gives a driver (`settings::read_driver_names` 0x4162DB): key
/// `name000` .. `name179`, then up to three words: a name (at most 30 characters kept), a
/// number under 0x28 for the driver record's +0 (`dword_73B688`), one under 0x22 for its +4
/// (`dword_73B68C`); a number too large is left out, a missing word ends the entry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DriverName {
    pub name: Option<Vec<u8>>,
    pub first: Option<u32>,
    pub second: Option<u32>,
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

/// Which of the processor's extensions the game uses (`settings::apply_feature_switches`
/// 0x4114BB): MMX, SSE, 3DNow each when the processor has it (bit 0) and its ignore
/// setting is 0.
pub fn feature_switches(has: [u32; 3], ignore: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|n| ((ignore[n] == 0) as u32) & has[n])
}
