//! The game's side of `rc2k_core::screens::screen::Screen` (2026-10-20, the screens' conversion): a
//! family's fonts are font records in game memory, its texts game addresses, its work
//! variables game dwords; drawn through the text functions and the canvas.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_glyph::BlitCall;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::results_store::RESULT_FIGURES;
use crate::screens::line_buffer;
use crate::session::Dword;
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::font::Font;
use rc2k_core::screens::screen::{Buffer, Drawn, Screen};
use std::marker::PhantomData;

/// A family's font: the address of its record (as IDA shows it).
pub trait GameFont: Copy {
    /// The font record in the game.
    fn pointer(self) -> *mut Font;
}

/// A font the game hands over as a pointer (the menu tables' fonts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontAt(pub u32);

impl GameFont for FontAt {
    fn pointer(self) -> *mut Font {
        self.0 as *mut Font
    }
}

/// A family's work variable.
pub trait GameWork: Copy {
    fn cell(self) -> &'static Dword;
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
fn clip_edge_cell(edge: rc2k_core::screens::screen::Edge) -> &'static crate::session::Dword {
    use crate::gfx::blit::BLIT;
    use rc2k_core::screens::screen::Edge;
    match edge {
        Edge::Left => &BLIT.clip.left,
        Edge::Right => &BLIT.clip.right,
        Edge::Top => &BLIT.clip.top,
        Edge::Bottom => &BLIT.clip.bottom,
    }
}

/// Where a shared buffer is.
pub fn buffer_at(buffer: Buffer) -> *mut u8 {
    match buffer {
        Buffer::Line => line_buffer(),
        Buffer::Text => crate::strings::text_from_ids::text(),
        Buffer::Cells => crate::screens::text_buffers_store::TEXT_BUFFERS
            .cells
            .as_ptr(),
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
        Ok(clip_edge_cell(edge).get())
    }

    fn set_clip_edge(
        &mut self,
        edge: rc2k_core::screens::screen::Edge,
        value: u32,
    ) -> Result<(), Refused> {
        clip_edge_cell(edge).set(value);
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
        work.cell().set(value);
    }
}

impl GameFont for rc2k_core::screens::best_scores::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::best_scores::Font as F;
        match self {
            F::Name => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::MarkedName => crate::gfx::typefaces::fonts::MARKED_NAME.as_ptr(),
            F::MarkedRank => crate::gfx::typefaces::fonts::MARKED_RANK.as_ptr(),
            F::MarkedTime => crate::gfx::typefaces::fonts::MARKED_TIME.as_ptr(),
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::best_scores::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::best_scores::Work as W;
        match self {
            W::Rank => &DRAW_WORK.seconds,
            W::RowY => &DRAW_WORK.work_b0,
            W::CarGlyph => &DRAW_WORK.work_b4,
            W::Heading => &DRAW_WORK.figure,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_panel::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::stage_panel::Font as F;
        match self {
            F::Label => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            F::Value => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Heading => crate::gfx::typefaces::fonts::HEADING.as_ptr(),
            F::RallyHeading => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::Description => crate::gfx::typefaces::fonts::LABEL.as_ptr(),
            F::Number => crate::gfx::typefaces::fonts::SMALL.as_ptr(),
            F::Title => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_panel::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::stage_panel::Work as W;
        match self {
            W::Scratch => &DRAW_WORK.figure,
            W::Finish => &DRAW_WORK.hhmm,
            W::Entry34 => &crate::race::state_store::RACE_STATE.settings.stage,
        }
    }
}

impl GameFont for rc2k_core::screens::itinerary::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::itinerary::Font as F;
        match self {
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
            F::Plain => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Marked => crate::gfx::typefaces::fonts::MARKED_NAME.as_ptr(),
            F::Clock => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::MarkedClock => crate::gfx::typefaces::fonts::MARKED_TIME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::itinerary::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::itinerary::Work as W;
        match self {
            W::Y => &DRAW_WORK.seconds,
            W::Row => &DRAW_WORK.work_b0,
            W::Stage => &DRAW_WORK.work_b4,
            W::Service => &DRAW_WORK.work_0c,
            W::ServiceString => &DRAW_WORK.work_08,
        }
    }
}

