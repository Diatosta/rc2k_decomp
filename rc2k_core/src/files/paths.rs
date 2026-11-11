//! The game's paths as text (from `files::append_directory` 0x403070,
//! `files::append_file_name` 0x403040, `files::numbered_path` 0x403292,
//! `files::resolve_path` 0x403206, `files::swap_missing_extension` 0x4032DB). Paths are
//! given without their NUL.

use crate::Invalid;
use crate::strings::format::{Pad, decimal};

/// Whether a byte ends a directory.
pub fn is_separator(b: u8) -> bool {
    matches!(b, b'/' | b'\\' | b':')
}

/// The last separator's place, searched from the end down to the second byte (the first
/// is never looked at, as in the original).
pub fn last_separator(path: &[u8]) -> Option<usize> {
    (1..path.len()).rev().find(|&k| is_separator(path[k]))
}

/// The directory, up to and with its last separator; `None` without one.
pub fn directory(path: &[u8]) -> Option<&[u8]> {
    last_separator(path).map(|end| &path[..=end])
}

/// The file name, after the last separator (the whole path without one).
pub fn file_name(path: &[u8]) -> &[u8] {
    &path[last_separator(path).map_or(0, |k| k + 1)..]
}

/// The template with `number` written over its last `!!`, or if there is none its last
/// `00`: two digits, zero-padded (a larger number's last two). `Ok(None)` without
/// either marker (a fatal error in the game); an empty template is refused (the original
/// reads before it).
pub fn numbered(template: &[u8], number: u32) -> Result<Option<Vec<u8>>, Invalid> {
    if template.is_empty() {
        return Err(Invalid(
            "empty template: the original reads before its buffer",
        ));
    }
    let last = |mark: u8| {
        (1..template.len())
            .rev()
            .find(|&at| template[at] == mark && template.get(at + 1) == Some(&mark))
    };
    let Some(at) = last(b'!').or_else(|| last(b'0')) else {
        return Ok(None);
    };
    let mut out = template.to_vec();
    out.push(0);
    out[at..at + 2].copy_from_slice(&decimal(number, Pad::Zeros(2)));
    Ok(Some(out))
}

/// Where the game keeps its files, for `resolve`.
#[derive(Clone, Copy, Debug)]
pub struct Bases<'a> {
    /// How the game was installed (2: the large install, 3 and up: everything on disk).
    pub media_mode: u32,
    /// The CD's folder and the installed one.
    pub a: &'a [u8],
    pub b: &'a [u8],
    /// What `%` stands for (the locale).
    pub percent: &'a [u8],
}

/// A path as the game opens it, with its NUL; `None` for one used as it is (`var\`,
/// `save...` or with a drive). `;3` and `;4` before a path choose the first base and the
/// installed one; otherwise the media mode chooses. Each `%` becomes the locale.
pub fn resolve(path: &[u8], bases: &Bases) -> Option<Vec<u8>> {
    if path.starts_with(b"var\\") || path.starts_with(b"save") || path.get(1) == Some(&b':') {
        return None;
    }
    let (semi3, semi4) = (path.starts_with(b";3"), path.starts_with(b";4"));
    let use_b = !semi4 && (bases.media_mode >= 3 || (!semi3 && bases.media_mode == 2));
    let base = if use_b { bases.b } else { bases.a };
    let rel = if path.first() == Some(&b';') {
        &path[2.min(path.len())..]
    } else {
        path
    };
    let mut out = base.to_vec();
    for &c in rel {
        if c == b'%' {
            out.extend_from_slice(bases.percent);
        } else {
            out.push(c);
        }
    }
    out.push(0);
    Some(out)
}

/// The ending a path of at least five characters takes when its file is not there: `.pcx`
/// for `.png`, `.mpg` for `.avi` (lower case only).
pub fn missing_partner(path: &[u8]) -> Option<&'static [u8; 4]> {
    const PARTNERS: [(&[u8; 4], &[u8; 4]); 2] = [(b".png", b".pcx"), (b".avi", b".mpg")];
    if path.len() < 5 {
        return None;
    }
    let ending = &path[path.len() - 4..];
    PARTNERS
        .iter()
        .find(|(from, _)| ending == &from[..])
        .map(|(_, to)| *to)
}

/// The root `files::drive_type` (0x4031D8) asks about: `x:\` with
/// the letter lowered (`lower`, the game's case table), with its NUL; None (the current
/// drive) for a letter of 0.
pub fn drive_root(letter: u8, lower: impl FnOnce(u8) -> u8) -> Option<[u8; 4]> {
    (letter != 0).then(|| [lower(letter), b':', b'\\', 0])
}

/// `GetDriveTypeA`'s answer for a CD-ROM drive.
pub const CD_ROM: u32 = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves() {
        assert_eq!(directory(b"files\\gfx\\a.png"), Some(&b"files\\gfx\\"[..]));
        assert_eq!(file_name(b"files\\gfx\\a.png"), b"a.png");
        assert_eq!(directory(b"\\a"), None); // the first byte is not looked at
        assert_eq!(file_name(b"c:x"), b"x");
        assert_eq!(file_name(b"plain"), b"plain");
    }

    #[test]
    fn numbers() {
        assert_eq!(
            numbered(b"save!!.dat", 7).unwrap().unwrap(),
            b"save07.dat\0"
        );
        assert_eq!(numbered(b"car00x00", 5).unwrap().unwrap(), b"car00x05\0");
        assert_eq!(numbered(b"a!!", 123).unwrap().unwrap(), b"a23\0");
        assert_eq!(numbered(b"plain", 1).unwrap(), None);
        assert!(numbered(b"", 1).is_err());
    }

    #[test]
    fn resolves() {
        let bases = Bases {
            media_mode: 1,
            a: b"d:\\",
            b: b"c:\\rc\\",
            percent: b"uk",
        };
        assert_eq!(resolve(b"var\\ral.cfg", &bases), None);
        assert_eq!(resolve(b"c:\\x", &bases), None);
        assert_eq!(
            resolve(b"files\\%\\a.txt", &bases).unwrap(),
            b"d:\\files\\uk\\a.txt\0"
        );
        assert_eq!(resolve(b";4x", &bases).unwrap(), b"d:\\x\0");
        let installed = Bases {
            media_mode: 2,
            ..bases
        };
        assert_eq!(resolve(b"x", &installed).unwrap(), b"c:\\rc\\x\0");
        assert_eq!(resolve(b";3x", &installed).unwrap(), b"d:\\x\0");
        assert_eq!(missing_partner(b"a.png"), Some(b".pcx"));
        assert_eq!(missing_partner(b".png"), None);
        assert_eq!(missing_partner(b"a.PNG"), None);
    }

    #[test]
    fn drive_roots() {
        assert_eq!(
            drive_root(b'D', |b| b.to_ascii_lowercase()),
            Some(*b"d:\\\0")
        );
        assert_eq!(drive_root(0, |_| unreachable!()), None);
    }
}
