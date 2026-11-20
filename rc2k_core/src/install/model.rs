//! The installer's pages as typed data: a page is a `PageDef` (a title and items), made by
//! `build` from what the installation is (`Facts`); a key goes to it through `PageDef::key`.
//!
//! An item is a link (leaves the page with its answer, or runs its action), a text, or an edit
//! (the folder's). Each has its marks (`Marks`: hidden, not to be chosen, or both for the item lit
//! under the highlight) and the box it is placed in. The pieces a page is drawn from are made from
//! it by `pieces::lay_out` (`PageDef` is a `ListSource`).
//!
//! The model holds only the kinds the installer's pages use: no item is a choice or a value.

use super::drives;
use super::edits::{self, EditKey};
use super::flow::Page;
use super::items::{self, EnterWay, ItemKey, LinkWay, PageKey};
use super::layout::Area;
use super::pieces::{self, ListSource};
use crate::Invalid;

/// One of the installer's lists. The drive page has three, by the drives found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum List {
    Language,
    Main,
    /// The drive page for up to 8 drives, up to 16, and more.
    Drives8,
    Drives16,
    DrivesMore,
    Folder,
    NoRoom,
    Removal,
}

impl List {
    /// The lists, in the order of the enum.
    pub const ALL: [List; 8] = [
        List::Language,
        List::Main,
        List::Drives8,
        List::Drives16,
        List::DrivesMore,
        List::Folder,
        List::NoRoom,
        List::Removal,
    ];

    /// The list `page` is shown as, for `drives` drives found.
    pub fn of(page: Page, drives: u32) -> List {
        match page {
            Page::Language => List::Language,
            Page::Main => List::Main,
            Page::Drive => match drives::menu(drives) {
                drives::DriveMenu::Eight => List::Drives8,
                drives::DriveMenu::Sixteen => List::Drives16,
                drives::DriveMenu::More => List::DrivesMore,
            },
            Page::Folder => List::Folder,
            Page::NoRoom => List::NoRoom,
            Page::Removal => List::Removal,
        }
    }

    /// The page the list is shown for.
    pub fn page(self) -> Page {
        match self {
            List::Language => Page::Language,
            List::Main => Page::Main,
            List::Drives8 | List::Drives16 | List::DrivesMore => Page::Drive,
            List::Folder => Page::Folder,
            List::NoRoom => Page::NoRoom,
            List::Removal => Page::Removal,
        }
    }

    /// The item highlighted the first time the list is shown: after the two items before the
    /// drives, and on the removal page's first answer.
    pub const fn first_highlight(self) -> u32 {
        match self {
            List::Drives8 | List::Drives16 | List::DrivesMore | List::Removal => 2,
            _ => 0,
        }
    }

    /// The most items the list's drives can fill: its items, less the two before them.
    fn drive_room(self) -> usize {
        match self {
            List::Drives8 => 8,
            List::Drives16 => 16,
            _ => 24,
        }
    }
}

/// An item's marks: hidden, not to be chosen, or both (lit: the highlighted item).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub hidden: bool,
    pub unusable: bool,
}

impl Marks {
    pub const NONE: Marks = Marks {
        hidden: false,
        unusable: false,
    };
    pub const LIT: Marks = Marks {
        hidden: true,
        unusable: true,
    };

    /// The marks as the top two bits of an item's first byte.
    pub const fn bits(self) -> u8 {
        (self.hidden as u8) << 6 | (self.unusable as u8) << 7
    }
}

/// What a link does when it is chosen, besides leaving the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// The not-enough-room page's "go on anyway": the lack of room is ignored.
    GoOnAnyway,
}

/// An edit item: the folder's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    /// The text is being edited.
    pub editing: bool,
    /// The caret is drawn after the text while it is edited.
    pub shows_caret: bool,
    /// The most bytes of the text.
    pub most: u8,
    /// The string id of the text Enter puts in when the text is blank.
    pub default_text: u16,
    /// The kind of the font its text is drawn in.
    pub font_kind: u8,
    /// The widest the text may be, in pixels.
    pub width: u16,
    /// The text, without a NUL.
    pub text: Vec<u8>,
    /// The text as it was when the edit began (what Escape puts back).
    backup: Vec<u8>,
}

