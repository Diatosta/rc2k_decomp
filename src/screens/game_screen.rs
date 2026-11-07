//! The game's side of `rc2k_core::screens::screen::Screen` (2026-10-20, the screens' conversion): a
//! family's fonts are font records in game memory, its texts places in typed stores or
//! held names, its work variables game dwords; drawn through the text functions and the canvas.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_glyph::BlitCall;
use crate::pointers::GamePtr;
use crate::race::owner::race;
use crate::session::Dword;
use crate::text::draw_text::{Canvas, draw_text_at};

use crate::gfx::owner::gfx;
use crate::menu::owner::menus;
use rc2k_core::screens::screen::{Buffer, Drawn, Screen};
use std::marker::PhantomData;

/// A family's font: the address of its record (as IDA shows it).
pub trait GameFont: Copy {
    /// The font record in the game.
    fn pointer(self) -> GamePtr;
}

/// A font of the menus: one of the font objects a menu item or label names, or the blended
/// font a highlighted item pulses in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuFont {
    Object(crate::gfx::typefaces::FontId),
    Blended,
}

impl GameFont for MenuFont {
    fn pointer(self) -> GamePtr {
        match self {
            MenuFont::Object(id) => id.game_ptr(),
            MenuFont::Blended => GamePtr::to(&crate::text::blend_font::BLENDED_FONT),
        }
    }
}

/// A family's work variable.
pub trait GameWork: Copy {
    fn cell(self) -> &'static Dword;
}

/// A text a screen draws (`HeldText`: a place in a typed store, bounded by it, or a name
/// held by value). The game's tables keep texts as addresses; the screens get them as
/// places (a player's name by `player_name`).
pub trait GameText: Copy {
    /// Drawn in `font` at x, y.
    fn draw(self, font: GamePtr, x: i32, y: i32, canvas: &mut dyn Canvas)
    -> Result<Drawn, Refused>;
    /// A shared buffer of the game's.
    fn of_buffer(at: GamePtr) -> Self;
    /// Measured in `font` from x: its widest line's width.
    fn measure(self, font: GamePtr, x: i32, canvas: &mut dyn Canvas) -> Result<u32, Refused>;
}

/// A text held by value (a typed table's name, copied out of it): at most `ROOM` bytes,
/// without its NUL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    bytes: [u8; Held::ROOM],
    len: usize,
}

impl Held {
    pub const ROOM: usize = 0x40;
    pub const EMPTY: Self = Self {
        bytes: [0; Self::ROOM],
        len: 0,
    };

    /// None for a text longer than `ROOM`.
    pub fn new(text: &[u8]) -> Option<Self> {
        let mut bytes = [0; Self::ROOM];
        bytes.get_mut(..text.len())?.copy_from_slice(text);
        Some(Self {
            bytes,
            len: text.len(),
        })
    }

    /// The text and its NUL.
    fn with_nul(&self) -> Vec<u8> {
        let mut t = self.bytes[..self.len].to_vec();
        t.push(0);
        t
    }
}

/// A typed entry's name (its text up to the NUL, None without one in the entry) held for
/// drawing; refused without its NUL or past the held room.
pub fn held_name(text: Option<Vec<u8>>) -> Result<HeldText, crate::errors::BadInput> {
    text.and_then(|t| Held::new(&t))
        .map(HeldText::Held)
        .ok_or(crate::errors::BadInput("a name past its entry"))
}

/// The name of player `player` (counted from 0), drawn from the names' store.
pub fn player_name_of(player: usize) -> HeldText {
    HeldText::Place(crate::menu::player_names::name_ptr(player))
}

/// The name an entry's name address names (`player_names::name_address`), drawn from the
/// names' store; refused when it names no player's name.
pub fn player_name(name_at: u32) -> Result<HeldText, crate::errors::BadInput> {
    crate::menu::player_names::index_of(name_at)
        .map(|k| HeldText::Place(crate::menu::player_names::name_ptr(k)))
        .ok_or(crate::errors::BadInput(
            "a player's name that is no player's name",
        ))
}

