//! The files about the installation (converted 2026-10-21 from `install::read_record`
//! 0x4110C9, `install::write_record` 0x411186, `install::read_sku` 0x411696,
//! `install::read_languages` 0x413908 and `install::choose_language` 0x41397A): the record
//! `var\ral.zog`, the edition file `files\d\gfx\sku` and the languages file `lang.cfg`, all
//! settings texts (`files::settings`); the keys and the record's header are RAL.EXE's.

use crate::Invalid;
use crate::files::settings::{find_key, number, value};
use crate::strings::bytes::Bytes;
use crate::strings::case::CaseTable;
use crate::strings::format::{Pad, decimal};

/// The record's keys.
pub const CDROM: &[u8] = b"cdrom";
pub const LANGUAGE: &[u8] = b"language";
pub const INSTALLATION: &[u8] = b"installation";
/// The record's first lines.
pub const HEADER: &[u8] = b"; Rally Championship\r\n; INSTALLATION CONFIGURATION\r\n\r\n";

/// A directory's place: 0x100 bytes.
pub const PATH_ROOM: usize = 0x100;

/// What the record holds, each `None` when its key is not there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    /// `cdrom`: base directory A, with its NUL.
    pub base: Option<Vec<u8>>,
    /// `installation`: the installation's kind, held to 0 .. 3 (signed).
    pub mode: Option<u32>,
    /// `language`: what `%` stands for in a path, with its NUL.
    pub locale: Option<Vec<u8>>,
}

/// A directory as the record names it: the value, a `\` unless it is empty or ends in `\`
/// or `/`, a NUL. Refused: one past its place (the original writes on).
pub fn directory(mut text: Vec<u8>) -> Result<Vec<u8>, Invalid> {
    if text.len() + 2 > PATH_ROOM {
        return Err(Invalid("a directory in the record longer than its place"));
    }
    if !matches!(text.last(), None | Some(b'/' | b'\\')) {
        text.push(b'\\');
    }
    text.push(0);
    Ok(text)
}

/// The record read from its text (to its NUL): the keys looked for in the original's order,
/// `cdrom`, `installation`, `language`.
pub fn read_record(text: &(impl Bytes + ?Sized), case: &CaseTable) -> Result<Record, Invalid> {
    let base = find_key(text, CDROM, case)?
        .map(|at| directory(value(text, at)?))
        .transpose()?;
    let mode = find_key(text, INSTALLATION, case)?
        .map(|at| number(text, at).map(|n| (n as i32).clamp(0, 3) as u32))
        .transpose()?;
    let locale = find_key(text, LANGUAGE, case)?
        .map(|at| directory(value(text, at)?))
        .transpose()?;
    Ok(Record { base, mode, locale })
}

/// The record's text: the header, `cdrom=` the folder installed into, `language=` the
/// language's directory when there is one, `installation=` the kind, each line ended CR LF.
pub fn record_text(installed_into: &[u8], locale: Option<&[u8]>, mode: u32) -> Vec<u8> {
    let mut out = HEADER.to_vec();
    let mut line = |key: &[u8], value: &[u8]| {
        out.extend(key);
        out.push(b'=');
        out.extend(value);
        out.extend(b"\r\n");
    };
    line(CDROM, installed_into);
    if let Some(locale) = locale {
        line(LANGUAGE, locale);
    }
    line(INSTALLATION, &decimal(mode, Pad::None));
    out
}

/// The edition file's key.
pub const SKU: &[u8] = b"sku";

/// What the edition file's `sku` sets: the marks `oem:` and `dvd:` (each one there sets its
/// switch to 1; what is not there keeps its value), a number, after the next `:` a second
/// and after the next `.` (within 254 characters) a third.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sku {
    pub oem: bool,
    pub dvd: bool,
    pub number: u32,
    pub second: Option<u8>,
    pub third: Option<u8>,
}

/// The edition file read from its text; `None` without the key.
pub fn read_sku(text: &(impl Bytes + ?Sized), case: &CaseTable) -> Result<Option<Sku>, Invalid> {
    let Some(mut at) = find_key(text, SKU, case)? else {
        return Ok(None);
    };
    let byte = |n: usize| {
        text.byte(n)
            .ok_or(Invalid("the edition file without its NUL"))
    };
    let mut sku = Sku::default();
    loop {
        let mark = [byte(at)?, byte(at + 1)?, byte(at + 2)?, byte(at + 3)?];
        match &mark {
            b"oem:" => sku.oem = true,
            b"dvd:" => sku.dvd = true,
            _ => break,
        }
        at += 4;
    }
    sku.number = number(text, at)?;
    // On to after the next `:`, however far.
    loop {
        let c = byte(at)?;
        at += 1;
        match c {
            0 => return Ok(Some(sku)),
            b':' => break,
            _ => {}
        }
    }
    sku.second = Some(number(text, at)? as u8);
    // On to after the next `.`: the original's count gives up at the 255th character.
    for seen in 1.. {
        let c = byte(at)?;
        at += 1;
        if c == 0 || seen == 255 {
            return Ok(Some(sku));
        }
        if c == b'.' {
            break;
        }
    }
    sku.third = Some(number(text, at)? as u8);
    Ok(Some(sku))
}

/// A language's slot: 0x80 bytes, two texts each ended by a NUL (`name:directory` in the
/// file); nine of them.
pub const SLOT: usize = 0x80;
pub const SLOTS: usize = 9;

/// The languages file's keys, `lang1` .. `lang9`, for slots 0 .. 8 (RAL.EXE's table at
/// 0x51C378).
pub const LANGUAGE_KEYS: [&[u8]; SLOTS] = [
    b"lang1", b"lang2", b"lang3", b"lang4", b"lang5", b"lang6", b"lang7", b"lang8", b"lang9",
];