impl GameFont for rc2k_core::screens::settings_pages::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::settings_pages::Font as F;
        match self {
            F::Value => crate::gfx::typefaces::fonts::LIST.as_ptr(),
            F::Small => crate::gfx::typefaces::fonts::SMALL.as_ptr(),
            F::Title => crate::gfx::typefaces::fonts::BOX_TEXT.as_ptr(),
            F::Head => crate::gfx::typefaces::fonts::HEAD.as_ptr(),
            F::Line => crate::gfx::typefaces::fonts::ITEM_PLAIN.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::settings_pages::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::settings_pages::Work as W;
        match self {
            W::Scratch => &DRAW_WORK.figure,
            W::Seconds => &DRAW_WORK.seconds,
            W::Glyph => &DRAW_WORK.hhmm,
            W::CheckX => &DRAW_WORK.marker_x,
        }
    }
}

/// The settings pages' screen.
pub type SettingsScreen<'a> = GameScreen<
    'a,
    rc2k_core::screens::settings_pages::Font,
    rc2k_core::screens::settings_pages::Work,
>;

impl GameFont for rc2k_core::screens::car_pages::Font {
    fn pointer(self) -> *mut Font {
        match self {
            rc2k_core::screens::car_pages::Font::Labels => {
                crate::gfx::typefaces::fonts::RANK.as_ptr()
            }
            rc2k_core::screens::car_pages::Font::Figures => {
                crate::gfx::typefaces::fonts::NAME.as_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::car_pages::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::car_pages::Work::TextKept => &DRAW_WORK.figure,
        }
    }
}

impl GameFont for rc2k_core::screens::slot_lists::Font {
    fn pointer(self) -> *mut Font {
        match self {
            rc2k_core::screens::slot_lists::Font::Names => {
                crate::gfx::typefaces::fonts::LIST.as_ptr()
            }
            rc2k_core::screens::slot_lists::Font::Setups => {
                crate::gfx::typefaces::fonts::TITLE.as_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::slot_lists::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::slot_lists::Work::X => &DRAW_WORK.marker_x,
            rc2k_core::screens::slot_lists::Work::Y => &DRAW_WORK.marker_y,
            rc2k_core::screens::slot_lists::Work::Row => &DRAW_WORK.figure,
        }
    }
}

/// The load and save pages' lists' screen.
pub type SlotScreen<'a> =
    GameScreen<'a, rc2k_core::screens::slot_lists::Font, rc2k_core::screens::slot_lists::Work>;

impl GameFont for rc2k_core::screens::savegame_preview::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::savegame_preview::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Title => crate::gfx::typefaces::fonts::RANKING_TITLE.as_ptr(),
            F::RowNumber => crate::gfx::typefaces::fonts::RANK.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::savegame_preview::Work {
    fn cell(self) -> &'static Dword {
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
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::players_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Small => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::players_results::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::players_results::Work::Gap => &DRAW_WORK.work_b8,
            rc2k_core::screens::players_results::Work::FirstTime => &DRAW_WORK.first_player_time,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_results::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::stage_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Small => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_results::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::stage_results::Work as W;
        match self {
            W::Rank => &DRAW_WORK.seconds,
            W::Scratch => &DRAW_WORK.figure,
            W::FirstTime => &crate::menu::state_store::MENU_STATE.menus.first_row_time,
            W::Gap => &DRAW_WORK.hhmm,
            W::RowY => &DRAW_WORK.work_b0,
        }
    }
}

impl GameFont for rc2k_core::screens::results_pages::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::results_pages::Font as F;
        match self {
            F::Title => crate::gfx::typefaces::fonts::HEAD.as_ptr(),
            F::StageName => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
            F::Place => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            F::Name => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.as_ptr(),
            F::PlayerPlace => crate::gfx::typefaces::fonts::MARKED_RANK.as_ptr(),
            F::PlayerName => crate::gfx::typefaces::fonts::MARKED_NAME.as_ptr(),
            F::PlayerTime => crate::gfx::typefaces::fonts::MARKED_TIME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::results_pages::Work as W;
        match self {
            W::FirstTime => &DRAW_WORK.first_player_time,
            W::OverallFirst => &DRAW_WORK.overall_first_time,
            W::Gap => &DRAW_WORK.glyph,
            W::Row => &LOOP_FLAGS.champ_row,
            W::RowEnd => &CHAMP_VIEW.row_end,
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::RankingWork {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::results_pages::RankingWork as W;
        match self {
            W::FirstTime => &DRAW_WORK.first_player_time,
            W::Place => &DRAW_WORK.seconds,
            W::Gap => &DRAW_WORK.glyph,
            W::SpriteY => &DRAW_WORK.work_b0,
        }
    }
}

impl GameFont for rc2k_core::screens::player_result::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::player_result::Font as F;
        match self {
            F::Labels => crate::gfx::typefaces::fonts::LABELS.as_ptr(),
            F::Values => crate::gfx::typefaces::fonts::VALUES.as_ptr(),
            F::Measure => crate::gfx::typefaces::fonts::MEASURE_23.as_ptr(),
            F::Name => crate::gfx::typefaces::fonts::NAME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::player_result::Work {
    fn cell(self) -> &'static Dword {
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
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::duel_result::Font as F;
        match self {
            F::Big => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            F::Small => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.as_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::duel_result::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::duel_result::Work::First => &RESULT_FIGURES.order[0],
            rc2k_core::screens::duel_result::Work::Second => &RESULT_FIGURES.order[1],
        }
    }
}

impl GameFont for rc2k_core::screens::notes::Font {
    fn pointer(self) -> *mut Font {
        match self {
            rc2k_core::screens::notes::Font::Question => {
                crate::gfx::typefaces::fonts::TITLE.as_ptr()
            }
            rc2k_core::screens::notes::Font::Value => crate::gfx::typefaces::fonts::LIST.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::notes::Work {
    fn cell(self) -> &'static Dword {
        match self {}
    }
}

/// A one-string screen drawn: what it leaves in the registers.
pub fn draw_note(
    canvas: &mut dyn Canvas,
    note: &rc2k_core::screens::notes::Note,
) -> Result<rc2k_core::screens::screen::Drawn, Refused> {
    rc2k_core::screens::notes::note(
        &mut GameScreen::<rc2k_core::screens::notes::Font, rc2k_core::screens::notes::Work>::new(
            canvas,
        ),
        note,
    )
}

impl GameFont for rc2k_core::screens::player_details::Font {
    fn pointer(self) -> *mut Font {
        match self {
            rc2k_core::screens::player_details::Font::Value => {
                crate::gfx::typefaces::fonts::LIST.as_ptr()
            }
            rc2k_core::screens::player_details::Font::Measure => {
                crate::gfx::typefaces::fonts::MEASURE.as_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::player_details::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::player_details::Work::Seconds => &DRAW_WORK.seconds,
            rc2k_core::screens::player_details::Work::Kept => &DRAW_WORK.figure,
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
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::menus::Work;
        match self {
            Work::SpacedFont => &DRAW_WORK.figure,
            Work::MarkerX => &DRAW_WORK.marker_x,
            Work::MarkerY => &DRAW_WORK.marker_y,
            Work::Highlighted => &DRAW_WORK.drawing_highlighted_item,
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
        use crate::text::blend_font::blend_font;
        unsafe { blend_font(since_key, 0x100, a.pointer(), b.pointer()) }?;
        Ok(FontAt(crate::text::blend_font::BLENDED_FONT.addr() as u32))
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
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::service::Font;
        match self {
            Font::Times => crate::gfx::typefaces::fonts::LABEL.as_ptr(),
            Font::Available => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            Font::Remaining => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            Font::Overrun => crate::gfx::typefaces::fonts::MARKED_NAME.as_ptr(),
            Font::Penalty => crate::gfx::typefaces::fonts::MARKED_RANK.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::service::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::service::Work;
        match self {
            Work::Penalty => &crate::menu::state_store::MENU_STATE.driver_stats.money,
            Work::A => &DRAW_WORK.seconds,
            Work::B => &DRAW_WORK.figure,
        }
    }
}

/// The service area's readouts' screen.
pub type ServiceScreen<'a> =
    GameScreen<'a, rc2k_core::screens::service::Font, rc2k_core::screens::service::Work>;

impl GameFont for rc2k_core::screens::highlighted_caption::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::highlighted_caption::Font;
        match self {
            Font::Measure => crate::gfx::typefaces::fonts::MEASURE.as_ptr(),
            Font::A => crate::gfx::typefaces::fonts::CAPTION_A.as_ptr(),
            Font::B => crate::gfx::typefaces::fonts::CAPTION_B.as_ptr(),
            // g_blendedFont.
            Font::Blended => crate::text::blend_font::BLENDED_FONT.ptr().cast(),
        }
    }
}

impl GameWork for rc2k_core::screens::highlighted_caption::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::highlighted_caption::Work;
        match self {
            Work::MarkerX => &DRAW_WORK.marker_x,
            Work::MarkerY => &DRAW_WORK.marker_y,
            Work::Width => &DRAW_WORK.seconds,
            Work::X => &DRAW_WORK.work_b0,
            Work::Y => &DRAW_WORK.work_b4,
            Work::Left => &DRAW_WORK.glyph,
            Work::Right => &DRAW_WORK.work_c4,
            Work::LeftX => &DRAW_WORK.marker_end_a,
            Work::LeftY => &DRAW_WORK.marker_end_b,
            Work::RightX => &DRAW_WORK.marker_end_c,
            Work::RightY => &DRAW_WORK.marker_end_d,
        }
    }
}

impl GameFont for rc2k_core::screens::inspection::Font {
    fn pointer(self) -> *mut Font {
        crate::gfx::typefaces::fonts::SMALL.as_ptr()
    }
}

impl GameWork for rc2k_core::screens::inspection::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::inspection::Work::Lost => &DRAW_WORK.figure,
            rc2k_core::screens::inspection::Work::Status => {
                &crate::menu::state_store::MENU_STATE.driver_stats.score
            }
        }
    }
}

impl GameFont for rc2k_core::screens::controls::Font {
    fn pointer(self) -> *mut Font {
        crate::gfx::typefaces::fonts::LIST.as_ptr()
    }
}

impl GameWork for rc2k_core::screens::controls::Work {
    fn cell(self) -> &'static Dword {
        match self {}
    }
}

