//! Reading the game's packed data (texts, glyph tables) a byte at a time. A slice answers
//! `None` past its end, so the core never reads out of bounds.

/// Bytes at offsets from a start.
pub trait Bytes {
    /// The byte at `at`, `None` past the end.
    fn byte(&self, at: usize) -> Option<u8>;

    /// The `len` bytes from `at` in one piece, when they are held that way: a faster way to
    /// the same bytes, `None` meaning only that the caller goes byte by byte.
    fn run(&self, _at: usize, _len: usize) -> Option<&[u8]> {
        None
    }

    /// Fills `out` with the bytes from `at`; `None` (with `out` unspecified) when any lies
    /// past the end. The same bytes `byte` gives one by one, in one read where the holder
    /// can.
    fn copy_run(&self, at: usize, out: &mut [u8]) -> Option<()> {
        for (i, place) in out.iter_mut().enumerate() {
            *place = self.byte(at.checked_add(i)?)?;
        }
        Some(())
    }

    fn u32_le(&self, at: usize) -> Option<u32> {
        let mut word = [0; 4];
        self.copy_run(at, &mut word)?;
        Some(u32::from_le_bytes(word))
    }
}

impl Bytes for [u8] {
    fn byte(&self, at: usize) -> Option<u8> {
        self.get(at).copied()
    }
    fn run(&self, at: usize, len: usize) -> Option<&[u8]> {
        self.get(at..at.checked_add(len)?)
    }
}

impl<const N: usize> Bytes for [u8; N] {
    fn byte(&self, at: usize) -> Option<u8> {
        self.get(at).copied()
    }
}

/// A borrowed slice (sized, so it can be `&dyn Bytes`).
impl Bytes for &[u8] {
    fn byte(&self, at: usize) -> Option<u8> {
        self.get(at).copied()
    }
}

impl Bytes for Vec<u8> {
    fn byte(&self, at: usize) -> Option<u8> {
        self.get(at).copied()
    }
}

/// The bytes of `B` from offset `start` on.
pub struct Offset<'a, B: ?Sized> {
    pub bytes: &'a B,
    pub start: usize,
}

impl<B: Bytes + ?Sized> Bytes for Offset<'_, B> {
    fn byte(&self, at: usize) -> Option<u8> {
        self.bytes.byte(self.start.checked_add(at)?)
    }
}
