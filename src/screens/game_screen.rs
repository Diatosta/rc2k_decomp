//! The game's side of `rc2k_core::screens::screen::Screen` (2026-10-20, the screens' conversion): a
//! family's fonts are font records in game memory, its texts game addresses, its work
//! variables game dwords; drawn through the text functions and the canvas.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::gfx::blit_glyph::BlitCall;
use crate::screens::{font_at, line_buffer};
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::font::Font;
use rc2k_core::screens::screen::{Buffer, Drawn, Screen};
use std::marker::PhantomData;

/// A family's font: the address of its record (as IDA shows it).
pub trait GameFont: Copy {
    fn address(self) -> usize;
    /// The font record in the game.
    fn pointer(self) -> *mut Font {
        font_at(self.address())
    }
}

/// A font the game hands over as a pointer (the menu tables' fonts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontAt(pub u32);

impl GameFont for FontAt {
    fn address(self) -> usize {
        self.0 as usize
    }
    fn pointer(self) -> *mut Font {
        self.0 as *mut Font
    }
}

/// A family's work variable: its address (as IDA shows it).
pub trait GameWork: Copy {
    fn address(self) -> usize;
}

pub struct GameScreen<'a, F, W> {
    pub canvas: &'a mut dyn Canvas,
    /// The menu's first item (the menus' screen: `rc2k_core::screens::menus::MenuScreen`).
    pub menu_items: u32,
    sprites: u32,
    family: PhantomData<(F, W)>,
}

impl<'a, F, W> GameScreen<'a, F, W> {
    pub fn new(canvas: &'a mut dyn Canvas) -> Self {
        Self {
            canvas,
            menu_items: 0,
            sprites: crate::gfx::blit_marker_pair::sprite_table(),
            family: PhantomData,
        }
    }
}

fn font_of(f: impl GameFont) -> *mut Font {
    f.pointer()
}

/// Where a clip edge is kept.
fn clip_rva(edge: rc2k_core::screens::screen::Edge) -> usize {
    use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};
    use rc2k_core::screens::screen::Edge;
    match edge {
        Edge::Left => CLIP_LEFT_RVA,
        Edge::Right => CLIP_RIGHT_RVA,
        Edge::Top => CLIP_TOP_RVA,
        Edge::Bottom => CLIP_BOTTOM_RVA,
    }
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

    fn bar(&mut self, bar: rc2k_core::screens::screen::Bar) -> Result<(), Refused> {
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

    fn blend(&mut self, since_key: u32, a: F, b: F) -> Result<(), Refused> {
        unsafe { crate::text::blend_font::blend_font(since_key, 0x100, font_of(a), font_of(b)) }?;
        Ok(())
    }

    fn clip_edge(&mut self, edge: rc2k_core::screens::screen::Edge) -> Result<u32, Refused> {
        Ok(unsafe { game_read(clip_rva(edge)) })
    }

    fn set_clip_edge(
        &mut self,
        edge: rc2k_core::screens::screen::Edge,
        value: u32,
    ) -> Result<(), Refused> {
        unsafe { game_write(clip_rva(edge), value) };
        Ok(())
    }

    fn item(&mut self, n: u32, on: bool) -> Result<(), Refused> {
        if on {
            crate::menu::enable_item::enable_item(n)
        } else {
            crate::menu::disable_item::disable_item(n)
        }
    }

    fn measure_string(&mut self, font: F, id: u32, x: i32) -> Result<(u32, u32), Refused> {
        let m = unsafe {
            crate::text::measure_string_by_id::measure_string_by_id_at(
                id,
                font_of(font),
                x,
                self.canvas,
            )
        }?;
        Ok((m.width, m.lines))
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

impl GameFont for rc2k_core::screens::best_scores::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::best_scores::Font as F;
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

impl GameWork for rc2k_core::screens::best_scores::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::best_scores::Work as W;
        match self {
            W::Rank => 0x7559AC,
            W::RowY => 0x7559B0,
            W::CarGlyph => 0x7559B4,
            W::Heading => 0x7559EC,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_panel::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::stage_panel::Font as F;
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

impl GameWork for rc2k_core::screens::stage_panel::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::stage_panel::Work as W;
        match self {
            W::Scratch => 0x7559EC,
            W::Finish => 0x7559F0,
            W::Entry34 => 0x75ACD4,
        }
    }
}

impl GameFont for rc2k_core::screens::itinerary::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::itinerary::Font as F;
        match self {
            F::Heading => 0x51E9A4,
            F::Plain => 0x51E584,
            F::Marked => 0x51E524,
            F::Clock => 0x51E564,
            F::MarkedClock => 0x51E504,
        }
    }
}