/// The controller pages' lists' screen.
pub type ControlsScreen<'a> =
    GameScreen<'a, rc2k_core::screens::controls::Font, rc2k_core::screens::controls::Work>;

impl GameFont for rc2k_core::screens::championship_table::Font {
    fn pointer(self) -> *mut Font {
        use rc2k_core::screens::championship_table::Font;
        match self {
            Font::Position => crate::gfx::typefaces::fonts::RANK.as_ptr(),
            Font::Text => crate::gfx::typefaces::fonts::NAME.as_ptr(),
            Font::MarkedPosition => crate::gfx::typefaces::fonts::MARKED_RANK.as_ptr(),
            Font::MarkedText => crate::gfx::typefaces::fonts::MARKED_NAME.as_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::championship_table::Work {
    fn cell(self) -> &'static Dword {
        match self {
            rc2k_core::screens::championship_table::Work::Position => &LOOP_FLAGS.champ_row,
            rc2k_core::screens::championship_table::Work::End => &CHAMP_VIEW.row_end,
        }
    }
}

impl GameFont for rc2k_core::screens::controller_page::Font {
    fn pointer(self) -> *mut Font {
        crate::gfx::typefaces::fonts::LIST.as_ptr()
    }
}

impl GameWork for rc2k_core::screens::controller_page::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::controller_page::Work;
        match self {
            Work::Seconds => &DRAW_WORK.seconds,
            Work::Record => &DRAW_WORK.work_0c,
            Work::FirstLeft => &DRAW_WORK.slider_a_start,
            Work::FirstRight => &DRAW_WORK.slider_a_end,
            Work::SecondLeft => &DRAW_WORK.slider_b_start,
            Work::SecondRight => &DRAW_WORK.slider_b_end,
            Work::Place => &DRAW_WORK.figure,
            Work::Buttons => &DRAW_WORK.hhmm,
        }
    }
}
