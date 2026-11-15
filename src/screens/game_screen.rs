//! The game's side of `rc2k_core::screens::screen::Screen`: a family's fonts are font records
//! in game memory, its texts places in typed stores or held names, its work variables game
//! dwords; drawn through the text functions and the canvas.

use crate::errors::{BadInput, Refused};
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::typefaces::FontRef;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::screens::text_buffers_store::text_buffers;
use crate::session::DwordLike;
use crate::strings::text_buffer::TextBuf;
use crate::text::draw_text::{Canvas, draw_text_at};

use crate::gfx::owner::Gfx;
use rc2k_core::screens::screen::{Buffer, Drawn, Screen};
use std::marker::PhantomData;

/// A family's font: the handle of its record.
pub trait GameFont: Copy {
    /// The font record in the game.
    fn pointer(self) -> FontRef;
}

/// A font of the menus: one of the font objects a menu item or label names, or the blended
/// font a highlighted item pulses in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuFont {
    Object(crate::gfx::typefaces::FontId),
    Blended,
}

impl GameFont for MenuFont {
    fn pointer(self) -> FontRef {
        match self {
            MenuFont::Object(id) => id.into(),
            MenuFont::Blended => FontRef::Blended,
        }
    }
}

/// A family's work variable: the game state it reports to, if any.
pub trait GameWork: Copy {
    /// Where the value is kept, `None` for a variable nothing reads.
    fn cell(self, menus: &'static Menus, race: &'static Race) -> Option<&'static dyn DwordLike>;
}

/// A text a screen draws (`HeldText`: a place in a typed store, bounded by it, or a name
/// held by value). The game's tables keep texts as addresses; the screens get them as
/// places (a player's name by `player_name`).
pub trait GameText: Copy {
    /// Drawn in `font` at x, y.
    fn draw(
        self,
        o: &Owners<'static>,
        font: FontRef,
        x: i32,
        y: i32,
        canvas: &mut dyn Canvas,
    ) -> Result<Drawn, Refused>;
    /// A shared buffer of the game's, read when drawn.
    fn of_buffer(buffer: Buffer) -> Self;
    /// Measured in `font` from x: its widest line's width.
    fn measure(
        self,
        o: &Owners<'static>,
        font: FontRef,
        x: i32,
        canvas: &mut dyn Canvas,
    ) -> Result<u32, Refused>;
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

/// The text from byte `at` of `run` up to and with its NUL, or to the run's end when it has
/// none (the drawer then refuses it). Nothing past the run.
fn text_in(run: &dyn crate::place::Run, at: usize) -> Vec<u8> {
    let mut text = Vec::new();
    for n in at..run.len() {
        let b = run.get(n);
        text.push(b);
        if b == 0 {
            break;
        }
    }
    text
}

/// A name in a field of a typed table (a record's `name`), held for drawing with the table's
/// bytes as they are now; refused without its NUL inside the field or past the held room.
/// The screens build their rows and draw them in the same call, so nothing changes the
/// field in between.
pub fn held_field(field: &dyn crate::place::Run) -> Result<HeldText, crate::errors::BadInput> {
    let mut text = text_in(field, 0);
    held_name(if text.pop() == Some(0) {
        Some(text)
    } else {
        None
    })
}

/// A table of slot names the screens list: 0x20-byte names one after the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotNames {
    /// The saved games' names (the load and save screens).
    SavedGames,
    /// The setups' names.
    Setups,
    /// The replays' names.
    Replays,
}

impl SlotNames {
    /// The table and the length of one name in it.
    fn table(self, o: &Owners<'static>) -> (&'static dyn crate::place::Run, usize) {
        match self {
            SlotNames::SavedGames => (&o.saves.save_games.names, 0x20),
            SlotNames::Setups => (
                &o.menus.service.set_ups.names,
                crate::service::read_setup_slot::NAME_LEN,
            ),
            SlotNames::Replays => (
                &o.race.shared.replays().names,
                crate::menu::name_lists::NAME_LEN,
            ),
        }
    }
}

/// The name of player `player` (counted from 0), drawn from the names' store.
pub fn player_name_of(player: usize) -> HeldText {
    HeldText::Name(crate::menu::player_names::NameAt::of(player))
}

