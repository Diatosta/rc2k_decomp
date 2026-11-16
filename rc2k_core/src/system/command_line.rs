//! The command line's switches. The first word is the program; of the rest only words of two
//! characters or more count, by their first two: `-v` (version shown, game left; later words
//! not read), `-u`, `-p` (installing), `-a` (no audio), `-i` (the next two words copied, the
//! first characters of the two after them less `0`), `-c` (the next word taken and not used:
//! it named a 3D card to leave out). The line ends at its NUL.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::words::next_argument;

/// The room for a word after `-i`.
pub const I_WORD_ROOM: usize = 0x100;

/// One `-i`: its two words (start, length) and two digits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Install {
    pub words: [(usize, usize); 2],
    pub digits: [u32; 2],
}

/// What the line asks for. Each `-i` is kept, in order (a later one's words are copied over
/// the earlier's, without a NUL).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Switches {
    /// The program's word (start, length).
    pub program: (usize, usize),
    pub version: bool,
    pub u: bool,
    pub installing: bool,
    pub no_audio: bool,
    pub install: Vec<Install>,
}

/// The switches of the line in `text` (to its NUL). Refused: a text without its NUL, a word
/// after `-i` of `I_WORD_ROOM` bytes or more.
pub fn read(text: &(impl Bytes + ?Sized)) -> Result<Switches, Invalid> {
    let byte = |p: usize| text.byte(p).ok_or(Invalid("a text without its NUL"));
    let first = next_argument(text, 0)?;
    let mut s = Switches {
        program: (first.start, first.len),
        ..Switches::default()
    };
    let mut from = first.next;
    let word = |from: &mut usize| -> Result<(usize, usize), Invalid> {
        let w = next_argument(text, *from)?;
        *from = w.next;
        Ok((w.start, w.len))
    };
    loop {
        let (start, len) = word(&mut from)?;
        if len == 0 && start == from {
            return Ok(s);
        }
        if len < 2 {
            continue;
        }
        match [byte(start)?, byte(start + 1)?] {
            [b'-', b'v'] => {
                s.version = true;
                return Ok(s);
            }
            [b'-', b'u'] => s.u = true,
            [b'-', b'p'] => s.installing = true,
            [b'-', b'a'] => s.no_audio = true,
            [b'-', b'i'] => {
                let mut i = Install::default();
                for w in &mut i.words {
                    *w = word(&mut from)?;
                    if w.1 >= I_WORD_ROOM {
                        return Err(Invalid("a word after -i longer than its place"));
                    }
                }
                for d in &mut i.digits {
                    let (start, len) = word(&mut from)?;
                    *d = if len == 0 {
                        0
                    } else {
                        (byte(start)? as u32).wrapping_sub(b'0' as u32)
                    };
                }
                s.install.push(i);
            }
            [b'-', b'c'] => {
                // The 3D card to leave out: its word is taken, there is one card.
                word(&mut from)?;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line() {
        let s = read(&b"ral.exe -a x -c 2 -i one two 7\0"[..]).unwrap();
        assert_eq!(s.program, (0, 7));
        assert!(s.no_audio && !s.version);
        assert_eq!(
            s.install,
            vec![Install {
                words: [(21, 3), (25, 3)],
                digits: [7, 0]
            }]
        );
        let s = read(&b"\"a b\" -v -u\0"[..]).unwrap();
        assert!(read(&b"ral -u   \0"[..]).unwrap().u);
        assert!(s.version && !s.u);
    }
}
