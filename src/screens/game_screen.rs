//! The game's side of `rc2k_core::screen::Screen` (2026-10-20, the screens' conversion): a
//! family's fonts are font records in game memory, its texts game addresses, its work
//! variables game dwords; drawn through the text functions and the canvas.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_write, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::{font_at, line_buffer};
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::font::Font;
use rc2k_core::screen::{Buffer, Drawn, Screen};
use std::marker::PhantomData;

/// A family's font: the address of its record (as IDA shows it).
pub trait GameFont: Copy {
    fn address(self) -> usize;
}

/// A family's work variable: its address (as IDA shows it).
pub trait GameWork: Copy {
    fn address(self) -> usize;
}

pub struct GameScreen<'a, F, W> {
    pub canvas: &'a mut dyn Canvas,
    sprites: u32,
    family: PhantomData<(F, W)>,
}

impl<'a, F, W> GameScreen<'a, F, W> {
    pub fn new(canvas: &'a mut dyn Canvas) -> Self {
        Self {
            canvas,
            sprites: crate::gfx::blit_marker_pair::sprite_table(),
            family: PhantomData,
        }
    }
}

fn font_of(f: impl GameFont) -> *mut Font {
    font_at(f.address())
}

/// Where a shared buffer is.
pub fn buffer_at(buffer: Buffer) -> *mut u8 {
    match buffer {
        Buffer::Line => line_buffer(),
        Buffer::Text => crate::strings::text_from_ids::text(),
    }
}

/// `bytes` (with their NUL) into a shared buffer; refused past its end (the line buffer
/// 0x100 bytes: `byte_A8BC54` follows).
pub unsafe fn store_in(buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
    let room = match buffer {
        Buffer::Line => 0x100,
        Buffer::Text => crate::strings::text_from_ids::TEXT_LEN,
    };
    if bytes.len() > room {
        return Err(BadInput("a text past its buffer").into());
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer_at(buffer), bytes.len()) };
    Ok(())
}

impl<F: GameFont, W: GameWork> Screen for GameScreen<'_, F, W> {
    type Font = F;
    type Text = u32;
    type Work = W;
    type Error = Refused;

    fn text(&mut self, font: F, text: u32, x: i32, y: i32) -> Result<Drawn, Refused> {
        let out = unsafe { draw_text_at(font_of(font), text as *const u8, x, y, self.canvas) }?;
        Ok(Drawn {
            y: out.y,
            after: out.text as u32,
            too_wide: out.too_wide,
        })
    }

    fn store(&mut self, buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
        unsafe { store_in(buffer, bytes) }
    }

    fn draw(&mut self, font: F, buffer: Buffer, x: i32, y: i32) -> Result<Drawn, Refused> {
        self.text(font, buffer_at(buffer) as u32, x, y)
    }

    fn draw_time(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
        unsafe {
            draw_time_fixed_pitch_at(font_of(font), buffer_at(buffer), right, y, self.canvas)
        }?;
        Ok(())
    }

    fn string(&mut self, font: F, id: u32, x: i32, y: i32) -> Result<Drawn, Refused> {
        use crate::text::draw_string_by_id::draw_string_by_id_at;
        let out = unsafe { draw_string_by_id_at(id, font_of(font), x, y, self.canvas) }?;
        Ok(Drawn {
            y: out.y,
            after: out.text as u32,
            too_wide: out.too_wide,
        })
    }

    fn string_bytes(&mut self, id: u32) -> Vec<u8> {
        let mut out = Vec::new();
        unsafe { crate::strings::append_string_by_id::push_string(&mut out, id) };
        out
    }

    fn strings(&mut self, first: u32, count: u32) -> Result<Vec<u8>, Refused> {
        Ok(unsafe { crate::strings::text_from_ids::text_from_ids(first, count) }?)
    }

    fn sprite(&mut self, glyph: u32, x: i32, y: i32, palette: u32) -> Result<(), Refused> {
        let call = BlitCall::new(glyph, x, y, palette, self.sprites);
        Ok(unsafe { self.canvas.blit_glyph(call) }?)
    }

    fn keep(&mut self, work: W, value: u32) {
        unsafe { game_write(rva(work.address()), value) };
    }
}

impl GameFont for rc2k_core::best_scores::Font {
    fn address(self) -> usize {
        use rc2k_core::best_scores::Font as F;
        match self {
            F::Name => 0x51E584,
            F::Rank => 0x51E5A4,
            F::Time => 0x51E564,
            F::MarkedName => 0x51E524,
            F::MarkedRank => 0x51E544,
            F::MarkedTime => 0x51E504,
            F::Heading => 0x51E9A4,
            F::Fastest => 0x51E6E4,
        }
    }
}

impl GameWork for rc2k_core::best_scores::Work {
    fn address(self) -> usize {
        use rc2k_core::best_scores::Work as W;
        match self {
            W::Rank => 0x7559AC,
            W::RowY => 0x7559B0,
            W::CarGlyph => 0x7559B4,
            W::Heading => 0x7559EC,
        }
    }
}

impl GameFont for rc2k_core::stage_panel::Font {
    fn address(self) -> usize {
        use rc2k_core::stage_panel::Font as F;
        match self {
            F::Label => 0x51E5A4,
            F::Value => 0x51E584,
            F::Heading => 0x51E5C4,
            F::RallyHeading => 0x51E564,
            F::Description => 0x51E3E4,
            F::Number => 0x51E3C4,
            F::Title => 0x51E9A4,
            F::Time => 0x51E564,
        }
    }
}

impl GameWork for rc2k_core::stage_panel::Work {
    fn address(self) -> usize {
        use rc2k_core::stage_panel::Work as W;
        match self {
            W::Scratch => 0x7559EC,
            W::Finish => 0x7559F0,
            W::Entry34 => 0x75ACD4,
        }
    }
}