/// The name an entry's name address names (`player_names::name_address`), drawn from the
/// names' store; refused when it names no player's name.
pub fn player_name(name_at: u32) -> Result<HeldText, crate::errors::BadInput> {
    crate::menu::player_names::index_of(name_at)
        .map(|k| HeldText::Name(crate::menu::player_names::NameAt::of(k)))
        .ok_or(crate::errors::BadInput(
            "a player's name that is no player's name",
        ))
}

/// The texts of a screen whose names are held by value: those, or the game's own buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldText {
    /// The name of slot `.1` in a table of slot names, read when drawn and run on to the
    /// table's end.
    Slot(SlotNames, u32),
    /// A shared text buffer, read when drawn.
    Buffer(Buffer),
    /// A saved car's name in the read saved-game file (from this offset), run on to the
    /// file's end.
    SavedCarName(usize),
    /// A player's name in the names' area, run on to the area's end.
    Name(crate::menu::player_names::NameAt),
    Held(Held),
}

impl HeldText {
    /// The bytes of a text that stays in its table or buffer, as they are now.
    fn read_in_place(self, o: &Owners<'static>) -> Vec<u8> {
        match self {
            HeldText::Slot(names, slot) => {
                let (table, len) = names.table(o);
                text_in(table, (slot as usize).wrapping_mul(len))
            }
            HeldText::Buffer(buffer) => match buffer_at(o.strings, buffer) {
                TextBuf::Text(b) => text_in(b, 0),
                TextBuf::Line(b) => text_in(b, 0),
            },
            HeldText::SavedCarName(at) => text_in(&o.saves.save_games.file, at),
            HeldText::Name(_) | HeldText::Held(_) => Vec::new(),
        }
    }
}