impl GameWork for rc2k_core::screens::itinerary::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::itinerary::Work as W;
        match self {
            W::Y => 0x7559AC,
            W::Row => 0x7559B0,
            W::Stage => 0x7559B4,
            W::Service => 0x755A0C,
            W::ServiceString => 0x755A08,
        }
    }
}

impl GameFont for rc2k_core::screens::settings_pages::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::settings_pages::Font as F;
        match self {
            F::Value => 0x51E8C4,
            F::Small => 0x51E3C4,
            F::Title => 0x51E904,
            F::Head => 0x51E944,
            F::Line => 0x51E8E4,
        }
    }
}

impl GameWork for rc2k_core::screens::settings_pages::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::settings_pages::Work as W;
        match self {
            W::Scratch => 0x7559EC,
            W::Seconds => 0x7559AC,
            W::Glyph => 0x7559F0,
            W::CheckX => 0x755A18,
        }
    }
}

/// The settings pages' screen.
pub type SettingsScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::settings_pages::Font,
    rc2k_core::screens::settings_pages::Work,
>;

/// What a name drawn at x 0x1D1 leaves in the registers (carry, eax 0, ebx, ecx, edx).
pub fn left_of(out: rc2k_core::screens::screen::Drawn) -> crate::screens::harness::Left {
    use crate::framework::{EAX, EBX, ECX, EDX};
    crate::screens::harness::Left::carry(out.too_wide)
        .with(EAX, 0)
        .with(EBX, out.after)
        .with(ECX, 0x1D1)
        .with(EDX, out.y as u32)
}

impl GameFont for rc2k_core::screens::car_pages::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::car_pages::Font::Labels => 0x51E5A4,
            rc2k_core::screens::car_pages::Font::Figures => 0x51E584,
        }
    }
}

impl GameWork for rc2k_core::screens::car_pages::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::car_pages::Work::TextKept => 0x7559EC,
        }
    }
}

impl GameFont for rc2k_core::screens::slot_lists::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::slot_lists::Font::Names => 0x51E8C4,
            rc2k_core::screens::slot_lists::Font::Setups => 0x51E8A4,
        }
    }
}

impl GameWork for rc2k_core::screens::slot_lists::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::slot_lists::Work::X => 0x755A18,
            rc2k_core::screens::slot_lists::Work::Y => 0x755A1C,
            rc2k_core::screens::slot_lists::Work::Row => 0x7559EC,
        }
    }
}

/// The load and save pages' lists' screen.
pub type SlotScreen<'a> =
    GameScreen<'a, rc2k_core::screens::slot_lists::Font, rc2k_core::screens::slot_lists::Work>;

impl GameFont for rc2k_core::screens::savegame_preview::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::savegame_preview::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Title => 0x51E5E4,
            F::RowNumber => 0x51E5A4,
        }
    }
}

impl GameWork for rc2k_core::screens::savegame_preview::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// The load-game preview's screen.
pub type PreviewScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::savegame_preview::Font,
    rc2k_core::screens::savegame_preview::Work,
>;

impl GameFont for rc2k_core::screens::players_results::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::players_results::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Small => 0x51E564,
            F::Stage => 0x51E9A4,
        }
    }
}

impl GameWork for rc2k_core::screens::players_results::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::players_results::Work::Gap => 0x7559B8,
            rc2k_core::screens::players_results::Work::FirstTime => 0x7559BC,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_results::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::stage_results::Font as F;
        match self {
            F::Rows => 0x51E584,
            F::Small => 0x51E564,
            F::Rank => 0x51E5A4,
            F::Fastest => 0x51E6E4,
        }
    }
}

impl GameWork for rc2k_core::screens::stage_results::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::stage_results::Work as W;
        match self {
            W::Rank => 0x7559AC,
            W::Scratch => 0x7559EC,
            W::FirstTime => 0x75AB24,
            W::Gap => 0x7559F0,
            W::RowY => 0x7559B0,
        }
    }
}

impl GameFont for rc2k_core::screens::results_pages::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::results_pages::Font as F;
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

