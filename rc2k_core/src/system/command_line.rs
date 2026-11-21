//! The command line's switches. The first word is the program; of the rest only words of two
//! characters or more count, by their first two: `-v` (version shown, game left; later words
//! not read), `-u`, `-a` (no audio), `-c` (the next word taken and not used: it named a 3D
//! card to leave out). The line ends at its NUL.

use crate::Invalid;
use crate::strings::bytes::Bytes;
use crate::strings::words::next_argument;

/// What the line asks for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Switches {
    /// The program's word (start, length).
    pub program: (usize, usize),
    pub version: bool,
    pub u: bool,
    pub no_audio: bool,
}

/// The switches of the line in `text` (to its NUL). Refused: a text without its NUL.
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
            [b'-', b'a'] => s.no_audio = true,
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
        let s = read(&b"ral.exe -a x -c 2 -u\0"[..]).unwrap();
        assert_eq!(s.program, (0, 7));
        assert!(s.no_audio && !s.version && s.u);
        // `-i` and `-p` are no switches: they do nothing, and their words are not taken.
        let s = read(&b"ral.exe -i one two 7 8 -p\0"[..]).unwrap();
        assert_eq!(
            s,
            Switches {
                program: (0, 7),
                ..Switches::default()
            }
        );
        let s = read(&b"\"a b\" -v -u\0"[..]).unwrap();
        assert!(read(&b"ral -u   \0"[..]).unwrap().u);
        assert!(s.version && !s.u);
    }
}