/// The languages found in the file's text, in the keys' order: each slot and its value.
/// Refused: a value of `SLOT` bytes or more (the original writes on into the next slot).
pub fn find_languages(
    text: &(impl Bytes + ?Sized),
    case: &CaseTable,
) -> Result<Vec<(usize, Vec<u8>)>, Invalid> {
    let mut found = Vec::new();
    for (slot, key) in LANGUAGE_KEYS.iter().enumerate() {
        if let Some(at) = find_key(text, key, case)? {
            let v = value(text, at)?;
            if v.len() >= SLOT {
                return Err(Invalid("a language longer than its slot"));
            }
            found.push((slot, v));
        }
    }
    Ok(found)
}

/// The slots kept (`SLOTS` x `SLOT` bytes): those that start with a character above a
/// space other than `:` and have a `:` (anywhere in the slot), the `:` made a NUL, moved
/// together to the front in order. Their count.
pub fn keep_languages(slots: &mut [u8]) -> u32 {
    let mut kept = 0usize;
    for slot in 0..SLOTS {
        let from = slot * SLOT;
        let text = &mut slots[from..from + SLOT];
        if text[0] <= b' ' || text[0] == b':' {
            continue;
        }
        let Some(colon) = text.iter().position(|&c| c == b':') else {
            continue;
        };
        text[colon] = 0;
        slots.copy_within(from..from + SLOT, kept * SLOT);
        kept += 1;
    }
    kept as u32
}

/// The slot of language `number` (from 1) when there is one to choose: `None` for a number
/// above the `count` (nothing done). Refused: 0 (the original takes the slot before the
/// first) and one past the slots.
pub fn language_slot(number: u32, count: u32) -> Result<Option<usize>, Invalid> {
    if number > count {
        return Ok(None);
    }
    if number == 0 || number as usize > SLOTS {
        return Err(Invalid("a language number without a slot"));
    }
    Ok(Some(number as usize - 1))
}

/// A language's directory: its slot's second text. Refused: a slot whose two texts do not
/// end inside it.
pub fn language_directory(slot: &[u8]) -> Result<&[u8], Invalid> {
    let mut texts = slot.split(|&c| c == 0);
    let (Some(_), Some(directory), Some(_)) = (texts.next(), texts.next(), texts.next()) else {
        return Err(Invalid("a language slot whose texts do not end in it"));
    };
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &[u8]) -> Vec<u8> {
        let mut t = s.to_vec();
        t.push(0);
        t
    }

    #[test]
    fn record() {
        let case = CaseTable::ascii();
        let r = read_record(
            &text(b"cdrom=D:\r\nlanguage=english\\\r\ninstallation=7\r\n"),
            &case,
        )
        .unwrap();
        assert_eq!(r.base, Some(b"D:\\\0".to_vec()));
        assert_eq!(r.locale, Some(b"english\\\0".to_vec()));
        assert_eq!(r.mode, Some(3));
        let r = read_record(&text(b"installation=-4\r\ncdrom=\r\n"), &case).unwrap();
        assert_eq!((r.base, r.mode, r.locale), (Some(vec![0]), Some(0), None));
        assert!(
            read_record(
                &text(&[b"cdrom=".as_slice(), &[b'x'; 0xFF]].concat()),
                &case
            )
            .is_err()
        );
        assert_eq!(
            record_text(b"c:\\rally\\", Some(b"english\\"), 2),
            [
                HEADER,
                b"cdrom=c:\\rally\\\r\nlanguage=english\\\r\ninstallation=2\r\n".as_slice()
            ]
            .concat()
        );
        assert_eq!(
            record_text(b"x", None, 10),
            [HEADER, b"cdrom=x\r\ninstallation=10\r\n".as_slice()].concat()
        );
    }

    #[test]
    fn sku() {
        let case = CaseTable::ascii();
        let s = read_sku(&text(b"sku=dvd:oem:3:$1f.0x20\r\n"), &case)
            .unwrap()
            .unwrap();
        assert_eq!(
            s,
            Sku {
                oem: true,
                dvd: true,
                number: 3,
                second: Some(0x1F),
                third: Some(0x20)
            }
        );
        let s = read_sku(&text(b"sku=42\r\n"), &case).unwrap().unwrap();
        assert_eq!((s.number, s.second, s.third), (42, None, None));
        assert_eq!(read_sku(&text(b"other=1\r\n"), &case), Ok(None));
        let mut far = b"sku=1:2".to_vec();
        far.extend([b'x'; 254]);
        far.extend(b".9\r\n");
        assert_eq!(read_sku(&text(&far), &case).unwrap().unwrap().third, None);
    }

    #[test]
    fn languages() {
        let case = CaseTable::ascii();
        let found = find_languages(
            &text(b"lang3=Third:third\r\nlang1=nocolon\r\nlang2=:x\r\n"),
            &case,
        )
        .unwrap();
        assert_eq!(found.len(), 3);
        let mut slots = vec![0u8; SLOT * SLOTS];
        for (slot, v) in &found {
            slots[slot * SLOT..slot * SLOT + v.len()].copy_from_slice(v);
        }
        assert_eq!(keep_languages(&mut slots), 1);
        assert_eq!(&slots[..12], b"Third\0third\0");
        assert_eq!(language_directory(&slots[..SLOT]), Ok(b"third".as_slice()));
        assert_eq!(language_slot(1, 1), Ok(Some(0)));
        assert_eq!(language_slot(2, 1), Ok(None));
        assert!(language_slot(0, 1).is_err());
        assert!(language_directory(&[b'a'; SLOT]).is_err());
    }
}