/// The texts of a screen whose names are held by value: those, or the game's own buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldText {
    /// A text in a field of a typed store (a record's name), bounded by it.
    Place(GamePtr),
    Held(Held),
}

impl GameText for HeldText {
    fn draw(
        self,
        font: GamePtr,
        x: i32,
        y: i32,
        canvas: &mut dyn Canvas,
    ) -> Result<Drawn, Refused> {
        match self {
            HeldText::Place(at) => {
                let out = draw_text_at(font, &at.bytes("a name")?, x, y, canvas)?;
                Ok(Drawn {
                    y: out.y,
                    too_wide: out.too_wide,
                })
            }
            HeldText::Held(h) => {
                let out = draw_text_at(font, &h.with_nul()[..], x, y, canvas)?;
                Ok(Drawn {
                    y: out.y,
                    // Where the text after it starts, in the text (no caller of a held
                    // text reads it: the names' draws keep only the y).
                    too_wide: out.too_wide,
                })
            }
        }
    }

    fn of_buffer(at: GamePtr) -> Self {
        HeldText::Place(at)
    }

    fn measure(self, font: GamePtr, x: i32, canvas: &mut dyn Canvas) -> Result<u32, Refused> {
        match self {
            HeldText::Place(at) => {
                let m = crate::text::measure_text::measure_text_at(
                    font,
                    &at.bytes("a name")?,
                    x,
                    canvas,
                )?;
                Ok(m.width)
            }
            HeldText::Held(h) => {
                let m =
                    crate::text::measure_text::measure_text_at(font, &h.with_nul()[..], x, canvas)?;
                Ok(m.width)
            }
        }
    }
}

pub struct GameScreen<'a, F, W, T = HeldText> {
    pub canvas: &'a mut dyn Canvas,
    /// The menu's first item (the menus' screen: `rc2k_core::screens::menus::MenuScreen`).
    pub menu_items: &'static [crate::menu::menus_store::MenuItem],
    family: PhantomData<(F, W, T)>,
}

impl<'a, F, W, T> GameScreen<'a, F, W, T> {
    pub fn new(canvas: &'a mut dyn Canvas) -> Self {
        Self {
            canvas,
            menu_items: &[],
            family: PhantomData,
        }
    }
}

fn font_of(f: impl GameFont) -> GamePtr {
    f.pointer()
}

/// Where a clip edge is kept.
fn clip_edge_cell(edge: rc2k_core::screens::screen::Edge) -> &'static crate::session::Dword {
    let drawing = &gfx().drawing;
    use rc2k_core::screens::screen::Edge;
    match edge {
        Edge::Left => &drawing.blit.clip.left,
        Edge::Right => &drawing.blit.clip.right,
        Edge::Top => &drawing.blit.clip.top,
        Edge::Bottom => &drawing.blit.clip.bottom,
    }
}

/// Where a shared buffer is.
pub fn buffer_at(buffer: Buffer) -> GamePtr {
    use crate::screens::text_buffers_store::TEXT_BUFFERS;
    match buffer {
        Buffer::Line => GamePtr::to(&TEXT_BUFFERS.line),
        Buffer::Text => crate::strings::text_from_ids::text_at(),
        Buffer::Cells => GamePtr::to(&TEXT_BUFFERS.cells),
    }
}

/// `bytes` (with their NUL) into a shared buffer; refused past its end (the line buffer
/// 0x100 bytes: `byte_A8BC54` follows).
pub fn store_in(buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
    let room = match buffer {
        Buffer::Line => 0x100,
        Buffer::Text => crate::strings::text_from_ids::TEXT_LEN,
        Buffer::Cells => 0x100,
    };
    if bytes.len() > room {
        return Err(BadInput("a text past its buffer").into());
    }
    buffer_at(buffer).write_bytes(bytes, "a shared text buffer")?;
    Ok(())
}

