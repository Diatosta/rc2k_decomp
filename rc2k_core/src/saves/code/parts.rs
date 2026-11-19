//! A code too long for one QR code, cut into numbered parts, and the parts put together.
//!
//! A part's text is `RC2P:`, the part's number and the count (two digits each), the whole
//! text's check number (eight hex digits), `:`, a piece of the whole text, `:`. Every part of
//! one code has the same count and check number, so parts of two codes never mix, and the
//! pieces joined in order give the whole text, which must have that check number. The letters
//! are the QR code's alphanumeric ones, as the whole text's.

use super::{CodeError, crc32, qr};

/// A part's text begins with this.
pub const START: &str = "RC2P:";
/// What a part's text carries besides its piece: the start, number, count, check number and
/// the two colons.
const OVERHEAD: usize = START.len() + 2 + 2 + 8 + 2;
/// The most parts a code has (the count is two digits).
pub const MOST_PARTS: usize = 99;
/// The most letters of the whole text one part carries.
const MOST_PIECE: usize = qr::MOST_LETTERS - OVERHEAD;
/// The letters of the whole text a part carries when the code has parts to spare: a QR code of
/// about 60 modules a side, whose modules a camera at 1280 x 720 resolves when the code fills
/// half the picture's height (larger codes read only from close by or from a bigger picture).
pub const PREFERRED_PIECE: usize = 380;

/// The texts the QR codes of `whole` carry, in order: `whole` itself when it is short enough
/// for a small QR code, else its parts (pieces of equal length, give or take a letter): as many
/// as make each a small code, or fewer and larger ones when the most parts would not hold it
/// so. `None` when even the largest codes in the most parts do not hold it.
pub fn split(whole: &str) -> Option<Vec<String>> {
    if whole.len() <= PREFERRED_PIECE + OVERHEAD {
        return Some(vec![whole.to_owned()]);
    }
    let count = whole.len().div_ceil(PREFERRED_PIECE).min(MOST_PARTS);
    if whole.len().div_ceil(count) > MOST_PIECE {
        return None;
    }
    let id = crc32(whole.as_bytes());
    let piece = whole.len().div_ceil(count);
    Some(
        whole
            .as_bytes()
            .chunks(piece)
            .enumerate()
            .map(|(n, chunk)| {
                // The text is of ASCII letters, so a cut is always between letters.
                let chunk = std::str::from_utf8(chunk).unwrap_or("");
                format!("{START}{:02}{count:02}{id:08X}:{chunk}:", n + 1)
            })
            .collect(),
    )
}

/// One part, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    /// From 1.
    pub number: usize,
    pub count: usize,
    /// The whole text's check number.
    pub id: u32,
    pub piece: String,
}

impl Part {
    /// A part from its text; `None` when the text is not a part's at all.
    pub fn read(text: &str) -> Option<Self> {
        let text = text.trim();
        let rest = text.strip_prefix(START)?.strip_suffix(':')?;
        let (head, piece) = rest.split_at_checked(12)?;
        let piece = piece.strip_prefix(':')?;
        let digits = |s: &str| {
            s.bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| s.parse::<usize>().ok())
                .flatten()
        };
        let number = digits(&head[..2])?;
        let count = digits(&head[2..4])?;
        if head[4..].bytes().any(|b| !b.is_ascii_hexdigit()) {
            return None;
        }
        let id = u32::from_str_radix(&head[4..], 16).ok()?;
        (number >= 1 && number <= count && count >= 2).then(|| Self {
            number,
            count,
            id,
            piece: piece.to_owned(),
        })
    }
}

/// What taking a part in did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taken {
    /// A new part of the code being collected.
    New,
    /// A part already held.
    Again,
    /// A part of another code: not taken.
    Other,
}

/// The parts of one code, collected in any order.
#[derive(Debug, Default)]
pub struct Assembly {
    held: Option<Held>,
}

#[derive(Debug)]
struct Held {
    id: u32,
    pieces: Vec<Option<String>>,
}

impl Assembly {
    /// Takes a part. The first part decides which code the rest must belong to.
    pub fn take(&mut self, part: Part) -> Taken {
        match &mut self.held {
            None => {
                let mut pieces = vec![None; part.count];
                pieces[part.number - 1] = Some(part.piece);
                self.held = Some(Held {
                    id: part.id,
                    pieces,
                });
                Taken::New
            }
            Some(held) if held.id != part.id || held.pieces.len() != part.count => Taken::Other,
            Some(held) => {
                let slot = &mut held.pieces[part.number - 1];
                if slot.is_some() {
                    Taken::Again
                } else {
                    *slot = Some(part.piece);
                    Taken::New
                }
            }
        }
    }

    /// How many parts the code has (0 before the first is taken).
    pub fn count(&self) -> usize {
        self.held.as_ref().map_or(0, |h| h.pieces.len())
    }