impl GameWork for rc2k_core::screens::results_pages::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::results_pages::Work as W;
        match self {
            W::FirstTime => 0x7559BC,
            W::OverallFirst => 0x7559D4,
            W::Gap => 0x7559C0,
            W::Row => 0x755579,
            W::RowEnd => 0x7556BC,
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::RankingWork {
    fn address(self) -> usize {
        use rc2k_core::screens::results_pages::RankingWork as W;
        match self {
            W::FirstTime => 0x7559BC,
            W::Place => 0x7559AC,
            W::Gap => 0x7559C0,
            W::SpriteY => 0x7559B0,
        }
    }
}

impl GameFont for rc2k_core::screens::player_result::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::player_result::Font as F;
        match self {
            F::Labels => 0x51E6E4,
            F::Values => 0x51E6C4,
            F::Measure => 0x51E624,
            F::Name => 0x51E584,
        }
    }
}

impl GameWork for rc2k_core::screens::player_result::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// The player's result pages' screen.
pub type PlayerResultScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::player_result::Font,
    rc2k_core::screens::player_result::Work,
>;

impl GameFont for rc2k_core::screens::duel_result::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::duel_result::Font as F;
        match self {
            F::Big => 0x51E584,
            F::Small => 0x51E5A4,
            F::Stage => 0x51E9A4,
            F::Time => 0x51E564,
        }
    }
}

impl GameWork for rc2k_core::screens::duel_result::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::duel_result::Work::First => 0x758720,
            rc2k_core::screens::duel_result::Work::Second => 0x758724,
        }
    }
}

impl GameFont for rc2k_core::screens::notes::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::notes::Font::Question => 0x51E8A4,
            rc2k_core::screens::notes::Font::Value => 0x51E8C4,
        }
    }
}

impl GameWork for rc2k_core::screens::notes::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// A one-string screen drawn: what it leaves in the registers.
pub fn draw_note(
    canvas: &mut dyn Canvas,
    note: &rc2k_core::screens::notes::Note,
) -> Result<crate::screens::harness::Left, Refused> {
    let out = rc2k_core::screens::notes::note(
        &mut GameScreen::<rc2k_core::screens::notes::Font, rc2k_core::screens::notes::Work>::new(
            canvas,
        ),
        note,
    )?;
    Ok(crate::screens::harness::Left::of_drawn(out, note.x))
}

impl GameFont for rc2k_core::screens::player_details::Font {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::player_details::Font::Value => 0x51E8C4,
            rc2k_core::screens::player_details::Font::Measure => 0x51E824,
        }
    }
}

impl GameWork for rc2k_core::screens::player_details::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::player_details::Work::Seconds => 0x7559AC,
            rc2k_core::screens::player_details::Work::Kept => 0x7559EC,
        }
    }
}

/// The players' details pages' screen.
pub type DetailsScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::player_details::Font,
    rc2k_core::screens::player_details::Work,
>;

impl GameWork for rc2k_core::screens::menus::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::menus::Work;
        match self {
            Work::SpacedFont => 0x7559EC,
            Work::MarkerX => 0x755A18,
            Work::MarkerY => 0x755A1C,
            Work::Highlighted => 0x7559D0,
        }
    }
}

impl rc2k_core::screens::menus::MenuScreen for MenuScreen<'_> {
    fn own_button(&mut self, n: usize) -> Result<(), Refused> {
        unsafe { crate::screens::buttons::draw_item_button(self.item_address(n), self.canvas) }
    }
    fn text_button(
        &mut self,
        button: rc2k_core::screens::menus::TextButton,
        n: usize,
        x: u32,
    ) -> Result<(), Refused> {
        use crate::screens::buttons::{ALIGN_CENTRE, ALIGN_RIGHT, ALT, HIGHLIGHTED, PLAIN};
        use rc2k_core::screens::menus::TextButton;
        let (kind, align) = match button {
            TextButton::Right => (&ALT, ALIGN_RIGHT),
            TextButton::Centred => (&PLAIN, ALIGN_CENTRE),
            TextButton::HighlightedRight => (&HIGHLIGHTED, ALIGN_RIGHT),
            TextButton::HighlightedCentred => (&HIGHLIGHTED, ALIGN_CENTRE),
        };
        unsafe {
            crate::screens::buttons::draw_text_button(
                kind,
                align,
                self.item_address(n),
                x as i32,
                self.canvas,
            )
        }
    }
    fn blended(&mut self, since_key: u32, a: FontAt, b: FontAt) -> Result<FontAt, Refused> {
        use crate::text::blend_font::{BLENDED_FONT_RVA, blend_font};
        unsafe { blend_font(since_key, 0x100, a.pointer(), b.pointer()) }?;
        Ok(FontAt(crate::framework::game_addr(BLENDED_FONT_RVA) as u32))
    }
    fn font_value(&self, f: FontAt) -> u32 {
        f.0
    }
}