impl Edit {
    /// The edit's flags as `pieces::caret` reads them: bit 0 being edited, bit 1 the caret shown.
    pub fn flags(&self) -> u8 {
        self.editing as u8 | (self.shows_caret as u8) << 1
    }
}

/// What an item is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Leaves the page with `answer` (0 is back); a negative answer (bit 7) stays.
    Link {
        answer: u8,
        action: Option<Action>,
    },
    /// A text, in the font of `font_kind`; `ends_line` ends its line after the text.
    Text {
        font_kind: u8,
        ends_line: bool,
    },
    Edit(Edit),
}

/// A box of the page: items are placed in it. `None` is the page's own box, the title's.
pub type BoxId = Option<u8>;

/// The boxes items are placed in: the title item's (0), the title's (1), the drive texts'
/// (2 to 6), the drive page's heading (7) and its back (8).
const BOXES: [Area; 9] = [
    Area {
        x: 50,
        y: 150,
        width: 750,
        height: 44,
    },
    Area {
        x: 50,
        y: 150,
        width: 750,
        height: 400,
    },
    Area {
        x: 125,
        y: 150,
        width: 200,
        height: 400,
    },
    Area {
        x: 325,
        y: 150,
        width: 200,
        height: 400,
    },
    Area {
        x: 525,
        y: 150,
        width: 200,
        height: 400,
    },
    Area {
        x: 225,
        y: 150,
        width: 200,
        height: 400,
    },
    Area {
        x: 425,
        y: 150,
        width: 200,
        height: 400,
    },
    Area {
        x: 50,
        y: 200,
        width: 750,
        height: 30,
    },
    Area {
        x: 50,
        y: 495,
        width: 750,
        height: 30,
    },
];

/// The box of the title (and of every item with no box of its own).
const TITLE_BOX: u8 = 1;

/// The box `id` is, and whether it is the title's.
pub fn box_area(id: BoxId) -> (Area, bool) {
    let n = id.unwrap_or(TITLE_BOX);
    (BOXES[n as usize], id.is_none() || n == TITLE_BOX)
}

/// An item of a page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub kind: Kind,
    /// The item's string id.
    pub text: u16,
    pub marks: Marks,
    /// The box the item is placed in.
    pub box_id: BoxId,
    /// Bits 4 and 5 of the item's first byte (a label's absence, the swap of its kind when
    /// measured).
    pub style: u8,
}

impl Item {
    fn new(kind: Kind, text: u16) -> Item {
        Item {
            kind,
            text,
            marks: Marks::NONE,
            box_id: None,
            style: 0,
        }
    }

    fn link(text: u16, answer: u8) -> Item {
        Item::new(
            Kind::Link {
                answer,
                action: None,
            },
            text,
        )
    }

    fn text(text: u16, font_kind: u8) -> Item {
        Item::new(
            Kind::Text {
                font_kind,
                ends_line: false,
            },
            text,
        )
    }

    fn in_box(mut self, id: u8) -> Item {
        self.box_id = Some(id);
        self
    }

    /// The item's first byte: its kind in the low four bits, its style, its marks.
    pub fn first(&self) -> u8 {
        let kind = match self.kind {
            Kind::Link { .. } => 1,
            Kind::Edit(_) => 4,
            Kind::Text { .. } => 5,
        };
        kind | self.style | self.marks.bits()
    }

    /// The item's second byte: a link's answer, a text's font kind.
    pub fn count_byte(&self) -> u8 {
        match &self.kind {
            Kind::Link { answer, .. } => *answer,
            Kind::Text { font_kind, .. } => *font_kind,
            Kind::Edit(_) => 1,
        }
    }