impl<F: GameFont, W: GameWork, T: GameText> Screen for GameScreen<'_, F, W, T> {
    type Font = F;
    type Text = T;
    type Work = W;
    type Error = Refused;

    fn text(&mut self, font: F, text: T, x: i32, y: i32) -> Result<Drawn, Refused> {
        text.draw(font_of(font), x, y, self.canvas)
    }

    fn store(&mut self, buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
        store_in(buffer, bytes)
    }

    fn draw(&mut self, font: F, buffer: Buffer, x: i32, y: i32) -> Result<Drawn, Refused> {
        self.text(font, T::of_buffer(buffer_at(buffer)), x, y)
    }

    fn draw_time(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
        draw_time_fixed_pitch_at(font_of(font), buffer_at(buffer), right, y, self.canvas)?;
        Ok(())
    }

    fn draw_clock(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_clock_fixed_pitch::draw_clock_fixed_pitch_at;
        draw_clock_fixed_pitch_at(font_of(font), buffer_at(buffer), right, y, self.canvas)?;
        Ok(())
    }

    fn string(&mut self, font: F, id: u32, x: i32, y: i32) -> Result<Drawn, Refused> {
        use crate::text::draw_string_by_id::draw_string_by_id_at;
        let out = { draw_string_by_id_at(id, font_of(font), x, y, self.canvas) }?;
        Ok(Drawn {
            y: out.y,
            too_wide: out.too_wide,
        })
    }

    fn string_bytes(&mut self, id: u32) -> Vec<u8> {
        let mut out = Vec::new();
        crate::menu::event_flow::refuse_here(
            crate::strings::append_string_by_id::push_string(&mut out, id),
            "screens::game_screen: a string",
        );
        out
    }

    fn strings(&mut self, first: u32, count: u32) -> Result<Vec<u8>, Refused> {
        Ok(crate::strings::text_from_ids::text_from_ids(first, count)?)
    }

    fn sprite(&mut self, glyph: u32, x: i32, y: i32) -> Result<(), Refused> {
        let call = BlitCall::sprite(glyph, x, y);
        self.canvas.blit_glyph(call)
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
        blit_bar(&mut |call| canvas.blit_glyph(call))
    }

    fn blend(&mut self, since_key: u32, a: F, b: F) -> Result<(), Refused> {
        crate::text::blend_font::blend_font(since_key, 0x100, font_of(a), font_of(b))?;
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

    fn measure_string(&mut self, font: F, id: u32, x: i32) -> Result<u32, Refused> {
        let m = {
            crate::text::measure_string_by_id::measure_string_by_id_at(
                id,
                font_of(font),
                x,
                self.canvas,
            )
        }?;
        Ok(m.width)
    }

    fn measure(&mut self, font: F, text: T, x: i32) -> Result<u32, Refused> {
        text.measure(font_of(font), x, self.canvas)
    }

    fn spaced(&mut self, font: F, bytes: &[u8], x: i32, y: i32) -> Result<(), Refused> {
        let screens = &menus().screens;
        let mut text = bytes.to_vec();
        text.push(0);
        store_in(Buffer::Line, &text)?;
        crate::screens::draw_spaced_number::draw_spaced_number(
            screens,
            font_of(font),
            bytes,
            x,
            y,
            self.canvas,
        )
    }

    fn fatal(&mut self, code: u32, message: &'static str) -> Refused {
        crate::errors::FatalError { code, message }.into()
    }

    fn keep(&mut self, work: W, value: u32) {
        work.cell().set(value);
    }
}

