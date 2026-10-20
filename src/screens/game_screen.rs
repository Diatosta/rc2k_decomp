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
        Buffer::Cells => crate::framework::game_ptr(rva(0xA8BC54)),
    }
}

/// `bytes` (with their NUL) into a shared buffer; refused past its end (the line buffer
/// 0x100 bytes: `byte_A8BC54` follows).
pub unsafe fn store_in(buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
    let room = match buffer {
        Buffer::Line => 0x100,
        Buffer::Text => crate::strings::text_from_ids::TEXT_LEN,
        Buffer::Cells => 0x100,
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

    fn draw_clock(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_clock_fixed_pitch::draw_clock_fixed_pitch_at;
        unsafe {
            draw_clock_fixed_pitch_at(font_of(font), buffer_at(buffer), right, y, self.canvas)
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

    fn bar(&mut self, bar: rc2k_core::screen::Bar) -> Result<(), Refused> {
        use crate::gfx::blit_bar::{Bar, blit_bar};
        Bar {
            x: bar.x,
            y: bar.y,
            width: bar.width,
            fill_glyph: bar.fill_glyph,
            end_glyph: bar.end_glyph,
            percent: bar.percent,
        }
        .write();
        let canvas = &mut *self.canvas;
        Ok(unsafe { blit_bar(&mut |call| canvas.blit_glyph(call)) }?)
    }

    fn buffer_address(&mut self, buffer: Buffer) -> u32 {
        buffer_at(buffer) as u32
    }

    fn measure(&mut self, font: F, text: u32, x: i32) -> Result<(u32, u32), Refused> {
        let m = unsafe {
            crate::text::measure_text::measure_text_at(
                font_of(font),
                text as *const u8,
                x,
                self.canvas,
            )
        }?;
        Ok((m.width, m.lines))
    }

    fn spaced(&mut self, font: F, bytes: &[u8], x: i32, y: i32) -> Result<(), Refused> {
        let mut text = bytes.to_vec();
        text.push(0);
        unsafe {
            store_in(Buffer::Line, &text)?;
            let at = line_buffer();
            crate::screens::draw_spaced_number::draw_spaced_number(
                font_of(font),
                at,
                at.add(bytes.len()),
                x,
                y,
                self.canvas,
            )
        }
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        crate::errors::FatalError { code, message }.into()
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

impl GameFont for rc2k_core::itinerary::Font {
    fn address(self) -> usize {
        use rc2k_core::itinerary::Font as F;
        match self {
            F::Heading => 0x51E9A4,
            F::Plain => 0x51E584,
            F::Marked => 0x51E524,
            F::Clock => 0x51E564,
            F::MarkedClock => 0x51E504,
        }
    }
}

impl GameWork for rc2k_core::itinerary::Work {
    fn address(self) -> usize {
        use rc2k_core::itinerary::Work as W;
        match self {
            W::Y => 0x7559AC,
            W::Row => 0x7559B0,
            W::Stage => 0x7559B4,
            W::Service => 0x755A0C,
            W::ServiceString => 0x755A08,
        }
    }
}

impl GameFont for rc2k_core::settings_pages::Font {
    fn address(self) -> usize {
        use rc2k_core::settings_pages::Font as F;
        match self {
            F::Value => 0x51E8C4,
            F::Small => 0x51E3C4,
            F::Title => 0x51E904,
            F::Head => 0x51E944,
            F::Line => 0x51E8E4,
        }
    }
}

impl GameWork for rc2k_core::settings_pages::Work {
    fn address(self) -> usize {
        use rc2k_core::settings_pages::Work as W;
        match self {
            W::Scratch => 0x7559EC,
            W::Seconds => 0x7559AC,
            W::Glyph => 0x7559F0,
            W::CheckX => 0x755A18,
        }
    }
}

/// The settings pages' screen.
pub type SettingsScreen<'a> =
    GameScreen<'a, rc2k_core::settings_pages::Font, rc2k_core::settings_pages::Work>;

/// What a name drawn at x 0x1D1 leaves in the registers (carry, eax 0, ebx, ecx, edx).
pub fn left_of(out: rc2k_core::screen::Drawn) -> crate::screens::harness::Left {
    use crate::framework::{EAX, EBX, ECX, EDX};
    crate::screens::harness::Left::carry(out.too_wide)
        .with(EAX, 0)
        .with(EBX, out.after)
        .with(ECX, 0x1D1)
        .with(EDX, out.y as u32)
}

impl GameFont for rc2k_core::car_pages::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::car_pages::Font::Labels => 0x51E5A4,
            rc2k_core::car_pages::Font::Figures => 0x51E584,
        }
    }
}

impl GameWork for rc2k_core::car_pages::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::car_pages::Work::TextKept => 0x7559EC,
        }
    }
}