    /// The item cannot be chosen: marked hidden or unusable (not both), or a text.
    pub fn unusable(&self) -> bool {
        self.marks.hidden != self.marks.unusable || matches!(self.kind, Kind::Text { .. })
    }
}

/// What the pages depend on besides the answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// This version is installed.
    pub same_version: bool,
    /// An installation is recorded.
    pub installed: bool,
    /// The update script's entry: the string id the main page's first item shows.
    pub update_entry: Option<u16>,
    /// The languages found.
    pub languages: u32,
    /// The drives listed.
    pub drives: u32,
    /// 1 a bad folder, 2 no room.
    pub path_problem: u32,
    /// The folder field's most bytes and widest text, by the system.
    pub folder_most: u8,
    pub folder_width: u16,
}

/// The string id of the update script's second item.
const UPDATE_TITLE: u16 = 0xBD;

/// A page: its title and items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageDef {
    pub list: List,
    /// The title's string id.
    pub title: u16,
    pub items: Vec<Item>,
    /// The title's item: where and in what font the title is drawn.
    title_item: Item,
}

/// The owner of a piece: the page's title or one of its items.
pub use super::pieces::Owner;

/// A page made: its items and the marks the facts give them.
pub fn build(list: List, facts: &Facts) -> Result<PageDef, Invalid> {
    let (title, items) = match list {
        List::Language => (0x9B, language_items()),
        List::Main => (0x95, main_items(facts.update_entry)),
        List::Drives8 | List::Drives16 | List::DrivesMore => {
            (0x9A, drive_items(list, facts.drives as usize)?)
        }
        List::Folder => (0x9A, folder_items(facts)),
        List::NoRoom => (0x9A, room_items()),
        List::Removal => (0xB6, removal_items()),
    };
    let mut page = PageDef {
        list,
        title,
        items,
        title_item: Item::text(0, 9).in_box(0),
    };
    page.apply_marks(facts)?;
    Ok(page)
}

fn language_items() -> Vec<Item> {
    let mut items: Vec<Item> = (0..9)
        .map(|k| Item::link(0x102B + k as u16, k as u8 + 1))
        .collect();
    items.push(Item::link(0x99, 0));
    items
}

fn main_items(update_entry: Option<u16>) -> Vec<Item> {
    let mut items = vec![
        Item::link(0x96, 1),
        Item::link(0x97, 2),
        Item::link(0x98, 3),
        Item::link(0x99, 0),
    ];
    if let Some(entry) = update_entry {
        items[0] = Item::text(entry, 0x0C);
        items[1].text = UPDATE_TITLE;
    }
    items
}

fn drive_items(list: List, drives: usize) -> Result<Vec<Item>, Invalid> {
    if drives > list.drive_room() {
        return Err(Invalid("more drives than the drive page's list holds"));
    }
    let mut items = vec![
        Item::text(0x9D, 0x0C).in_box(7),
        Item::link(0xBF, 0).in_box(8),
    ];
    for k in 0..drives {
        let box_id = match (list, k) {
            (List::Drives8, _) => 3,
            (List::Drives16, 0..8) => 5,
            (List::Drives16, _) => 6,
            (_, 0..8) => 2,
            (_, 8..16) => 3,
            _ => 4,
        };
        items.push(Item::link(0x1011 + k as u16, 1).in_box(box_id));
    }
    Ok(items)
}

fn folder_items(facts: &Facts) -> Vec<Item> {
    let mut edit = Item::new(
        Kind::Edit(Edit {
            editing: false,
            shows_caret: true,
            most: facts.folder_most,
            default_text: 0xA0,
            font_kind: 0x0B,
            width: facts.folder_width,
            text: Vec::new(),
            backup: Vec::new(),
        }),
        0,
    );
    edit.style = 0x20;
    vec![
        Item::text(0x9E, 0x0C),
        edit,
        Item::text(0xAA, 0x0A),
        Item::text(0xAC, 0x0A),
        Item::text(0xAD, 0x0A),
        Item::text(0xAB, 0x0A),
    ]
}

