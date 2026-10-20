//! What a menu screen draws with (2026-10-20, the screens' conversion): texts and sprites,
//! and the work variables the original leaves behind. Each screen family names its own
//! fonts, texts and work variables (`Screen`'s associated types); the hook crate's adapter
//! draws them with the game's fonts and blitter.

use crate::Invalid;

/// What drawing a text leaves: the y of its last line, where the text ended (the
/// original's ebx), which some screens hand the blitter as a palette, and whether a line
/// was too wide for a font that says so (the carry flag).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawn {
    pub y: i32,
    pub after: u32,
    pub too_wide: bool,
}

/// The game's two shared text buffers the screens build their lines in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    /// The line buffer (0x100 bytes).
    Line,
    /// The text buffer (the descriptions').
    Text,
}

/// A screen being drawn.
pub trait Screen {
    /// The fonts the family draws with.
    type Font: Copy;
    /// Texts the game keeps (a name in a table), drawn where they are.
    type Text: Copy;
    /// The work variables the family leaves.
    type Work: Copy;
    type Error: From<Invalid>;

    /// A text the game keeps, at x, y.
    fn text(
        &mut self,
        font: Self::Font,
        text: Self::Text,
        x: i32,
        y: i32,
    ) -> Result<Drawn, Self::Error>;
    /// `bytes` (with their NUL) written into a buffer.
    fn store(&mut self, buffer: Buffer, bytes: &[u8]) -> Result<(), Self::Error>;
    /// What a buffer holds, drawn at x, y.
    fn draw(
        &mut self,
        font: Self::Font,
        buffer: Buffer,
        x: i32,
        y: i32,
    ) -> Result<Drawn, Self::Error>;
    /// What a buffer holds, a time drawn in fixed columns ending at `right`.
    fn draw_time(
        &mut self,
        font: Self::Font,
        buffer: Buffer,
        right: i32,
        y: i32,
    ) -> Result<(), Self::Error>;
    /// String `id` of the language's, drawn.
    fn string(&mut self, font: Self::Font, id: u32, x: i32, y: i32) -> Result<Drawn, Self::Error>;
    /// String `id`'s bytes (no NUL).
    fn string_bytes(&mut self, id: u32) -> Vec<u8>;
    /// `count` strings from `first` as one text with its NUL (`strings::text_from_ids`),
    /// for the text buffer.
    fn strings(&mut self, first: u32, count: u32) -> Result<Vec<u8>, Self::Error>;
    /// Sprite `glyph` of the sprite table.
    fn sprite(&mut self, glyph: u32, x: i32, y: i32, palette: u32) -> Result<(), Self::Error>;
    /// A work variable set.
    fn keep(&mut self, work: Self::Work, value: u32);

    /// `bytes` (no NUL) put in the line buffer with a NUL and drawn from there.
    fn line(
        &mut self,
        font: Self::Font,
        bytes: &[u8],
        x: i32,
        y: i32,
    ) -> Result<Drawn, Self::Error> {
        let mut text = bytes.to_vec();
        text.push(0);
        self.store(Buffer::Line, &text)?;
        self.draw(font, Buffer::Line, x, y)
    }

    /// A time (no NUL) put in the line buffer and drawn in fixed columns ending at `right`.
    fn time_line(
        &mut self,
        font: Self::Font,
        bytes: &[u8],
        right: i32,
        y: i32,
    ) -> Result<(), Self::Error> {
        let mut text = bytes.to_vec();
        text.push(0);
        self.store(Buffer::Line, &text)?;
        self.draw_time(font, Buffer::Line, right, y)
    }
}
