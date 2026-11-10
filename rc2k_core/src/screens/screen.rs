//! What a menu screen draws with (2026-10-20, the screens' conversion): texts and sprites,
//! and the work variables the original leaves behind. Each screen family names its own
//! fonts, texts and work variables (`Screen`'s associated types); the crate's adapter draws
//! them with the game's fonts and blitter.

use crate::Invalid;

/// What drawing a text leaves: the y of its last line and whether a line was too wide for
/// a font that says so (the carry flag). (The original also leaves where the text ended in
/// ebx, which some screens hand the blitter as a sprite's palette, never looked at.)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawn {
    pub y: i32,
    pub too_wide: bool,
}

/// The game's two shared text buffers the screens build their lines in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buffer {
    /// The line buffer (0x100 bytes).
    Line,
    /// The text buffer (the descriptions').
    Text,
    /// `byte_A8BC54`, after the line buffer (0x100 bytes).
    Cells,
}

/// A bar filled to a percentage (the game's six bar globals): the fill sprite up to
/// `percent` of `width`, an end piece there; nothing at 0 percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub fill_glyph: u32,
    pub end_glyph: u32,
    pub percent: i32,
}

/// An edge of the clip rectangle (`g_clipLeft` .. `g_clipBottom`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// The fatal error a bar whose width x percent is 0 (with a percent) reports.
pub const BAR_HAS_NO_LENGTH: u32 = 0x8070_1010;

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
    /// What a buffer holds, a clock ("09:05") drawn in fixed columns ending at `right`.
    fn draw_clock(
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
    fn sprite(&mut self, glyph: u32, x: i32, y: i32) -> Result<(), Self::Error>;
    /// A work variable set.
    fn keep(&mut self, work: Self::Work, value: u32);
    /// A bar drawn.
    fn bar(&mut self, bar: Bar) -> Result<(), Self::Error>;
    /// The error for the game's fatal error `code`.
    fn fatal(&mut self, code: u32, message: &'static str) -> Self::Error;
    /// `bytes` (no NUL, 3 .. 0x80 of them) put in the line buffer with a NUL and drawn
    /// spaced out, the last character's column at `x` + 0x32
    /// (`screens::draw_spaced_number`). A screen that draws none refuses.
    fn spaced(
        &mut self,
        _font: Self::Font,
        _bytes: &[u8],
        _x: i32,
        _y: i32,
    ) -> Result<(), Self::Error> {
        Err(Invalid("this screen draws no spaced numbers").into())
    }
    /// A text the game keeps measured from x (`text::measure_text`): its widest line's
    /// width. A screen that measures nothing refuses.
    fn measure(
        &mut self,
        _font: Self::Font,
        _text: Self::Text,
        _x: i32,
    ) -> Result<u32, Self::Error> {
        Err(Invalid("this screen measures no text").into())
    }
    /// String `id` measured from x, as `measure`. A screen that measures nothing refuses.
    fn measure_string(&mut self, _font: Self::Font, _id: u32, _x: i32) -> Result<u32, Self::Error> {
        Err(Invalid("this screen measures no text").into())
    }
    /// The font `a` blended with `b` by the time since the last key (half a pulse 0x100),
    /// into the family's blended font (`text::blend_font`, `stru_51E384`). A screen that
    /// blends none refuses.
    fn blend(
        &mut self,
        _since_key: u32,
        _a: Self::Font,
        _b: Self::Font,
    ) -> Result<(), Self::Error> {
        Err(Invalid("this screen blends no fonts").into())
    }
    /// An edge of the clip rectangle. A screen that clips nothing refuses.
    fn clip_edge(&mut self, _edge: Edge) -> Result<u32, Self::Error> {
        Err(Invalid("this screen clips nothing").into())
    }
    /// An edge of the clip rectangle set. A screen that clips nothing refuses.
    fn set_clip_edge(&mut self, _edge: Edge, _value: u32) -> Result<(), Self::Error> {
        Err(Invalid("this screen clips nothing").into())
    }
    /// Item `n` of the current menu made choosable or greyed out (`menu::enable_item`,
    /// `menu::disable_item`). A screen without a menu refuses.
    fn item(&mut self, _n: u32, _on: bool) -> Result<(), Self::Error> {
        Err(Invalid("this screen has no menu").into())
    }
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
