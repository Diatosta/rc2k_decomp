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
/// How much of the whole text one QR code carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Density {
    /// Large codes (about 117 modules a side, version 25): few codes for a save, read by a
    /// camera of 1080p or better that sees the code fill about half the picture's height.
    Dense,
    /// Small codes (about 60 modules a side, version 12): more of them, but a camera at
    /// 1280 x 720 resolves their modules when the code fills half the picture's height.
    Small,
}

impl Density {
    /// The letters of the whole text a part carries.
    pub const fn piece(self) -> usize {
        match self {
            Self::Dense => DENSE_PIECE,
            Self::Small => SMALL_PIECE,
        }
    }

    /// The other density.
    pub const fn other(self) -> Self {
        match self {
            Self::Dense => Self::Small,
            Self::Small => Self::Dense,
        }
    }
}

/// The letters of the whole text a dense part carries.
const DENSE_PIECE: usize = 1800;
/// The letters of the whole text a small part carries when the code has parts to spare.
const SMALL_PIECE: usize = 380;

/// The texts the QR codes of `whole` carry, in order: `whole` itself when it is short enough
/// for one code of the density, else its parts (pieces of equal length, give or take a letter):
/// as many as make each a code of the density, or fewer and larger ones when the most parts
/// would not hold it so. `None` when even the largest codes in the most parts do not hold it.
pub fn split(whole: &str, density: Density) -> Option<Vec<String>> {
    let preferred = density.piece();
    if whole.len() <= preferred + OVERHEAD {
        return Some(vec![whole.to_owned()]);
    }
    let count = whole.len().div_ceil(preferred).min(MOST_PARTS);
    if whole.len().div_ceil(count) > MOST_PIECE {
        return None;
    }
    let id = crc32(whole.as_bytes());
    let piece = whole.len().div_ceil(count);
    // The pieces of `piece` letters may leave fewer parts than `count`; the count named in each
    // part is the one made.
    let count = whole.len().div_ceil(piece);
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
        let t = text(SMALL_PIECE + OVERHEAD);
        assert_eq!(split(&t, Density::Small), Some(vec![t]));
    }

    #[test]
    fn parts_are_small_codes_until_there_are_too_many() {
        // Under the most parts at the preferred size: every part small.
        for n in [
            SMALL_PIECE + OVERHEAD + 1,
            2000,
            9000,
            MOST_PARTS * SMALL_PIECE,
        ] {
            let parts = split(&text(n), Density::Small).unwrap();
            assert_eq!(parts.len(), n.div_ceil(SMALL_PIECE).min(MOST_PARTS), "{n}");
            for p in &parts {
                assert!(p.len() <= SMALL_PIECE + OVERHEAD, "{n}: {}", p.len());
                let modules = qr::encode(p).unwrap();
                assert!(modules.side <= 65, "{n}: {}", modules.side);
            }
        }
        // Past it, the most parts, each as large as it must be.
        let parts = split(&text(MOST_PARTS * SMALL_PIECE + 1000), Density::Small).unwrap();
        assert_eq!(parts.len(), MOST_PARTS);
    }

    #[test]
    fn parts_fit_a_qr_code_and_join_in_any_order() {
        for n in [qr::MOST_LETTERS + 1, 9000, 12_000, 40_000, 90_000] {
            let whole = text(n);
            let parts = split(&whole, Density::Small).unwrap();
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
    fn dense_parts_are_fewer_larger_codes_and_never_mix_with_small_ones() {
        for n in [DENSE_PIECE + OVERHEAD + 1, 6000, 20_000] {
            let whole = text(n);
            let dense = split(&whole, Density::Dense).unwrap();
            let small = split(&whole, Density::Small).unwrap();
            assert_eq!(dense.len(), n.div_ceil(DENSE_PIECE), "{n}");
            assert!(dense.len() * 4 <= small.len() + 3, "{n}");
            for p in &dense {
                assert!(p.len() <= DENSE_PIECE + OVERHEAD);
                assert!(qr::encode(p).unwrap().side <= 117, "{n}");
            }
            // One set's count is not the other's, so a part of one is another set's to the other.
            let mut assembly = Assembly::default();
            assert_eq!(assembly.take(Part::read(&dense[0]).unwrap()), Taken::New);
            assert_eq!(assembly.take(Part::read(&small[1]).unwrap()), Taken::Other);
            for p in &dense[1..] {
                assembly.take(Part::read(p).unwrap());
            }
            assert_eq!(assembly.whole(), Some(Ok(whole)));
        }
        // A text a dense code holds whole is one code, small or not.
        let short = text(DENSE_PIECE);
        assert_eq!(split(&short, Density::Dense), Some(vec![short.clone()]));
        assert!(split(&short, Density::Small).unwrap().len() > 1);
    }

    #[test]
    fn parts_of_two_codes_do_not_mix() {
        let a = split(&text(9000), Density::Small).unwrap();
        let b = split(&text(9001), Density::Small).unwrap();
        let mut assembly = Assembly::default();
        assert_eq!(assembly.take(Part::read(&a[0]).unwrap()), Taken::New);
        assert_eq!(assembly.take(Part::read(&b[1]).unwrap()), Taken::Other);
        assert_eq!(assembly.take(Part::read(&a[0]).unwrap()), Taken::Again);
        assert_eq!(assembly.numbers(), vec![1]);
    }

    #[test]
    fn a_wrong_piece_fails_the_check_number() {
        let parts = split(&text(9000), Density::Small).unwrap();
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
        assert!(split(&text(MOST_PARTS * MOST_PIECE + 1), Density::Small).is_none());
        assert!(split(&text(MOST_PARTS * MOST_PIECE), Density::Small).is_some());
    }
}