fn ending_line(text: u16) -> Item {
    Item::new(
        Kind::Text {
            font_kind: 0x0A,
            ends_line: true,
        },
        text,
    )
}

fn room_items() -> Vec<Item> {
    vec![
        ending_line(0xAD),
        Item::new(
            Kind::Link {
                answer: 1,
                action: Some(Action::GoOnAnyway),
            },
            0xAE,
        ),
        Item::link(0xBF, 0),
    ]
}

fn removal_items() -> Vec<Item> {
    vec![
        ending_line(0xB7),
        Item::link(0xB8, 1),
        Item::link(0xB9, 2),
        Item::link(0xBF, 0),
    ]
}

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keyed {
    /// The key word the caller goes on reading: the key itself, with bit 3 of its bits set when an
    /// edit ended; after a character was typed, the field's allowed width (callers read only the
    /// low byte as a key).
    pub key_word: u32,
    /// The key did something.
    pub taken: bool,
    /// It was only an edit (not Escape or Enter on one).
    pub plain_edit: bool,
    /// The page is left with this answer.
    pub leave: Option<u8>,
    /// The lack of room is to be ignored.
    pub ignore_room: bool,
    /// The key was not one of the item's: a screenshot key is looked for in it.
    pub other: bool,
    /// The key is to be noted as taken (the menus' sound reads it).
    pub noted: bool,
}

/// What a key asks of the host: the width of a text, and a string.
pub trait EditHost {
    type Error;
    /// The width in pixels of `text` (without its NUL) in the edit's font.
    fn text_width(&mut self, text: &[u8]) -> Result<u32, Self::Error>;
    /// The text of string `id`.
    fn string(&mut self, id: u16) -> Vec<u8>;
    /// The folder a blank folder text becomes, if the system has a first one (else the field's
    /// default text).
    fn first_folder(&mut self) -> Option<Vec<u8>>;
}

/// The page's own key, taken after the item's: the folder page's Escape and Enter that ended an
/// edit (their key word has bit 3 of the bits set). Whether it was taken, and the answer it leaves with.
pub fn own_key(page: Page, key_word: u32) -> (bool, Option<u8>) {
    if page != Page::Folder {
        return (false, None);
    }
    use items::FolderKey;
    let way = FolderKey::of(key_word);
    // The installer's lists have no flag that keeps a page on Escape.
    let answer = way.answer(|| 0);
    let taken = match way {
        FolderKey::NotEdited => key_word == 0,
        FolderKey::Back => answer.is_some(),
        FolderKey::On => true,
        FolderKey::Other => false,
    };
    (taken, answer)
}

impl PageDef {
    /// The marks of every item cleared.
    pub fn clear_marks(&mut self) {
        for item in &mut self.items {
            item.marks = Marks::NONE;
        }
    }