impl MenuScreen<'_> {
    /// Item `n`'s address.
    fn item_address(&self, n: usize) -> u32 {
        self.menu_items
            .wrapping_add(n as u32 * crate::menu::item_at::ITEM_LEN)
    }
}

/// The menus' screen: fonts by pointer.
pub type MenuScreen<'a> = GameScreen<'a, FontAt, rc2k_core::screens::menus::Work>;

impl GameFont for rc2k_core::screens::service::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::service::Font;
        match self {
            Font::Times => 0x51E3E4,
            Font::Available => 0x51E584,
            Font::Remaining => 0x51E5A4,
            Font::Overrun => 0x51E524,
            Font::Penalty => 0x51E544,
        }
    }
}

impl GameWork for rc2k_core::screens::service::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::service::Work;
        match self {
            Work::Penalty => 0x75ACC0,
            Work::A => 0x7559AC,
            Work::B => 0x7559EC,
        }
    }
}

/// The service area's readouts' screen.
pub type ServiceScreen<'a> =
    GameScreen<'a, rc2k_core::screens::service::Font, rc2k_core::screens::service::Work>;

impl GameFont for rc2k_core::screens::highlighted_caption::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::highlighted_caption::Font;
        match self {
            Font::Measure => 0x51E824,
            Font::A => 0x51E964,
            Font::B => 0x51E844,
            Font::Blended => 0x51E384,
        }
    }
}

impl GameWork for rc2k_core::screens::highlighted_caption::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::highlighted_caption::Work;
        match self {
            Work::MarkerX => 0x755A18,
            Work::MarkerY => 0x755A1C,
            Work::Width => 0x7559AC,
            Work::X => 0x7559B0,
            Work::Y => 0x7559B4,
            Work::Left => 0x7559C0,
            Work::Right => 0x7559C4,
            Work::LeftX => 0x755A20,
            Work::LeftY => 0x755A24,
            Work::RightX => 0x755A28,
            Work::RightY => 0x755A2C,
        }
    }
}

impl GameFont for rc2k_core::screens::inspection::Font {
    fn address(self) -> usize {
        0x51E3C4
    }
}

impl GameWork for rc2k_core::screens::inspection::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::inspection::Work::Lost => 0x7559EC,
            rc2k_core::screens::inspection::Work::Status => 0x75AC80,
        }
    }
}

impl GameFont for rc2k_core::screens::controls::Font {
    fn address(self) -> usize {
        0x51E8C4
    }
}

impl GameWork for rc2k_core::screens::controls::Work {
    fn address(self) -> usize {
        match self {}
    }
}

/// The controller pages' lists' screen.
pub type ControlsScreen<'a> =
    GameScreen<'a, rc2k_core::screens::controls::Font, rc2k_core::screens::controls::Work>;

impl GameFont for rc2k_core::screens::championship_table::Font {
    fn address(self) -> usize {
        use rc2k_core::screens::championship_table::Font;
        match self {
            Font::Position => 0x51E5A4,
            Font::Text => 0x51E584,
            Font::MarkedPosition => 0x51E544,
            Font::MarkedText => 0x51E524,
        }
    }
}

impl GameWork for rc2k_core::screens::championship_table::Work {
    fn address(self) -> usize {
        match self {
            rc2k_core::screens::championship_table::Work::Position => 0x755579,
            rc2k_core::screens::championship_table::Work::End => 0x7556BC,
        }
    }
}

impl GameFont for rc2k_core::screens::controller_page::Font {
    fn address(self) -> usize {
        0x51E8C4
    }
}

impl GameWork for rc2k_core::screens::controller_page::Work {
    fn address(self) -> usize {
        use rc2k_core::screens::controller_page::Work;
        match self {
            Work::Seconds => 0x7559AC,
            Work::Record => 0x755A0C,
            Work::FirstLeft => 0x7559DC,
            Work::FirstRight => 0x7559E0,
            Work::SecondLeft => 0x7559E4,
            Work::SecondRight => 0x7559E8,
            Work::Place => 0x7559EC,
            Work::Buttons => 0x7559F0,
        }
    }
}