impl GameFont for rc2k_core::slot_lists::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::slot_lists::Font::Names => 0x51E8C4,
            rc2k_core::slot_lists::Font::Setups => 0x51E8A4,
        }
    }
}

impl GameWork for rc2k_core::slot_lists::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::slot_lists::Work::X => 0x755A18,
            rc2k_core::slot_lists::Work::Y => 0x755A1C,
            rc2k_core::slot_lists::Work::Row => 0x7559EC,
        }
    }
}

/// The load and save pages' lists' screen.
pub type SlotScreen<'a> = GameScreen<'a, rc2k_core::slot_lists::Font, rc2k_core::slot_lists::Work>;

impl GameFont for rc2k_core::savegame_preview::Font {
    fn address(self) -> usize {
        use rc2k_core::savegame_preview::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Title => 0x51E5E4,
            F::RowNumber => 0x51E5A4,
        }
    }
}

impl GameWork for rc2k_core::savegame_preview::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// The load-game preview's screen.
pub type PreviewScreen<'a> =
    GameScreen<'a, rc2k_core::savegame_preview::Font, rc2k_core::savegame_preview::Work>;

impl GameFont for rc2k_core::players_results::Font {
    fn address(self) -> usize {
        use rc2k_core::players_results::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Small => 0x51E564,
            F::Stage => 0x51E9A4,
        }
    }
}

impl GameWork for rc2k_core::players_results::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::players_results::Work::Gap => 0x7559B8,
            rc2k_core::players_results::Work::FirstTime => 0x7559BC,
        }
    }
}

impl GameFont for rc2k_core::stage_results::Font {
    fn address(self) -> usize {
        use rc2k_core::stage_results::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Small => 0x51E564,
            F::Rank => 0x51E5A4,
            F::Fastest => 0x51E6E4,
        }
    }
}

impl GameWork for rc2k_core::stage_results::Work {
    fn address(self) -> usize {
        use rc2k_core::stage_results::Work as W;
        match self {
            W::Rank => 0x7559AC,
            W::Scratch => 0x7559EC,
            W::FirstTime => 0x75AB24,
            W::Gap => 0x7559F0,
            W::RowY => 0x7559B0,
        }
    }
}

impl GameFont for rc2k_core::results_pages::Font {
    fn address(self) -> usize {
        use rc2k_core::results_pages::Font as F;
        match self {
            F::Title => 0x51E944,
            F::StageName => 0x51E9A4,
            F::Place => 0x51E5A4,
            F::Name => 0x51E584,
            F::Time => 0x51E564,
            F::PlayerPlace => 0x51E544,
            F::PlayerName => 0x51E524,
            F::PlayerTime => 0x51E504,
        }
    }
}

impl GameWork for rc2k_core::results_pages::Work {
    fn address(self) -> usize {
        use rc2k_core::results_pages::Work as W;
        match self {
            W::FirstTime => 0x7559BC,
            W::OverallFirst => 0x7559D4,
            W::Gap => 0x7559C0,
            W::Row => 0x755579,
            W::RowEnd => 0x7556BC,
        }
    }
}

impl GameWork for rc2k_core::results_pages::RankingWork {
    fn address(self) -> usize {
        use rc2k_core::results_pages::RankingWork as W;
        match self {
            W::FirstTime => 0x7559BC,
            W::Place => 0x7559AC,
            W::Gap => 0x7559C0,
            W::SpriteY => 0x7559B0,
        }
    }
}

impl GameFont for rc2k_core::player_result::Font {
    fn address(self) -> usize {
        use rc2k_core::player_result::Font as F;
        match self {
            F::Labels => 0x51E6E4,
            F::Values => 0x51E6C4,
            F::Measure => 0x51E624,
            F::Name => 0x51E584,
        }
    }
}

impl GameWork for rc2k_core::player_result::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// The player's result pages' screen.
pub type PlayerResultScreen<'a> =
    GameScreen<'a, rc2k_core::player_result::Font, rc2k_core::player_result::Work>;

impl GameFont for rc2k_core::duel_result::Font {
    fn address(self) -> usize {
        use rc2k_core::duel_result::Font as F;
        match self {
            F::Big => 0x51E584,
            F::Small => 0x51E5A4,
            F::Stage => 0x51E9A4,
            F::Time => 0x51E564,
        }
    }
}

impl GameWork for rc2k_core::duel_result::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::duel_result::Work::First => 0x758720,
            rc2k_core::duel_result::Work::Second => 0x758724,
        }
    }
}