    /// The marks the installation gives the page's items: the language page's items past the
    /// languages found hidden, the main page's items by what is installed, the folder page's
    /// notices that are not the problem hidden.
    pub fn apply_marks(&mut self, facts: &Facts) -> Result<(), Invalid> {
        match self.list {
            List::Language => {
                if facts.languages > 9 {
                    return Err(Invalid("more languages than the language page's items"));
                }
                for item in &mut self.items[facts.languages as usize..9] {
                    item.marks.hidden = true;
                }
            }
            List::Main => {
                if !facts.same_version {
                    self.items[1].marks.unusable = true;
                }
                if !facts.installed {
                    self.items[2].marks.unusable = true;
                }
                if self.items[1].text == UPDATE_TITLE {
                    self.items[1].marks.unusable = false;
                    self.items[2].marks.hidden = true;
                }
            }
            List::Folder => {
                let (bad_hidden, room_hidden) = (facts.path_problem != 1, facts.path_problem != 2);
                for (hidden, notices) in [(bad_hidden, 2..4), (room_hidden, 4..6)] {
                    if hidden {
                        for item in &mut self.items[notices] {
                            item.marks.hidden = true;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// The item the highlight is on, lit when it has no marks: while it is being edited, or by
    /// the blink (the clock's bit 3 clear).
    pub fn mark_highlight(&mut self, highlight: u32, clock: u32) {
        let Some(item) = self.items.get_mut(highlight as usize) else {
            return;
        };
        if item.marks != Marks::NONE {
            return;
        }
        let editing = match &item.kind {
            Kind::Edit(edit) => Some(edit.editing),
            _ => None,
        };
        if items::lit(editing, clock) {
            item.marks = Marks::LIT;
        }
    }

    /// The item `owner` names.
    pub fn owner_item(&self, owner: Owner) -> &Item {
        match owner {
            Owner::Title => &self.title_item,
            Owner::Item(k) => &self.items[k as usize],
        }
    }

    /// From item `from` on, the first edit item: its number, or the end.
    pub fn next_edit(&self, from: u32) -> (u32, bool) {
        items::next_edit(from, self.items.len() as u32, |n| {
            self.items[n as usize].first()
        })
    }

    /// Edit item `n` begins to be edited: its text is cut at its most bytes and kept for
    /// Escape.
    pub fn start_edit(&mut self, n: u32) {
        if let Some(Item {
            kind: Kind::Edit(edit),
            ..
        }) = self.items.get_mut(n as usize)
        {
            edit.text.truncate(edit.most as usize);
            edit.backup = edit.text.clone();
            edit.editing = true;
        }
    }

    /// `key` (the key in its low byte, its bits in the next) on item `highlight`.
    pub fn key<H: EditHost>(
        &mut self,
        highlight: u32,
        key: u32,
        host: &mut H,
    ) -> Result<Keyed, H::Error> {
        let item = &mut self.items[highlight as usize];
        let mut keyed = Keyed {
            key_word: key,
            taken: false,
            plain_edit: false,
            leave: None,
            ignore_room: false,
            other: false,
            noted: false,
        };
        let Some(way) = items::page_key(key as u8) else {
            // No key, or one of 0x80 and above.
            keyed.taken = key as u8 == 0;
            return Ok(keyed);
        };
        if let Kind::Edit(edit) = &mut item.kind
            && edit.editing
        {
            let (key_word, plain_edit) = edit_key(edit, key, host)?;
            keyed.key_word = key_word;
            keyed.plain_edit = plain_edit;
            keyed.taken = plain_edit;
            keyed.noted = plain_edit;
            keyed.other = true;
            return Ok(keyed);
        }
        match way {
            // Arrows and number keys do nothing to a link, a text or an edit not being edited.
            PageKey::Item(ItemKey::Up | ItemKey::Down | ItemKey::Left | ItemKey::Right) => {}
            PageKey::Item(ItemKey::Number) => {}
            PageKey::Enter => {
                self.enter(highlight, &mut keyed);
                keyed.noted = keyed.taken;
                return Ok(keyed);
            }
            PageKey::Escape => {
                keyed.leave = Some(0);
                keyed.taken = true;
                return Ok(keyed);
            }
            PageKey::Other => keyed.other = true,
        }
        Ok(keyed)
    }

    /// Enter on item `highlight`, by its kind.
    fn enter(&mut self, highlight: u32, keyed: &mut Keyed) {
        let kind = self.items[highlight as usize].first() & 0xF;
        match EnterWay::of(kind) {
            Some(EnterWay::StartEdit) => {
                self.start_edit(highlight);
                keyed.taken = true;
            }
            Some(EnterWay::Link) => {
                let Kind::Link { answer, action } = self.items[highlight as usize].kind else {
                    return;
                };
                match (action, items::link(0, || answer)) {
                    (Some(Action::GoOnAnyway), _) => {
                        keyed.ignore_room = true;
                        keyed.leave = Some(1);
                        keyed.taken = true;
                    }
                    (None, LinkWay::Leave(next)) => {
                        keyed.leave = Some(next);
                        keyed.taken = next == 0;
                    }
                    (None, _) => keyed.taken = answer == 0,
                }
            }
            // No item is an action.
            _ => {}
        }
    }
}

/// A key while the edit is being edited: the key word to go on reading, and whether it was a
/// plain edit (not an end of it).
fn edit_key<H: EditHost>(edit: &mut Edit, key: u32, host: &mut H) -> Result<(u32, bool), H::Error> {
    let ended = (key | 0x800, false);
    let plain = |key_word| Ok((key_word, true));
    let ch = if key & 0x100 != 0 {
        key as u8
    } else {
        match EditKey::of(key as u8) {
            EditKey::Escape => {
                edit.editing = false;
                edit.text = edit.backup.clone();
                return Ok(ended);
            }
            EditKey::Enter => {
                edit.editing = false;
                let mut text = edit.text.clone();
                text.push(0);
                // The text carries its NUL: it is never refused for lacking one.
                let blank = edits::blank(&text).map(|(blank, _)| blank).unwrap_or(false);
                if blank {
                    edit.text = host
                        .first_folder()
                        .unwrap_or_else(|| host.string(edit.default_text));
                }
                return Ok(ended);
            }
            EditKey::Clear => {
                edit.text.clear();
                return plain(key);
            }
            EditKey::Backspace => {
                edit.text.pop();
                return plain(key);
            }
            EditKey::Arrow => return plain(key),
            // The characters come as typed text, not as keys.
            EditKey::Other => return plain(key),
        }
    };
    if ch < 0x20 || !edits::takes(ch, edit.text.len(), edit.most) {
        return plain(key);
    }
    edit.text.push(ch);
    let width = host.text_width(&edit.text)?;
    if edits::too_wide(width, edit.width) {
        edit.text.pop();
    }
    plain(u32::from(edit.width))
}

/// A page as `pieces::lay_out` reads a list.
struct Source<'a, F> {
    page: &'a PageDef,
    text_font: F,
}

impl<F: Fn(u8) -> u8> ListSource for Source<'_, F> {
    fn flags(&self) -> u8 {
        // The installer's lists have no flags: the title is shown, a page leaves on Escape.
        0
    }

    fn count(&self) -> u8 {
        self.page.items.len() as u8
    }

    fn title_text(&self) -> u16 {
        self.page.title
    }

    fn item(&self, k: u32) -> pieces::Item {
        let item = &self.page.items[k as usize];
        pieces::Item {
            first: item.first(),
            count: item.count_byte(),
            text: item.text,
        }
    }

    fn text_font(&self, font: u8) -> u8 {
        (self.text_font)(font)
    }
}

impl PageDef {
    /// The pieces the page is drawn from (`pieces::lay_out`); `text_font` gives the font table's
    /// byte of a text item's font number.
    pub fn lay_out(
        &self,
        text_font: impl Fn(u8) -> u8,
    ) -> Result<Vec<pieces::NewPiece>, pieces::LayoutError> {
        pieces::lay_out(&Source {
            page: self,
            text_font,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> Facts {
        Facts {
            same_version: true,
            installed: true,
            update_entry: None,
            languages: 3,
            drives: 3,
            path_problem: 0,
            folder_most: 0x7C,
            folder_width: 0x2E9,
        }
    }

    /// A host with a fixed width a character, a default text and no first folder.
    struct Host {
        first: Option<Vec<u8>>,
    }

    impl EditHost for Host {
        type Error = ();
        fn text_width(&mut self, text: &[u8]) -> Result<u32, ()> {
            Ok(text.len() as u32 * 100)
        }
        fn string(&mut self, id: u16) -> Vec<u8> {
            format!("<{id:x}>").into_bytes()
        }
        fn first_folder(&mut self) -> Option<Vec<u8>> {
            self.first.clone()
        }
    }

    fn marks(page: &PageDef) -> Vec<u8> {
        page.items.iter().map(|item| item.first() & 0xC0).collect()
    }

    #[test]
    fn the_pages_hold_their_items() {
        let lists = [
            (List::Language, 10),
            (List::Main, 4),
            (List::Drives8, 5),
            (List::Folder, 6),
            (List::NoRoom, 3),
            (List::Removal, 4),
        ];
        for (list, count) in lists {
            assert_eq!(
                build(list, &facts()).unwrap().items.len(),
                count,
                "{list:?}"
            );
        }
        let page = build(
            List::Drives16,
            &Facts {
                drives: 16,
                ..facts()
            },
        )
        .unwrap();
        assert_eq!(page.items.len(), 18);
        assert_eq!(page.items[9].box_id, Some(5));
        assert_eq!(page.items[10].box_id, Some(6));
        let page = build(
            List::DrivesMore,
            &Facts {
                drives: 24,
                ..facts()
            },
        )
        .unwrap();
        assert_eq!(page.items.len(), 26);
        assert_eq!(page.items[25].box_id, Some(4));
        assert!(
            build(
                List::Drives8,
                &Facts {
                    drives: 9,
                    ..facts()
                }
            )
            .is_err()
        );
        assert!(
            build(
                List::DrivesMore,
                &Facts {
                    drives: 25,
                    ..facts()
                }
            )
            .is_err()
        );
        assert_eq!(box_area(None).1, true);
        assert_eq!(box_area(Some(0)).1, false);
    }

    #[test]
    fn the_facts_mark_the_items() {
        let language = build(List::Language, &facts()).unwrap();
        assert_eq!(
            marks(&language),
            [0, 0, 0, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0]
        );
        assert!(
            build(
                List::Language,
                &Facts {
                    languages: 10,
                    ..facts()
                }
            )
            .is_err()
        );
        let main = build(
            List::Main,
            &Facts {
                same_version: false,
                installed: false,
                ..facts()
            },
        )
        .unwrap();
        assert_eq!(marks(&main), [0, 0x80, 0x80, 0]);
        let update = build(
            List::Main,
            &Facts {
                same_version: false,
                update_entry: Some(0x1006),
                ..facts()
            },
        )
        .unwrap();
        assert_eq!(marks(&update), [0, 0, 0x40, 0]);
        assert_eq!(update.items[0].first(), 5);
        assert_eq!(update.items[1].text, 0xBD);
        let folder = build(
            List::Folder,
            &Facts {
                path_problem: 2,
                ..facts()
            },
        )
        .unwrap();
        assert_eq!(marks(&folder), [0, 0, 0x40, 0x40, 0, 0]);
    }

    #[test]
    fn the_highlight_is_lit_unless_marked() {
        let mut page = build(List::Removal, &facts()).unwrap();
        page.mark_highlight(2, 8);
        assert_eq!(page.items[2].marks, Marks::NONE);
        page.mark_highlight(2, 0);
        assert_eq!(page.items[2].marks, Marks::LIT);
        page.mark_highlight(2, 0);
        assert_eq!(page.items[2].first(), 0xC1);
        assert!(page.items[0].unusable());
        assert!(!page.items[1].unusable());
    }

    #[test]
    fn a_key_on_a_link() {
        let mut page = build(List::Removal, &facts()).unwrap();
        let mut host = Host { first: None };
        let enter = page.key(2, 0x1C, &mut host).unwrap();
        assert_eq!(
            (enter.taken, enter.leave, enter.noted),
            (false, Some(2), false)
        );
        let back = page.key(3, 0x1C, &mut host).unwrap();
        assert_eq!((back.taken, back.leave, back.noted), (true, Some(0), true));
        let escape = page.key(1, 1, &mut host).unwrap();
        assert_eq!(
            (escape.taken, escape.leave, escape.noted),
            (true, Some(0), false)
        );
        for key in [0x66, 0x6B, 0x68, 0x69, 2, 0xA] {
            let keyed = page.key(1, key, &mut host).unwrap();
            assert_eq!(
                (keyed.taken, keyed.leave, keyed.other),
                (false, None, false)
            );
        }
        let other = page.key(1, 0x52, &mut host).unwrap();
        assert!(other.other && !other.taken);
        let room = build(List::NoRoom, &facts()).unwrap();
        let mut room = room;
        let anyway = room.key(1, 0x1C, &mut host).unwrap();
        assert!(anyway.ignore_room && anyway.taken && anyway.leave == Some(1));
    }

    #[test]
    fn an_edit_is_started_typed_into_and_ended() {
        let mut page = build(List::Folder, &facts()).unwrap();
        assert_eq!(page.next_edit(0), (1, true));
        assert_eq!(page.next_edit(2), (6, false));
        let mut host = Host { first: None };
        let Kind::Edit(edit) = &mut page.items[1].kind else {
            panic!()
        };
        edit.text = b"abc".to_vec();
        page.start_edit(1);
        let typed = page.key(1, 0x100 | u32::from(b'd'), &mut host).unwrap();
        assert_eq!(
            (typed.key_word, typed.plain_edit, typed.noted),
            (0x2E9, true, true)
        );
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!(edit.text, b"abcd");
        // A key that is only an arrow, backspace, clear.
        page.key(1, 0xE, &mut host).unwrap();
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!(edit.text, b"abc");
        // Escape puts the text back and ends the edit.
        page.key(1, 0x100 | u32::from(b'x'), &mut host).unwrap();
        let escape = page.key(1, 1, &mut host).unwrap();
        assert_eq!(
            (escape.key_word, escape.plain_edit, escape.taken),
            (0x801, false, false)
        );
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!((edit.text.as_slice(), edit.editing), (&b"abc"[..], false));
        // The folder page leaves on the key the edit ended with.
        assert_eq!(own_key(Page::Folder, 0x801), (true, Some(0)));
        assert_eq!(own_key(Page::Folder, 0x81C), (true, Some(1)));
        assert_eq!(own_key(Page::Folder, 0x830), (false, None));
        assert_eq!(own_key(Page::Folder, 0), (true, None));
        assert_eq!(own_key(Page::Main, 0), (false, None));
        // Enter on a blank text puts in the default text, or the first folder.
        page.start_edit(1);
        let Kind::Edit(edit) = &mut page.items[1].kind else {
            panic!()
        };
        edit.text = b"  ".to_vec();
        page.key(1, 0x1C, &mut host).unwrap();
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!(edit.text, b"<a0>");
        page.start_edit(1);
        let mut with_first = Host {
            first: Some(b"/home".to_vec()),
        };
        let Kind::Edit(edit) = &mut page.items[1].kind else {
            panic!()
        };
        edit.text.clear();
        page.key(1, 0x1C, &mut with_first).unwrap();
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!(edit.text, b"/home");
    }

    #[test]
    fn a_text_too_wide_loses_its_last_character() {
        let mut page = build(
            List::Folder,
            &Facts {
                folder_width: 250,
                ..facts()
            },
        )
        .unwrap();
        let mut host = Host { first: None };
        page.start_edit(1);
        page.key(1, 0x100 | u32::from(b'a'), &mut host).unwrap();
        page.key(1, 0x100 | u32::from(b'b'), &mut host).unwrap();
        page.key(1, 0x100 | u32::from(b'c'), &mut host).unwrap();
        let Kind::Edit(edit) = &page.items[1].kind else {
            panic!()
        };
        assert_eq!(edit.text, b"ab");
    }

    #[test]
    fn the_pieces_come_from_the_items() {
        let page = build(List::NoRoom, &facts()).unwrap();
        let laid = page.lay_out(|font| font).unwrap();
        let owners: Vec<_> = laid.iter().map(|piece| piece.owner).collect();
        assert_eq!(
            owners,
            [Owner::Title, Owner::Item(0), Owner::Item(1), Owner::Item(2)]
        );
        assert_eq!(page.owner_item(Owner::Title).box_id, Some(0));
    }
}
