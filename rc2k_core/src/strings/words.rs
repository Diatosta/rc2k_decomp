//! Words cut off a command line or a settings value.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A word: where it starts, its length, and where to go on from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Word {
    pub start: usize,
    pub len: usize,
    pub next: usize,
}

/// The next word from `at`: spaces and tabs before it passed over; it ends at a space, a
/// tab or a carriage return (passed over) or at the text's end (its NUL, not passed). A
/// word that starts with `"` is what follows up to the next `"` or carriage return
/// (passed over) or the end. Refused: a text without its NUL.
pub(crate) fn next_argument(text: &(impl Bytes + ?Sized), at: usize) -> Result<Word, Invalid> {
    let byte = |p: usize| text.byte(p).ok_or(Invalid("a text without its NUL"));
    let mut start = at;
    while matches!(byte(start)?, b' ' | b'\t') {
        start += 1;
    }
    let quoted = byte(start)? == b'"';
    if quoted {
        start += 1;
    }
    let mut p = start;
    loop {
        let ends = match byte(p)? {
            0 => {
                return Ok(Word {
                    start,
                    len: p - start,
                    next: p,
                });
            }
            b'\r' => true,
            b'"' => quoted,
            b' ' | b'\t' => !quoted,
            _ => false,
        };
        if ends {
            return Ok(Word {
                start,
                len: p - start,
                next: p + 1,
            });
        }
        p += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        let t = b"  ab\t\"c d\" e\r\0";
        let w = next_argument(t, 0).unwrap();
        assert_eq!(
            w,
            Word {
                start: 2,
                len: 2,
                next: 5
            }
        );
        let w = next_argument(t, w.next).unwrap();
        assert_eq!((&t[w.start..w.start + w.len], w.next), (&b"c d"[..], 10));
        let w = next_argument(t, w.next).unwrap();
        assert_eq!((&t[w.start..w.start + w.len], w.next), (&b"e"[..], 13));
        let w = next_argument(t, w.next).unwrap();
        assert_eq!((w.len, w.next), (0, 13));
        assert!(next_argument(b"ab", 0).is_err());
    }
}
