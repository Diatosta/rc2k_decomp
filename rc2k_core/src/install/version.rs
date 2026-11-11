//! Whether the installed `ral.exe` is this same build (from
//! `install::check_installed_version` 0x413B0B): the file read in chunks of 0xFF00 bytes,
//! each scanned (with the last 0x100 bytes of the one before in front of it) for
//! "$VER: Rally 5"; at the first one, the text after the version's NUL compared with the
//! game's copyright line up to its first '.'.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// The version's marker.
pub const MARKER: &[u8; 13] = b"$VER: Rally 5";
/// The bytes read at a time, and scanned.
pub const CHUNK: u32 = 0xFF00;
/// The bytes of the chunk before carried to the front.
pub const CARRIED: u32 = 0x100;
/// The most bytes looked through for the version's NUL (less one).
const TO_NUL: u32 = 0x80;

/// How a chunk's scan ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scan {
    /// No marker: the next chunk is read.
    Nothing,
    /// The first marker found ends the reading: the same build or not, and where the scan
    /// stopped (its offset from the buffer's start, and the matched length).
    Stopped { same: bool, at: u32, count: u32 },
}

fn at(bytes: &(impl Bytes + ?Sized), n: u32) -> Result<u8, Invalid> {
    bytes
        .byte(n as usize)
        .ok_or(Invalid("a version scan past its buffer"))
}

/// One chunk scanned (`buffer`: the carried bytes and the chunk, and the bytes after it
/// that a marker at its end reaches), against `copyright`.
pub fn scan(
    buffer: &(impl Bytes + ?Sized),
    copyright: &(impl Bytes + ?Sized),
) -> Result<Scan, Invalid> {
    for start in 0..CHUNK {
        if at(buffer, start)? != b'$' {
            continue;
        }
        let mut hit = true;
        for (k, &m) in MARKER.iter().enumerate().skip(1) {
            if at(buffer, start + k as u32)? != m {
                hit = false;
                break;
            }
        }
        if !hit {
            continue;
        }
        let mut position = start;
        let mut left = TO_NUL;
        loop {
            left -= 1;
            if left == 0 {
                return Ok(Scan::Stopped {
                    same: false,
                    at: position,
                    count: 0,
                });
            }
            let b = at(buffer, position)?;
            position += 1;
            if b < 1 {
                break;
            }
        }
        let mut n = 0u32;
        loop {
            let want = at(copyright, n)?;
            if want != at(buffer, position + n)? {
                return Ok(Scan::Stopped {
                    same: false,
                    at: position,
                    count: n,
                });
            }
            n += 1;
            if want == b'.' {
                return Ok(Scan::Stopped {
                    same: true,
                    at: position,
                    count: n,
                });
            }
        }
    }
    Ok(Scan::Nothing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer(at: usize, text: &[u8]) -> Vec<u8> {
        let mut b = vec![0x20u8; (CHUNK + 0x100) as usize];
        b[at..at + text.len()].copy_from_slice(text);
        b
    }

    #[test]
    fn scanning() {
        let copyright = b"(c) 1999 Someone. More";
        assert_eq!(scan(&buffer(0, b"nothing"), copyright), Ok(Scan::Nothing));
        let same = buffer(0x10, b"$VER: Rally 5.01\0(c) 1999 Someone.");
        assert_eq!(
            scan(&same, copyright),
            Ok(Scan::Stopped {
                same: true,
                at: 0x21,
                count: 17
            })
        );
        let other = buffer(0x10, b"$VER: Rally 5.01\0(c) 2000 Someone.");
        assert_eq!(
            scan(&other, copyright),
            Ok(Scan::Stopped {
                same: false,
                at: 0x21,
                count: 4
            })
        );
        let mut long = buffer(0x10, b"$VER: Rally 5");
        long[0x1D..0x200].fill(b'x');
        assert_eq!(
            scan(&long, copyright),
            Ok(Scan::Stopped {
                same: false,
                at: 0x10 + 0x7F,
                count: 0
            })
        );
        assert_eq!(
            scan(&buffer(0, b"$VER: Rally 4"), copyright),
            Ok(Scan::Nothing)
        );
        // A marker at the chunk's end reads into the bytes after it.
        let end = buffer(CHUNK as usize - 1, b"$VER: Rally 5\0(c) 1999 Someone.");
        assert!(matches!(
            scan(&end, copyright),
            Ok(Scan::Stopped { same: true, .. })
        ));
        assert!(scan(&[0u8; 4], copyright).is_err());
    }
}