impl GameFont for rc2k_core::screens::best_scores::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::best_scores::Font as F;
        match self {
            F::Name => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::MarkedName => crate::gfx::typefaces::fonts::MARKED_NAME.game_ptr(),
            F::MarkedRank => crate::gfx::typefaces::fonts::MARKED_RANK.game_ptr(),
            F::MarkedTime => crate::gfx::typefaces::fonts::MARKED_TIME.game_ptr(),
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::best_scores::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::best_scores::Work as W;
        let screens = &menus().screens;
        match self {
            W::Rank => &screens.draw_work.seconds,
            W::RowY => &screens.draw_work.work_b0,
            W::CarGlyph => &screens.draw_work.work_b4,
            W::Heading => &screens.draw_work.figure,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_panel::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::stage_panel::Font as F;
        match self {
            F::Label => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            F::Value => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Heading => crate::gfx::typefaces::fonts::HEADING.game_ptr(),
            F::RallyHeading => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::Description => crate::gfx::typefaces::fonts::LABEL.game_ptr(),
            F::Number => crate::gfx::typefaces::fonts::SMALL.game_ptr(),
            F::Title => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_panel::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::stage_panel::Work as W;
        let screens = &menus().screens;
        match self {
            W::Scratch => &screens.draw_work.figure,
            W::Finish => &screens.draw_work.hhmm,
            W::Entry34 => &race().shared.state.settings.stage,
        }
    }
}

impl GameFont for rc2k_core::screens::itinerary::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::itinerary::Font as F;
        match self {
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
            F::Plain => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Marked => crate::gfx::typefaces::fonts::MARKED_NAME.game_ptr(),
            F::Clock => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::MarkedClock => crate::gfx::typefaces::fonts::MARKED_TIME.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::itinerary::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::itinerary::Work as W;
        let screens = &menus().screens;
        match self {
            W::Y => &screens.draw_work.seconds,
            W::Row => &screens.draw_work.work_b0,
            W::Stage => &screens.draw_work.work_b4,
            W::Service => &screens.draw_work.work_0c,
            W::ServiceString => &screens.draw_work.work_08,
        }
    }
}

