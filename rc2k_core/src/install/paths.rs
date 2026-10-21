//! The installer's folder paths (converted 2026-10-21 from `install::normalise_path`
//! 0x4139EC and `install::check_folder_name` 0x413A6C): the one form it keeps them in, and
//! the characters a folder's name may not hold.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::case::CaseTable;

/// The characters no folder name may hold (RAL.EXE's `unk_51DE18`, NUL-ended there). The
/// `\` in it is never compared: a `\` is looked at as a separator first.
pub const NOT_IN_NAMES: &[u8] = b"\"+*,;=/:<>?\\{}[]|\x7f";

/// The longest path taken, less one (the installer's places are 0x100 bytes).
pub const LONGEST: usize = 0xF8;

/// Whether a folder path holds a character of `NOT_IN_NAMES` or a double backslash, from
/// its third byte on (past a drive letter and colon, not looked at, nor whether the path is
/// that long) to its NUL.
pub fn bad_folder_name(path: &(impl Bytes + ?Sized)) -> Result<bool, Invalid> {
    let at = |n: usize| path.byte(n).ok_or(Invalid("a folder path without its NUL"));
    let mut n = 2;
    loop {
        let c = at(n)?;
        n += 1;
        match c {
            0 => return Ok(false),
            b'\\' => {
                if at(n)? == b'\\' {
                    return Ok(true);
                }
            }
            c if NOT_IN_NAMES.contains(&c) => return Ok(true),
            _ => {}
        }
    }
}

fn is_slash(c: u8) -> bool {
    c == b'\\' || c == b'/'
}

/// A directory's path in the installer's form, with its NUL; `None` if it is not one that
/// is taken. The form: lower case (`case`), `\` for every `/`, and a `\` at the end. An
/// empty path stands for the current directory (`current`, asked only then; `None`: not
/// written), which gets a `\` after it unless it ends in one. Otherwise it must start with
/// `\\` or `//`, or with a drive letter and `:`, after which a `\` is put if there is none.
/// Refused: a path of `LONGEST` bytes or more.
pub fn normalise<E: From<Invalid>>(
    path: &[u8],
    case: &CaseTable,
    current: impl FnOnce() -> Result<Option<Vec<u8>>, E>,
) -> Result<Option<Vec<u8>>, E> {
    if path.len() >= LONGEST {
        return Err(Invalid("a path longer than the installer's places").into());
    }
    let plain = |c: u8| case.lower(if c == b'/' { b'\\' } else { c });
    let mut out = Vec::new();
    let rest: Vec<u8> = if path.is_empty() {
        let Some(mut directory) = current()? else {
            return Ok(None);
        };
        if !directory.last().is_some_and(|&c| is_slash(c)) {
            directory.push(b'\\');
        }
        directory
    } else if path.starts_with(b"\\\\") || path.starts_with(b"//") {
        path.to_vec()
    } else if path.get(1) == Some(&b':') {
        out.extend([case.lower(path[0]), b':']);
        if !path.get(2).is_some_and(|&c| is_slash(c)) {
            out.push(b'\\');
        }
        path[2..].to_vec()
    } else {
        return Ok(None);
    };
    out.extend(rest.into_iter().map(plain));
    if !out.last().is_some_and(|&c| is_slash(c)) {
        out.push(b'\\');
    }
    out.push(0);
    Ok(Some(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_names() {
        assert_eq!(bad_folder_name(b"c:\\games\\rally\0"), Ok(false));
        assert_eq!(bad_folder_name(b"c:\\games\\\\rally\0"), Ok(true));
        assert_eq!(bad_folder_name(b"c:\\ga?mes\0"), Ok(true));
        assert_eq!(bad_folder_name(b"c:\\a:b\0"), Ok(true));
        assert_eq!(bad_folder_name(b"?:\\a\0"), Ok(false));
        assert!(bad_folder_name(b"c:\\abc").is_err());
    }

    #[test]
    fn normalised() {
        let case = CaseTable::ascii();
        let none = || -> Result<Option<Vec<u8>>, Invalid> { Ok(None) };
        let n = |p: &[u8]| normalise(p, &case, none).unwrap();
        assert_eq!(n(b"C:\\Games\\RC2k"), Some(b"c:\\games\\rc2k\\\0".to_vec()));
        assert_eq!(n(b"c:"), Some(b"c:\\\0".to_vec()));
        assert_eq!(n(b"D:/Mixed/Case/"), Some(b"d:\\mixed\\case\\\0".to_vec()));
        assert_eq!(n(b"E:File"), Some(b"e:\\file\\\0".to_vec()));
        assert_eq!(
            n(b"//Server/Share"),
            Some(b"\\\\server\\share\\\0".to_vec())
        );
        assert_eq!(n(b"relative"), None);
        assert_eq!(n(b""), None);
        let here = || -> Result<Option<Vec<u8>>, Invalid> { Ok(Some(b"Z:\\Here".to_vec())) };
        assert_eq!(
            normalise(b"", &case, here),
            Ok(Some(b"z:\\here\\\0".to_vec()))
        );
        assert!(normalise(&[b'a'; LONGEST], &case, none).is_err());
    }
}