impl GameText for HeldText {
    fn draw(
        self,
        o: &Owners<'static>,
        font: FontRef,
        x: i32,
        y: i32,
        canvas: &mut dyn Canvas,
    ) -> Result<Drawn, Refused> {
        match self {
            HeldText::Slot(..) | HeldText::Buffer(_) | HeldText::SavedCarName(_) => {
                let out = draw_text_at(font, &self.read_in_place(o), x, y, canvas)?;
                Ok(Drawn {
                    y: out.y,
                    too_wide: out.too_wide,
                })
            }
            HeldText::Name(at) => {
                let out = draw_text_at(font, &at.tail(o.menus), x, y, canvas)?;
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

    fn of_buffer(buffer: Buffer) -> Self {
        HeldText::Buffer(buffer)
    }

    fn measure(
        self,
        o: &Owners<'static>,
        font: FontRef,
        x: i32,
        canvas: &mut dyn Canvas,
    ) -> Result<u32, Refused> {
        match self {
            HeldText::Slot(..) | HeldText::Buffer(_) | HeldText::SavedCarName(_) => {
                let m = crate::text::measure_text::measure_text_at(
                    font,
                    &self.read_in_place(o),
                    x,
                    canvas,
                )?;
                Ok(m.width)
            }
            HeldText::Name(at) => {
                let m =
                    crate::text::measure_text::measure_text_at(font, &at.tail(o.menus), x, canvas)?;
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
    /// The menus' state the screens read.
    menus: &'static Menus,
    /// Every owner, for the texts, buffers and work variables the family names.
    owners: Owners<'static>,
    family: PhantomData<(F, W, T)>,
}

impl<'a, F, W, T> GameScreen<'a, F, W, T> {
    /// A screen drawing on `canvas`, the game's state being `o`'s.
    pub fn new(canvas: &'a mut dyn Canvas, o: Owners<'static>) -> Self {
        Self {
            canvas,
            menu_items: &[],
            menus: o.menus,
            owners: o,
            family: PhantomData,
        }
    }
}

fn font_of(f: impl GameFont) -> FontRef {
    f.pointer()
}

/// Where a clip edge is kept.
fn clip_edge_cell(
    gfx: &'static Gfx,
    edge: rc2k_core::screens::screen::Edge,
) -> &'static crate::session::PlainDword {
    let drawing = &gfx.drawing;
    use rc2k_core::screens::screen::Edge;
    match edge {
        Edge::Left => &drawing.blit.clip.left,
        Edge::Right => &drawing.blit.clip.right,
        Edge::Top => &drawing.blit.clip.top,
        Edge::Bottom => &drawing.blit.clip.bottom,
    }
}

/// Where a shared buffer is.
pub fn buffer_at(strings: &'static crate::strings::owner::Strings, buffer: Buffer) -> TextBuf {
    let text_buffers = text_buffers();
    match buffer {
        Buffer::Line => TextBuf::Line(&text_buffers.line),
        Buffer::Text => crate::strings::text_from_ids::text_at(strings),
        Buffer::Cells => TextBuf::Line(&text_buffers.cells),
    }
}

/// `bytes` (with their NUL) into a shared buffer; refused past its end (the line buffer is
/// 0x100 bytes, and the next buffer follows).
pub fn store_in(
    strings: &'static crate::strings::owner::Strings,
    buffer: Buffer,
    bytes: &[u8],
) -> Result<(), Refused> {
    let room = match buffer {
        Buffer::Line => 0x100,
        Buffer::Text => crate::strings::text_from_ids::TEXT_LEN,
        Buffer::Cells => 0x100,
    };
    if bytes.len() > room {
        return Err(BadInput("a text past its buffer").into());
    }
    buffer_at(strings, buffer).write_at(0, bytes, "a shared text buffer")?;
    Ok(())
}

impl<F: GameFont, W: GameWork, T: GameText> Screen for GameScreen<'_, F, W, T> {
    type Font = F;
    type Text = T;
    type Work = W;
    type Error = Refused;

    fn text(&mut self, font: F, text: T, x: i32, y: i32) -> Result<Drawn, Refused> {
        text.draw(&self.owners, font_of(font), x, y, self.canvas)
    }

    fn store(&mut self, buffer: Buffer, bytes: &[u8]) -> Result<(), Refused> {
        store_in(self.owners.strings, buffer, bytes)
    }

    fn draw(&mut self, font: F, buffer: Buffer, x: i32, y: i32) -> Result<Drawn, Refused> {
        self.text(font, T::of_buffer(buffer), x, y)
    }

    fn draw_time(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_time_fixed_pitch::draw_time_fixed_pitch_at;
        draw_time_fixed_pitch_at(
            font_of(font),
            buffer_at(self.owners.strings, buffer),
            right,
            y,
            self.canvas,
        )?;
        Ok(())
    }

    fn draw_clock(&mut self, font: F, buffer: Buffer, right: i32, y: i32) -> Result<(), Refused> {
        use crate::text::draw_clock_fixed_pitch::draw_clock_fixed_pitch_at;
        draw_clock_fixed_pitch_at(
            font_of(font),
            buffer_at(self.owners.strings, buffer),
            right,
            y,
            self.canvas,
        )?;
        Ok(())
    }

    fn string(&mut self, font: F, id: u32, x: i32, y: i32) -> Result<Drawn, Refused> {
        use crate::text::draw_string_by_id::draw_string_by_id_at;
        let out = {
            draw_string_by_id_at(
                crate::strings::append_string_by_id::StringOwners::of(self.owners),
                id,
                font_of(font),
                x,
                y,
                self.canvas,
            )
        }?;
        Ok(Drawn {
            y: out.y,
            too_wide: out.too_wide,
        })
    }

    fn string_bytes(&mut self, id: u32) -> Vec<u8> {
        let mut out = Vec::new();
        crate::menu::event_flow::refuse_here(
            crate::strings::append_string_by_id::push_string(
                crate::strings::append_string_by_id::StringOwners::of(self.owners),
                &mut out,
                id,
            ),
            "screens::game_screen: a string",
        );
        out
    }

    fn strings(&mut self, first: u32, count: u32) -> Result<Vec<u8>, Refused> {
        Ok(crate::strings::text_from_ids::text_from_ids(
            crate::strings::append_string_by_id::StringOwners::of(self.owners),
            first,
            count,
        )?)
    }

    fn sprite(&mut self, glyph: u32, x: i32, y: i32) -> Result<(), Refused> {
        let call = BlitCall::sprite(self.menus, glyph, x, y, self.canvas.clip_right());
        self.canvas.blit_glyph(call)
    }

    fn bar(&mut self, bar: rc2k_core::screens::screen::Bar) -> Result<(), Refused> {
        use crate::gfx::blit_bar::{Bar, blit_bar_in};
        Bar {
            x: bar.x,
            y: bar.y,
            width: bar.width,
            fill_glyph: bar.fill_glyph,
            end_glyph: bar.end_glyph,
            percent: bar.percent,
        }
        .write_in(self.owners.gfx);
        let canvas = &mut *self.canvas;
        blit_bar_in(self.owners.gfx, self.owners.menus, &mut |call| {
            canvas.blit_glyph(call)
        })
    }

    fn blend(&mut self, since_key: u32, a: F, b: F) -> Result<(), Refused> {
        crate::text::blend_font::blend_font(
            self.owners.strings,
            self.owners.gfx,
            self.owners.system,
            since_key,
            0x100,
            font_of(a),
            font_of(b),
        )?;
        Ok(())
    }

    fn clip_edge(&mut self, edge: rc2k_core::screens::screen::Edge) -> Result<u32, Refused> {
        Ok(clip_edge_cell(self.owners.gfx, edge).get())
    }

    fn set_clip_edge(
        &mut self,
        edge: rc2k_core::screens::screen::Edge,
        value: u32,
    ) -> Result<(), Refused> {
        clip_edge_cell(self.owners.gfx, edge).set(value);
        Ok(())
    }

    fn item(&mut self, n: u32, on: bool) -> Result<(), Refused> {
        if on {
            crate::menu::enable_item::enable_item(self.menus, n)
        } else {
            crate::menu::disable_item::disable_item(self.menus, n)
        }
    }

    fn measure_string(&mut self, font: F, id: u32, x: i32) -> Result<u32, Refused> {
        let m = {
            crate::text::measure_string_by_id::measure_string_by_id_at(
                crate::strings::append_string_by_id::StringOwners::of(self.owners),
                id,
                font_of(font),
                x,
                self.canvas,
            )
        }?;
        Ok(m.width)
    }

    fn measure(&mut self, font: F, text: T, x: i32) -> Result<u32, Refused> {
        text.measure(&self.owners, font_of(font), x, self.canvas)
    }

    fn spaced(&mut self, font: F, bytes: &[u8], x: i32, y: i32) -> Result<(), Refused> {
        let mut text = bytes.to_vec();
        text.push(0);
        store_in(self.owners.strings, Buffer::Line, &text)?;
        crate::screens::draw_spaced_number::draw_spaced_number(
            text_buffers(),
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
        if let Some(cell) = work.cell(self.menus, self.owners.race) {
            cell.set(value);
        }
    }
}

impl GameFont for rc2k_core::screens::best_scores::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::best_scores::Font as F;
        match self {
            F::Name => crate::gfx::typefaces::fonts::NAME.into(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.into(),
            F::Time => crate::gfx::typefaces::fonts::TIME.into(),
            F::MarkedName => crate::gfx::typefaces::fonts::MARKED_NAME.into(),
            F::MarkedRank => crate::gfx::typefaces::fonts::MARKED_RANK.into(),
            F::MarkedTime => crate::gfx::typefaces::fonts::MARKED_TIME.into(),
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::best_scores::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::best_scores::Work as W;
        let screens = &menus.screens;
        match self {
            W::Rank => None,
            W::RowY => Some(&screens.draw_work.button_x),
            W::CarGlyph => Some(&screens.draw_work.button_y),
            W::Heading => None,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_panel::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::stage_panel::Font as F;
        match self {
            F::Label => crate::gfx::typefaces::fonts::RANK.into(),
            F::Value => crate::gfx::typefaces::fonts::NAME.into(),
            F::Heading => crate::gfx::typefaces::fonts::HEADING.into(),
            F::RallyHeading => crate::gfx::typefaces::fonts::TIME.into(),
            F::Description => crate::gfx::typefaces::fonts::LABEL.into(),
            F::Number => crate::gfx::typefaces::fonts::SMALL.into(),
            F::Title => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
            F::Time => crate::gfx::typefaces::fonts::TIME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_panel::Work {
    fn cell(self, _menus: &'static Menus, race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::stage_panel::Work as W;
        match self {
            W::Scratch => None,
            W::Finish => None,
            W::Entry34 => Some(&race.shared.state.settings.stage),
        }
    }
}

impl GameFont for rc2k_core::screens::itinerary::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::itinerary::Font as F;
        match self {
            F::Heading => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
            F::Plain => crate::gfx::typefaces::fonts::NAME.into(),
            F::Marked => crate::gfx::typefaces::fonts::MARKED_NAME.into(),
            F::Clock => crate::gfx::typefaces::fonts::TIME.into(),
            F::MarkedClock => crate::gfx::typefaces::fonts::MARKED_TIME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::itinerary::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::itinerary::Work as W;
        let screens = &menus.screens;
        match self {
            W::Y => None,
            W::Row => Some(&screens.draw_work.button_x),
            W::Stage => Some(&screens.draw_work.button_y),
            W::Service => None,
            W::ServiceString => None,
        }
    }
}

impl GameFont for rc2k_core::screens::settings_pages::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::settings_pages::Font as F;
        match self {
            F::Value => crate::gfx::typefaces::fonts::LIST.into(),
            F::Small => crate::gfx::typefaces::fonts::SMALL.into(),
            F::Title => crate::gfx::typefaces::fonts::BOX_TEXT.into(),
            F::Head => crate::gfx::typefaces::fonts::HEAD.into(),
            F::Line => crate::gfx::typefaces::fonts::ITEM_PLAIN.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::settings_pages::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::settings_pages::Work as W;
        match self {
            W::Scratch => None,
            W::Seconds => None,
            W::Glyph => None,
            W::CheckX => None,
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
    fn pointer(self) -> FontRef {
        match self {
            rc2k_core::screens::car_pages::Font::Labels => {
                crate::gfx::typefaces::fonts::RANK.into()
            }
            rc2k_core::screens::car_pages::Font::Figures => {
                crate::gfx::typefaces::fonts::NAME.into()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::car_pages::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {}
    }
}

impl GameFont for rc2k_core::screens::slot_lists::Font {
    fn pointer(self) -> FontRef {
        match self {
            rc2k_core::screens::slot_lists::Font::Names => {
                crate::gfx::typefaces::fonts::LIST.into()
            }
            rc2k_core::screens::slot_lists::Font::Setups => {
                crate::gfx::typefaces::fonts::TITLE.into()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::slot_lists::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {
            rc2k_core::screens::slot_lists::Work::X => None,
            rc2k_core::screens::slot_lists::Work::Y => None,
            rc2k_core::screens::slot_lists::Work::Row => None,
        }
    }
}

/// The load and save pages' lists' screen.
pub type SlotScreen<'a> =
    GameScreen<'a, rc2k_core::screens::slot_lists::Font, rc2k_core::screens::slot_lists::Work>;

impl GameFont for rc2k_core::screens::savegame_preview::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::savegame_preview::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.into(),
            F::Title => crate::gfx::typefaces::fonts::RANKING_TITLE.into(),
            F::RowNumber => crate::gfx::typefaces::fonts::RANK.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::savegame_preview::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
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
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::players_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.into(),
            F::Small => crate::gfx::typefaces::fonts::TIME.into(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::players_results::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {
            rc2k_core::screens::players_results::Work::Gap => None,
            rc2k_core::screens::players_results::Work::FirstTime => None,
        }
    }
}

impl GameFont for rc2k_core::screens::stage_results::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::stage_results::Font as F;
        match self {
            F::Rows => crate::gfx::typefaces::fonts::NAME.into(),
            F::Small => crate::gfx::typefaces::fonts::TIME.into(),
            F::Rank => crate::gfx::typefaces::fonts::RANK.into(),
            F::Fastest => crate::gfx::typefaces::fonts::LABELS.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::stage_results::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::stage_results::Work as W;
        let screens = &menus.screens;
        match self {
            W::Rank => None,
            W::Scratch => None,
            W::FirstTime => Some(&screens.state.menus.first_row_time),
            W::Gap => None,
            W::RowY => Some(&screens.draw_work.button_x),
        }
    }
}

impl GameFont for rc2k_core::screens::results_pages::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::results_pages::Font as F;
        match self {
            F::Title => crate::gfx::typefaces::fonts::HEAD.into(),
            F::StageName => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
            F::Place => crate::gfx::typefaces::fonts::RANK.into(),
            F::Name => crate::gfx::typefaces::fonts::NAME.into(),
            F::Time => crate::gfx::typefaces::fonts::TIME.into(),
            F::PlayerPlace => crate::gfx::typefaces::fonts::MARKED_RANK.into(),
            F::PlayerName => crate::gfx::typefaces::fonts::MARKED_NAME.into(),
            F::PlayerTime => crate::gfx::typefaces::fonts::MARKED_TIME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::results_pages::Work as W;
        match self {
            W::FirstTime => None,
            W::OverallFirst => None,
            W::Gap => None,
            W::Row => Some(&menus.screens.loop_flags.champ_row),
            W::RowEnd => Some(&menus.play.champ_view.rows.row_end),
        }
    }
}

impl GameWork for rc2k_core::screens::results_pages::RankingWork {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::results_pages::RankingWork as W;
        let screens = &menus.screens;
        match self {
            W::FirstTime => None,
            W::Place => None,
            W::Gap => None,
            W::SpriteY => Some(&screens.draw_work.button_x),
        }
    }
}

impl GameFont for rc2k_core::screens::player_result::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::player_result::Font as F;
        match self {
            F::Labels => crate::gfx::typefaces::fonts::LABELS.into(),
            F::Values => crate::gfx::typefaces::fonts::VALUES.into(),
            F::Measure => crate::gfx::typefaces::fonts::MEASURE_23.into(),
            F::Name => crate::gfx::typefaces::fonts::NAME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::player_result::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
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
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::duel_result::Font as F;
        match self {
            F::Big => crate::gfx::typefaces::fonts::NAME.into(),
            F::Small => crate::gfx::typefaces::fonts::RANK.into(),
            F::Stage => crate::gfx::typefaces::fonts::STAGE_TITLE.into(),
            F::Time => crate::gfx::typefaces::fonts::TIME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::duel_result::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        let play = &menus.play;
        match self {
            rc2k_core::screens::duel_result::Work::First => Some(&play.results.order[0]),
            rc2k_core::screens::duel_result::Work::Second => Some(&play.results.order[1]),
        }
    }
}

impl GameFont for rc2k_core::screens::notes::Font {
    fn pointer(self) -> FontRef {
        match self {
            rc2k_core::screens::notes::Font::Question => crate::gfx::typefaces::fonts::TITLE.into(),
            rc2k_core::screens::notes::Font::Value => crate::gfx::typefaces::fonts::LIST.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::notes::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {}
    }
}

/// A one-string screen drawn: what it leaves in the registers.
pub fn draw_note(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
    note: &rc2k_core::screens::notes::Note,
) -> Result<rc2k_core::screens::screen::Drawn, Refused> {
    rc2k_core::screens::notes::note(
        &mut GameScreen::<rc2k_core::screens::notes::Font, rc2k_core::screens::notes::Work>::new(
            canvas, o,
        ),
        note,
    )
}

impl GameFont for rc2k_core::screens::player_details::Font {
    fn pointer(self) -> FontRef {
        match self {
            rc2k_core::screens::player_details::Font::Value => {
                crate::gfx::typefaces::fonts::LIST.into()
            }
            rc2k_core::screens::player_details::Font::Measure => {
                crate::gfx::typefaces::fonts::MEASURE.into()
            }
        }
    }
}

impl GameWork for rc2k_core::screens::player_details::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {
            rc2k_core::screens::player_details::Work::Seconds => None,
            rc2k_core::screens::player_details::Work::Kept => None,
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
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::menus::Work;
        match self {
            Work::SpacedFont => None,
            Work::MarkerX => None,
            Work::MarkerY => None,
            Work::Highlighted => None,
        }
    }
}

impl rc2k_core::screens::menus::MenuScreen for MenuScreen<'_> {
    fn own_button(&mut self, n: usize) -> Result<(), Refused> {
        crate::screens::buttons::draw_item_button(self.menus, self.menu_item(n)?, self.canvas)
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
        crate::screens::buttons::draw_text_button(
            self.owners,
            kind,
            align,
            self.menu_item(n)?,
            self.canvas,
        )
    }
    fn blended(&mut self, since_key: u32, a: MenuFont, b: MenuFont) -> Result<MenuFont, Refused> {
        use crate::text::blend_font::blend_font;
        blend_font(
            self.owners.strings,
            self.owners.gfx,
            self.owners.system,
            since_key,
            0x100,
            a.pointer(),
            b.pointer(),
        )?;
        Ok(MenuFont::Blended)
    }
    fn list_sprite(&mut self, _list: usize, _x: i32, _y: i32) -> Result<(), Refused> {
        // The list's address as a sprite number is past every sprite table
        // (`gfx::blit_glyph::glyph_record` refuses it).
        Err(crate::errors::BadInput("a sprite list drawn as a sprite").into())
    }
    fn font_value(&self, f: MenuFont) -> u32 {
        match f {
            MenuFont::Object(id) => id.number(),
            MenuFont::Blended => crate::gfx::typefaces::BLENDED_NUMBER,
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
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::service::Font;
        match self {
            Font::Times => crate::gfx::typefaces::fonts::LABEL.into(),
            Font::Available => crate::gfx::typefaces::fonts::NAME.into(),
            Font::Remaining => crate::gfx::typefaces::fonts::RANK.into(),
            Font::Overrun => crate::gfx::typefaces::fonts::MARKED_NAME.into(),
            Font::Penalty => crate::gfx::typefaces::fonts::MARKED_RANK.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::service::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::service::Work;
        let screens = &menus.screens;
        match self {
            Work::Penalty => Some(&screens.state.driver_stats.money),
            Work::A => None,
            Work::B => None,
        }
    }
}

/// The service area's readouts' screen.
pub type ServiceScreen<'a> =
    GameScreen<'a, rc2k_core::screens::service::Font, rc2k_core::screens::service::Work>;

impl GameFont for rc2k_core::screens::highlighted_caption::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::highlighted_caption::Font;
        match self {
            Font::Measure => crate::gfx::typefaces::fonts::MEASURE.into(),
            Font::A => crate::gfx::typefaces::fonts::CAPTION_A.into(),
            Font::B => crate::gfx::typefaces::fonts::CAPTION_B.into(),
            // The blended font.
            Font::Blended => FontRef::Blended,
        }
    }
}

impl GameWork for rc2k_core::screens::highlighted_caption::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::highlighted_caption::Work;
        let screens = &menus.screens;
        match self {
            Work::MarkerX => None,
            Work::MarkerY => None,
            Work::Width => None,
            Work::X => Some(&screens.draw_work.button_x),
            Work::Y => Some(&screens.draw_work.button_y),
            Work::Left => None,
            Work::Right => None,
            Work::LeftX => None,
            Work::LeftY => None,
            Work::RightX => None,
            Work::RightY => None,
        }
    }
}

impl GameFont for rc2k_core::screens::inspection::Font {
    fn pointer(self) -> FontRef {
        crate::gfx::typefaces::fonts::SMALL.into()
    }
}

impl GameWork for rc2k_core::screens::inspection::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        let screens = &menus.screens;
        match self {
            rc2k_core::screens::inspection::Work::Lost => None,
            rc2k_core::screens::inspection::Work::Status => Some(&screens.state.driver_stats.score),
        }
    }
}

impl GameFont for rc2k_core::screens::controls::Font {
    fn pointer(self) -> FontRef {
        crate::gfx::typefaces::fonts::LIST.into()
    }
}

impl GameWork for rc2k_core::screens::controls::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {}
    }
}

/// The controller pages' lists' screen.
pub type ControlsScreen<'a> =
    GameScreen<'a, rc2k_core::screens::controls::Font, rc2k_core::screens::controls::Work>;

impl GameFont for rc2k_core::screens::championship_table::Font {
    fn pointer(self) -> FontRef {
        use rc2k_core::screens::championship_table::Font;
        match self {
            Font::Position => crate::gfx::typefaces::fonts::RANK.into(),
            Font::Text => crate::gfx::typefaces::fonts::NAME.into(),
            Font::MarkedPosition => crate::gfx::typefaces::fonts::MARKED_RANK.into(),
            Font::MarkedText => crate::gfx::typefaces::fonts::MARKED_NAME.into(),
        }
    }
}

impl GameWork for rc2k_core::screens::championship_table::Work {
    fn cell(self, menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        match self {
            rc2k_core::screens::championship_table::Work::Position => {
                Some(&menus.screens.loop_flags.champ_row)
            }
            rc2k_core::screens::championship_table::Work::End => {
                Some(&menus.play.champ_view.rows.row_end)
            }
        }
    }
}

impl GameFont for rc2k_core::screens::controller_page::Font {
    fn pointer(self) -> FontRef {
        crate::gfx::typefaces::fonts::LIST.into()
    }
}

impl GameWork for rc2k_core::screens::controller_page::Work {
    fn cell(self, _menus: &'static Menus, _race: &'static Race) -> Option<&'static dyn DwordLike> {
        use rc2k_core::screens::controller_page::Work;
        match self {
            Work::Seconds => None,
            Work::Record => None,
            Work::FirstLeft => None,
            Work::FirstRight => None,
            Work::SecondLeft => None,
            Work::SecondRight => None,
            Work::Place => None,
            Work::Buttons => None,
        }
    }
}