impl GameFont for rc2k_core::screens::settings_pages::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::settings_pages::Font as F;
        match self {
            F::Value => crate::gfx::typefaces::fonts::LIST.game_ptr(),
            F::Small => crate::gfx::typefaces::fonts::SMALL.game_ptr(),
            F::Title => crate::gfx::typefaces::fonts::BOX_TEXT.game_ptr(),
            F::Head => crate::gfx::typefaces::fonts::HEAD.game_ptr(),
            F::Line => crate::gfx::typefaces::fonts::ITEM_PLAIN.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::settings_pages::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::settings_pages::Work as W;
        let screens = &menus().screens;
        match self {
            W::Scratch => &screens.draw_work.figure,
            W::Seconds => &screens.draw_work.seconds,
            W::Glyph => &screens.draw_work.hhmm,
            W::CheckX => &screens.draw_work.marker_x,
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
    fn pointer(self) -> GamePtr {
        match self {
            rc2k_core::screens::car_pages::Font::Labels => {
                crate::gfx::typefaces::fonts::RANK.game_ptr()
            }
            rc2k_core::screens::car_pages::Font::Figures => {
                crate::gfx::typefaces::fonts::NAME.game_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::car_pages::Work {
    fn cell(self) -> &'static Dword {
        match self {}
    }
}

impl GameFont for rc2k_core::screens::slot_lists::Font {
    fn pointer(self) -> GamePtr {
        match self {
            rc2k_core::screens::slot_lists::Font::Names => {
                crate::gfx::typefaces::fonts::LIST.game_ptr()
            }
            rc2k_core::screens::slot_lists::Font::Setups => {
                crate::gfx::typefaces::fonts::TITLE.game_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::slot_lists::Work {
    fn cell(self) -> &'static Dword {
        let screens = &menus().screens;
        match self {
            rc2k_core::screens::slot_lists::Work::X => &screens.draw_work.marker_x,
            rc2k_core::screens::slot_lists::Work::Y => &screens.draw_work.marker_y,
            rc2k_core::screens::slot_lists::Work::Row => &screens.draw_work.figure,
        }
    }
}

/// The load and save pages' lists' screen.
pub type SlotScreen<'a> =
    GameScreen<'a, rc2k_core::screens::slot_lists::Font, rc2k_core::screens::slot_lists::Work>;

impl GameFont for rc2k_core::screens::savegame_preview::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::savegame_preview::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Title => crate::gfx::typefaces::fonts::RANKING_TITLE.game_ptr(),
            F::RowNumber => crate::gfx::typefaces::fonts::RANK.game_ptr(),
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
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::players_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Small => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::players_results::Work {
    fn cell(self) -> &'static Dword {
        let screens = &menus().screens;
        match self {
            rc2k_core::screens::players_results::Work::Gap => &screens.draw_work.work_b8,
            rc2k_core::screens::players_results::Work::FirstTime => {
                &screens.draw_work.first_player_time
            }
        }
    }
}

impl GameFont for rc2k_core::screens::stage_results::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::stage_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Small => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_results::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::stage_results::Work as W;
        let screens = &menus().screens;
        match self {
            W::Rank => &screens.draw_work.seconds,
            W::Scratch => &screens.draw_work.figure,
            W::FirstTime => &screens.state.menus.first_row_time,
            W::Gap => &screens.draw_work.hhmm,
            W::RowY => &screens.draw_work.work_b0,
        }
    }
}

impl GameFont for rc2k_core::screens::results_pages::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::results_pages::Font as F;
        match self {
            F::Title => crate::gfx::typefaces::fonts::HEAD.game_ptr(),
            F::StageName => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
            F::Place => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            F::Name => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.game_ptr(),
            F::PlayerPlace => crate::gfx::typefaces::fonts::MARKED_RANK.game_ptr(),
            F::PlayerName => crate::gfx::typefaces::fonts::MARKED_NAME.game_ptr(),
            F::PlayerTime => crate::gfx::typefaces::fonts::MARKED_TIME.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::results_pages::Work as W;
        let menus = menus();
        match self {
            W::FirstTime => &menus.screens.draw_work.first_player_time,
            W::OverallFirst => &menus.screens.draw_work.overall_first_time,
            W::Gap => &menus.screens.draw_work.glyph,
            W::Row => &menus.screens.loop_flags.champ_row,
            W::RowEnd => &menus.play.champ_view.row_end,
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::RankingWork {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::results_pages::RankingWork as W;
        let screens = &menus().screens;
        match self {
            W::FirstTime => &screens.draw_work.first_player_time,
            W::Place => &screens.draw_work.seconds,
            W::Gap => &screens.draw_work.glyph,
            W::SpriteY => &screens.draw_work.work_b0,
        }
    }
}

impl GameFont for rc2k_core::screens::player_result::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::player_result::Font as F;
        match self {
            F::Labels => crate::gfx::typefaces::fonts::LABELS.game_ptr(),
            F::Values => crate::gfx::typefaces::fonts::VALUES.game_ptr(),
            F::Measure => crate::gfx::typefaces::fonts::MEASURE_23.game_ptr(),
            F::Name => crate::gfx::typefaces::fonts::NAME.game_ptr(),
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
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::duel_result::Font as F;
        match self {
            F::Big => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            F::Small => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.game_ptr(),
            F::Time => crate::gfx::typefaces::fonts::TIME.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::duel_result::Work {
    fn cell(self) -> &'static Dword {
        let play = &menus().play;
        match self {
            rc2k_core::screens::duel_result::Work::First => &play.results.order[0],
            rc2k_core::screens::duel_result::Work::Second => &play.results.order[1],
        }
    }
}

impl GameFont for rc2k_core::screens::notes::Font {
    fn pointer(self) -> GamePtr {
        match self {
            rc2k_core::screens::notes::Font::Question => {
                crate::gfx::typefaces::fonts::TITLE.game_ptr()
            }
            rc2k_core::screens::notes::Font::Value => crate::gfx::typefaces::fonts::LIST.game_ptr(),
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
    fn pointer(self) -> GamePtr {
        match self {
            rc2k_core::screens::player_details::Font::Value => {
                crate::gfx::typefaces::fonts::LIST.game_ptr()
            }
            rc2k_core::screens::player_details::Font::Measure => {
                crate::gfx::typefaces::fonts::MEASURE.game_ptr()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::player_details::Work {
    fn cell(self) -> &'static Dword {
        let screens = &menus().screens;
        match self {
            rc2k_core::screens::player_details::Work::Seconds => &screens.draw_work.seconds,
            rc2k_core::screens::player_details::Work::Kept => &screens.draw_work.figure,
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
        let screens = &menus().screens;
        match self {
            Work::SpacedFont => &screens.draw_work.figure,
            Work::MarkerX => &screens.draw_work.marker_x,
            Work::MarkerY => &screens.draw_work.marker_y,
            Work::Highlighted => &screens.draw_work.drawing_highlighted_item,
        }
    }
}

impl rc2k_core::screens::menus::MenuScreen for MenuScreen<'_> {
    fn own_button(&mut self, n: usize) -> Result<(), Refused> {
        let screens = &menus().screens;
        crate::screens::buttons::draw_item_button(screens, self.menu_item(n)?, self.canvas)
    }
    fn text_button(
        &mut self,
        button: rc2k_core::screens::menus::TextButton,
        n: usize,
    ) -> Result<(), Refused> {
        use crate::screens::buttons::{ALIGN_CENTRE, ALIGN_RIGHT, ALT, HIGHLIGHTED, PLAIN};
        use rc2k_core::screens::menus::TextButton;
        let (kind, align) = match button {
            TextButton::Right => (&ALT, ALIGN_RIGHT),
            TextButton::Centred => (&PLAIN, ALIGN_CENTRE),
            TextButton::HighlightedRight => (&HIGHLIGHTED, ALIGN_RIGHT),
            TextButton::HighlightedCentred => (&HIGHLIGHTED, ALIGN_CENTRE),
        };
        crate::screens::buttons::draw_text_button(kind, align, self.menu_item(n)?, self.canvas)
    }
    fn blended(&mut self, since_key: u32, a: MenuFont, b: MenuFont) -> Result<MenuFont, Refused> {
        use crate::text::blend_font::blend_font;
        blend_font(since_key, 0x100, a.pointer(), b.pointer())?;
        Ok(MenuFont::Blended)
    }
    fn font_value(&self, f: MenuFont) -> u32 {
        match f {
            MenuFont::Object(id) => id.addr(),
            MenuFont::Blended => GamePtr::to(&crate::text::blend_font::BLENDED_FONT).addr(),
        }
    }
}

impl MenuScreen<'_> {
    /// Item `n` (from 0) of the menu drawn.
    fn menu_item(&self, n: usize) -> Result<&'static crate::menu::menus_store::MenuItem, Refused> {
        self.menu_items
            .get(n)
            .ok_or_else(|| crate::errors::BadInput("an item past its menu's end").into())
    }
}

/// The menus' screen: fonts of the menu tables.
pub type MenuScreen<'a> = GameScreen<'a, MenuFont, rc2k_core::screens::menus::Work>;

impl GameFont for rc2k_core::screens::service::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::service::Font;
        match self {
            Font::Times => crate::gfx::typefaces::fonts::LABEL.game_ptr(),
            Font::Available => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            Font::Remaining => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            Font::Overrun => crate::gfx::typefaces::fonts::MARKED_NAME.game_ptr(),
            Font::Penalty => crate::gfx::typefaces::fonts::MARKED_RANK.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::service::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::service::Work;
        let screens = &menus().screens;
        match self {
            Work::Penalty => &screens.state.driver_stats.money,
            Work::A => &screens.draw_work.seconds,
            Work::B => &screens.draw_work.figure,
        }
    }
}

/// The service area's readouts' screen.
pub type ServiceScreen<'a> =
    GameScreen<'a, rc2k_core::screens::service::Font, rc2k_core::screens::service::Work>;

impl GameFont for rc2k_core::screens::highlighted_caption::Font {
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::highlighted_caption::Font;
        match self {
            Font::Measure => crate::gfx::typefaces::fonts::MEASURE.game_ptr(),
            Font::A => crate::gfx::typefaces::fonts::CAPTION_A.game_ptr(),
            Font::B => crate::gfx::typefaces::fonts::CAPTION_B.game_ptr(),
            // g_blendedFont.
            Font::Blended => GamePtr::to(&crate::text::blend_font::BLENDED_FONT),
        }
    }
}

impl GameWork for rc2k_core::screens::highlighted_caption::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::highlighted_caption::Work;
        let screens = &menus().screens;
        match self {
            Work::MarkerX => &screens.draw_work.marker_x,
            Work::MarkerY => &screens.draw_work.marker_y,
            Work::Width => &screens.draw_work.seconds,
            Work::X => &screens.draw_work.work_b0,
            Work::Y => &screens.draw_work.work_b4,
            Work::Left => &screens.draw_work.glyph,
            Work::Right => &screens.draw_work.work_c4,
            Work::LeftX => &screens.draw_work.marker_end_a,
            Work::LeftY => &screens.draw_work.marker_end_b,
            Work::RightX => &screens.draw_work.marker_end_c,
            Work::RightY => &screens.draw_work.marker_end_d,
        }
    }
}

impl GameFont for rc2k_core::screens::inspection::Font {
    fn pointer(self) -> GamePtr {
        crate::gfx::typefaces::fonts::SMALL.game_ptr()
    }
}

impl GameWork for rc2k_core::screens::inspection::Work {
    fn cell(self) -> &'static Dword {
        let screens = &menus().screens;
        match self {
            rc2k_core::screens::inspection::Work::Lost => &screens.draw_work.figure,
            rc2k_core::screens::inspection::Work::Status => &screens.state.driver_stats.score,
        }
    }
}

impl GameFont for rc2k_core::screens::controls::Font {
    fn pointer(self) -> GamePtr {
        crate::gfx::typefaces::fonts::LIST.game_ptr()
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
    fn pointer(self) -> GamePtr {
        use rc2k_core::screens::championship_table::Font;
        match self {
            Font::Position => crate::gfx::typefaces::fonts::RANK.game_ptr(),
            Font::Text => crate::gfx::typefaces::fonts::NAME.game_ptr(),
            Font::MarkedPosition => crate::gfx::typefaces::fonts::MARKED_RANK.game_ptr(),
            Font::MarkedText => crate::gfx::typefaces::fonts::MARKED_NAME.game_ptr(),
        }
    }
}

impl GameWork for rc2k_core::screens::championship_table::Work {
    fn cell(self) -> &'static Dword {
        let menus = menus();
        match self {
            rc2k_core::screens::championship_table::Work::Position => {
                &menus.screens.loop_flags.champ_row
            }
            rc2k_core::screens::championship_table::Work::End => &menus.play.champ_view.row_end,
        }
    }
}

impl GameFont for rc2k_core::screens::controller_page::Font {
    fn pointer(self) -> GamePtr {
        crate::gfx::typefaces::fonts::LIST.game_ptr()
    }
}

impl GameWork for rc2k_core::screens::controller_page::Work {
    fn cell(self) -> &'static Dword {
        use rc2k_core::screens::controller_page::Work;
        let screens = &menus().screens;
        match self {
            Work::Seconds => &screens.draw_work.seconds,
            Work::Record => &screens.draw_work.work_0c,
            Work::FirstLeft => &screens.draw_work.slider_a_start,
            Work::FirstRight => &screens.draw_work.slider_a_end,
            Work::SecondLeft => &screens.draw_work.slider_b_start,
            Work::SecondRight => &screens.draw_work.slider_b_end,
            Work::Place => &screens.draw_work.figure,
            Work::Buttons => &screens.draw_work.hhmm,
        }
    }
}