    /// The numbers (from 1) of the parts held.
    pub fn numbers(&self) -> Vec<usize> {
        self.held.as_ref().map_or_else(Vec::new, |h| {
            (0..h.pieces.len())
                .filter(|&n| h.pieces[n].is_some())
                .map(|n| n + 1)
                .collect()
        })
    }

    /// The whole text once every part is held; refused if its check number is not the parts'.
    pub fn whole(&self) -> Option<Result<String, CodeError>> {
        let held = self.held.as_ref()?;
        let mut text = String::new();
        for piece in &held.pieces {
            text.push_str(piece.as_deref()?);
        }
        Some(if crc32(text.as_bytes()) == held.id {
            Ok(text)
        } else {
            Err(CodeError::Checksum)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(n: usize) -> String {
        let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
        (0..n)
            .map(|i| char::from(alphabet[(i * i + 3 * i + i / 7) % 45]))
            .collect()
    }

    #[test]
    fn a_short_text_is_one_qr_code_as_it_is() {
        let t = text(PREFERRED_PIECE + OVERHEAD);
        assert_eq!(split(&t), Some(vec![t]));
    }

    #[test]
    fn parts_are_small_codes_until_there_are_too_many() {
        // Under the most parts at the preferred size: every part small.
        for n in [
            PREFERRED_PIECE + OVERHEAD + 1,
            2000,
            9000,
            MOST_PARTS * PREFERRED_PIECE,
        ] {
            let parts = split(&text(n)).unwrap();
            assert_eq!(
                parts.len(),
                n.div_ceil(PREFERRED_PIECE).min(MOST_PARTS),
                "{n}"
            );
            for p in &parts {
                assert!(p.len() <= PREFERRED_PIECE + OVERHEAD, "{n}: {}", p.len());
                let modules = qr::encode(p).unwrap();
                assert!(modules.side <= 65, "{n}: {}", modules.side);
            }
        }
        // Past it, the most parts, each as large as it must be.
        let parts = split(&text(MOST_PARTS * PREFERRED_PIECE + 1000)).unwrap();
        assert_eq!(parts.len(), MOST_PARTS);
    }

    #[test]
    fn parts_fit_a_qr_code_and_join_in_any_order() {
        for n in [qr::MOST_LETTERS + 1, 9000, 12_000, 40_000, 90_000] {
            let whole = text(n);
            let parts = split(&whole).unwrap();
            assert!(parts.len() >= 2);
            for p in &parts {
                assert!(p.len() <= qr::MOST_LETTERS, "{n}: {}", p.len());
                assert!(qr::encode(p).is_some());
            }
            let mut order: Vec<usize> = (0..parts.len()).collect();
            order.reverse();
            let half = order.len() / 2;
            order.rotate_left(half);
            let mut assembly = Assembly::default();
            for (k, &i) in order.iter().enumerate() {
                assert!(assembly.whole().is_none());
                assert_eq!(assembly.take(Part::read(&parts[i]).unwrap()), Taken::New);
                assert_eq!(assembly.numbers().len(), k + 1);
            }
            assert_eq!(assembly.whole(), Some(Ok(whole)));
        }
    }

    #[test]
    fn parts_of_two_codes_do_not_mix() {
        let a = split(&text(9000)).unwrap();
        let b = split(&text(9001)).unwrap();
        let mut assembly = Assembly::default();
        assert_eq!(assembly.take(Part::read(&a[0]).unwrap()), Taken::New);
        assert_eq!(assembly.take(Part::read(&b[1]).unwrap()), Taken::Other);
        assert_eq!(assembly.take(Part::read(&a[0]).unwrap()), Taken::Again);
        assert_eq!(assembly.numbers(), vec![1]);
    }

    #[test]
    fn a_wrong_piece_fails_the_check_number() {
        let parts = split(&text(9000)).unwrap();
        let mut assembly = Assembly::default();
        for (n, p) in parts.iter().enumerate() {
            let mut part = Part::read(p).unwrap();
            if n == 1 {
                part.piece.replace_range(..1, "~");
            }
            assembly.take(part);
        }
        assert_eq!(assembly.whole(), Some(Err(CodeError::Checksum)));
    }

    #[test]
    fn texts_that_are_not_parts_are_not_read() {
        for bad in [
            "RC2K:ABC:",
            "RC2P:",
            "RC2P:0102AAAAAAAA:x",
            "RC2P:0002AAAAAAAA:x:",
            "RC2P:0302AAAAAAAA:x:",
            "RC2P:0101AAAAAAAA:x:",
            "RC2P:0102AAAAAAAG:x:",
            "RC2P:0102AAAAAAAA-x:",
            "RC2P:0102AAA:x:",
        ] {
            assert_eq!(Part::read(bad), None, "{bad}");
        }
        assert!(Part::read("RC2P:0102AAAAAAAA:x:").is_some());
    }

    #[test]
    fn too_long_a_text_has_no_parts() {
        assert!(split(&text(MOST_PARTS * MOST_PIECE + 1)).is_none());
        assert!(split(&text(MOST_PARTS * MOST_PIECE)).is_some());
    }
}
