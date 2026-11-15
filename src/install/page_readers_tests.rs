//! Characterisation of the installer pages' readers (`PageLink`, `PageEntry`, `ListMenu`,
//! `ListItem`, `Choices`, `BoxRecord`, `Piece`) over the pages as they start: every reader is
//! asked for every index it can be given, run-ons included, and what it returns, or that it
//! refuses, is compared with `page_readers.golden` (the readable lines, and per sweep a count
//! and a hash of every line). `RC2K_BLESS=1 cargo test page_readers` writes the golden again.
//!
//! The golden still names things by the byte offsets they had when the installer's pages were
//! read as one run of bytes (`legacy`), so its lines for everything inside the data read as
//! they always did. What the typed pages no longer read, a run on past a list's items, a table
//! or a record into the bytes that follow, is refused and shows as `!`.

use crate::install::page_links::{ListId, PageLink};
use crate::install::page_records::{BoxRecord, PageEntry, Piece};
use crate::list_handle::{ItemPosition, ListItem, ListMenu};
use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};

const GOLDEN: &str = include_str!("page_readers.golden");

/// A refusal is a panic in a test; its message is not worth the noise here.
fn quiet() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let text = info.to_string();
            if !text.contains(": refused:") && !text.contains("refused") {
                previous(info);
            }
        }));
    });
}

/// What `f` returns, or `!` when it refuses.
fn probe<T: Debug>(f: impl FnOnce() -> T) -> String {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => format!("{v:?}"),
        Err(_) => "!".into(),
    }
}

/// The lines of one sweep: a count and a hash, and the lines kept readable.
struct Sweep {
    name: String,
    count: usize,
    hash: u64,
    refused: usize,
}

impl Sweep {
    fn new(name: impl Into<String>) -> Self {
        Sweep {
            name: name.into(),
            count: 0,
            hash: 0xcbf29ce484222325,
            refused: 0,
        }
    }

    fn add(&mut self, line: &str) {
        self.count += 1;
        if line.contains('!') {
            self.refused += 1;
        }
        for b in line.bytes().chain(std::iter::once(b'\n')) {
            self.hash = (self.hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }

    fn line(&self) -> String {
        format!(
            "sweep {}: {} probes, {} with a refusal, hash {:016x}",
            self.name, self.count, self.refused, self.hash
        )
    }
}

/// The offsets the installer's pages' records had in the one run of bytes they were read as:
/// the names the golden still uses.
mod legacy {
    use super::*;

    /// The run's length.
    pub const SIZE: usize = 0x1A30;
    const TABLE: usize = 0x1194;
    const ACTION_PAGES: usize = 0x1910;
    const EDIT: usize = 0x1970;
    const TITLE: usize = 0x1978;
    const BOXES: usize = 0x1988;

    pub fn entry(index: u32) -> usize {
        TABLE + 4 * index as usize
    }

    pub fn list(list: ListId) -> usize {
        match list {
            ListId::Language => 0x1270,
            ListId::Main => 0x1328,
            ListId::Mode => 0x1390,
            ListId::Drives8 => 0x13F8,
            ListId::Drives16 => 0x14B0,
            ListId::DrivesMore => 0x15E8,
            ListId::Folder => 0x17A0,
            ListId::Room => 0x1818,
            ListId::Dxsetup => 0x1860,
            ListId::Finish => 0x18B8,
            ListId::InstallFiles => ACTION_PAGES,
            ListId::Uninstall => ACTION_PAGES + 0x18,
            ListId::OpenInstalled => ACTION_PAGES + 0x30,
            ListId::AskRegistration => ACTION_PAGES + 0x48,
        }
    }

    pub fn item(position: ItemPosition) -> usize {
        match position {
            ItemPosition::Title => TITLE,
            ItemPosition::In(l, n) => list(l) + 0x18 + 0x10 * n as usize,
        }
    }

    pub fn edit() -> usize {
        EDIT
    }

    /// The offset of the box `rect` is, if it is one of the pages' boxes.
    pub fn box_of(rect: &crate::list_handle::BoxRect) -> usize {
        let boxes = &crate::test_support::own::install().pages.boxes;
        BOXES + 16 * boxes.iter().position(|b| core::ptr::eq(b, rect)).unwrap()
    }
}

/// A text printed as it is, for what the golden names by a legacy offset.
struct Named(String);

impl Debug for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn list_name(list: ListMenu) -> Named {
    let install = crate::test_support::own::install();
    let id = ListId::ALL
        .into_iter()
        .find(|&id| ListMenu::installer(install, id) == list)
        .unwrap();
    Named(format!("ListMenu(pages +{:#x})", legacy::list(id)))
}

fn item_name(item: ListItem) -> Named {
    let install = crate::test_support::own::install();
    Named(format!(
        "ListItem(pages +{:#x})",
        legacy::item(item.installer_position(install).unwrap())
    ))
}

fn entry_name(entry: PageEntry) -> Named {
    Named(format!("PageEntry({})", legacy::entry(entry.index())))
}

/// The line of item `n` of `list`, or `!` for an item the list does not hold.
fn item_at(list: ListMenu, n: u32) -> (Named, String) {
    match catch_unwind(AssertUnwindSafe(|| list.item(n))) {
        Ok(item) => (item_name(item), item_line(item)),
        Err(_) => (Named("!".into()), "!".into()),
    }
}

/// The legacy offset of the box a link names.
fn box_offset(link: PageLink) -> Result<usize, crate::errors::BadInput> {
    let install = crate::test_support::own::install();
    link.box_rect(install).map(legacy::box_of)
}

/// The legacy offset of the thing a link names (a table entry, a list, a box, the edit record).
fn link_offset(link: PageLink) -> Option<usize> {
    let install = crate::test_support::own::install();
    if let Some(index) = link.table_index() {
        return Some(legacy::entry(index));
    }
    if let Some(list) = link.list_id() {
        return Some(legacy::list(list));
    }
    if let Ok(offset) = box_offset(link) {
        return Some(offset);
    }
    link.edit_record(install).ok().map(|_| legacy::edit())
}

fn item_line(item: ListItem) -> String {
    let install = crate::test_support::own::install();
    let choices = |item: ListItem| {
        let c = item.choices(install);
        (0..3).map(|n| (c.word(n), c.marks(n))).collect::<Vec<_>>()
    };
    format!(
        "first={} count={} text={} box={} target={} data={} least={} most={} \
         text_byte={} value={} choices={} edit={} pages_offset={}",
        probe(|| item.first()),
        probe(|| item.count()),
        probe(|| item.text_id()),
        probe(|| item.box_link().raw()),
        probe(|| item.target()),
        probe(|| item.data()),
        probe(|| item.value_least()),
        probe(|| item.value_most()),
        probe(|| crate::install::installer::rb(item.text(install))),
        probe(|| item.value(install).get()),
        probe(|| choices(item)),
        probe(|| {
            let e = item.edit_field(install);
            (
                e.flags(),
                e.most(),
                e.default_text(),
                e.font_kind(),
                e.width(),
            )
        }),
        probe(|| item
            .to_handle(install)
            .map(|_| legacy::item(item.installer_position(install).unwrap()))),
    )
}

fn header_line(list: ListMenu) -> String {
    format!(
        "flags={} count={} title={} highlight={} action={} key={} neighbours={}",
        probe(|| list.flags()),
        probe(|| list.count()),
        probe(|| list.title_text()),
        probe(|| list.highlight()),
        probe(|| list.action()),
        probe(|| list.key_action()),
        probe(|| list.neighbours().map(|n| n.read(4, "neighbours").unwrap())),
    )
}

fn entry_line(entry: PageEntry) -> String {
    let install = crate::test_support::own::install();
    format!(
        "has_list={} list={}",
        probe(|| entry.has_list(install)),
        probe(|| list_name(entry.list(install))),
    )
}

/// Every probe, as the golden's lines.
fn characterise() -> Vec<String> {
    quiet();
    let install = crate::test_support::own::install();
    let mut out: Vec<String> = Vec::new();
    let size = legacy::SIZE;

    // The links: every kind, with numbers at and past the edges.
    let mut links = Sweep::new("links");
    for kind in 0..10u32 {
        for number in [
            0, 1, 2, 8, 9, 13, 14, 15, 54, 55, 56, 0xFF, 0x100, 0xFF_FFFF,
        ] {
            let link = PageLink::from_raw(kind << 24 | number);
            let line = format!(
                "link {:#x}: offset={} place_byte={} font={} edit={} list={} box={} blended={}",
                link.raw(),
                probe(|| link_offset(link)),
                probe(|| link.place(install).map(|p| p.read(1, "link").unwrap())),
                probe(|| link.font()),
                probe(|| link.edit_record(install).is_ok()),
                probe(|| link.list_id()),
                probe(|| box_offset(link)),
                link.is_blended_palette(),
            );
            if number <= 2 || number == 55 || number == 56 {
                out.push(line.clone());
            }
            links.add(&line);
        }
    }
    // The folder typed: every offset the type allows, and past it.
    for number in 0..0x104u32 {
        let link = PageLink::from_raw(4 << 24 | number);
        let line = format!(
            "typed {number}: {}",
            probe(|| link.place(install).map(|p| p.read(1, "typed").unwrap()))
        );
        links.add(&line);
    }
    // Every number of the table kind, list kind and box kind.
    for kind in [1u32, 5, 6, 8] {
        for number in 0..80u32 {
            let link = PageLink::from_raw(kind << 24 | number);
            links.add(&format!(
                "{kind}/{number}: {} {}",
                probe(|| link_offset(link)),
                probe(|| link.place(install).map(|p| p.read(1, "link").unwrap())),
            ));
        }
    }
    out.push(links.line());

    // The page entries: both runs, every table entry, any thing of the pages as an entry.
    let mut entries = Sweep::new("entries");
    let mut starts: Vec<(String, PageLink)> = (0..55)
        .map(|i| (format!("table {i}"), PageLink::table(i)))
        .collect();
    starts.extend(ListId::ALL.map(|l| (format!("list {l:?}"), PageLink::list(l))));
    starts.extend((0..9).map(|i| (format!("box {i}"), PageLink::box_at(i))));
    starts.push(("edit".into(), PageLink::folder_edit()));
    for (name, link) in &starts {
        // Only a table entry is a page entry; a thing of another kind is refused.
        let Ok(entry) = catch_unwind(|| PageEntry::at(*link)) else {
            let head = format!("{name}: entry refused");
            out.push(head.clone());
            entries.add(&head);
            continue;
        };
        let head = format!(
            "{name}: {} entry={:?}",
            entry_line(entry),
            entry_name(entry)
        );
        out.push(head.clone());
        entries.add(&head);
        for offset in 0..0xC0u32 {
            let next = probe(|| entry.next(install, offset).map(entry_name));
            if offset % 4 == 0 && offset < 0x20 {
                out.push(format!("  {name} next({offset:#x}) = {next}"));
            }
            entries.add(&format!("{name} {offset}: {next}"));
        }
        for offset in [
            0x1000u32,
            0x1B00,
            0x1C00,
            u32::MAX - 3,
            u32::MAX,
            0x8000_0000,
        ] {
            let next = probe(|| entry.next(install, offset).map(entry_name));
            out.push(format!("  {name} next({offset:#x}) = {next}"));
            entries.add(&format!("{name} {offset}: {next}"));
        }
    }
    for (name, entry) in [
        ("normal", PageEntry::normal_run()),
        ("update", PageEntry::update_run()),
    ] {
        out.push(format!(
            "{name} run: {:?} {}",
            entry_name(entry),
            entry_line(entry)
        ));
    }
    out.push(entries.line());

    // The lists: the header, the real items and the run-on past them, every index.
    let limit = (size / 16 + 4) as u32;
    for list_id in ListId::ALL {
        let list = ListMenu::installer(install, list_id);
        let count = u32::from(list.count());
        out.push(format!(
            "list {list_id:?}: {:?} {}",
            list_name(list),
            header_line(list)
        ));
        let mut sweep = Sweep::new(format!("items of {list_id:?}"));
        let mut reach = None;
        for n in 0..limit {
            let (name, line) = item_at(list, n);
            if n < count + 3 || n + 1 == limit {
                out.push(format!("  {list_id:?} item {n}: {name:?} {line}"));
            }
            if reach.is_none() && line.contains("!") && n >= count {
                reach = Some(n);
            }
            sweep.add(&format!("{n}: {name:?} {line}"));
        }
        for n in [
            u32::MAX,
            u32::MAX - 1,
            0x1000_0000,
            0x1000_0001,
            0x2000_0000,
            0x1000_0000 - 1,
            0x0800_0000,
        ] {
            let (name, line) = item_at(list, n);
            let line = format!("{name:?} {line}");
            out.push(format!("  {list_id:?} item {n:#x}: {line}"));
            sweep.add(&format!("{n:#x}: {line}"));
        }
        out.push(format!(
            "  {list_id:?} first item with a refusing field past the count: {reach:?}"
        ));
        // The items by their offset from the list (the pages' tables of marks).
        for k in 0..40u32 {
            let line = catch_unwind(AssertUnwindSafe(|| {
                item_line(list.item_at_offset(0x18 + 0x10 * k))
            }))
            .unwrap_or_else(|_| "!".into());
            sweep.add(&format!("at_offset {k}: {line}"));
        }
        for offset in [0x19u32, 0x10, 0, 0x17, 0x28 + 1] {
            let line = probe(|| item_name(list.item_at_offset(offset)));
            out.push(format!(
                "  {list_id:?} item_at_offset({offset:#x}) = {line}"
            ));
        }
        // A choices table walked far past its length.
        let mut choices = Sweep::new(format!("choices of {list_id:?}"));
        for k in 0..count.min(40) {
            let item = list.item(k);
            if item.kind() != 2 {
                continue;
            }
            for n in (0..0x2000u32).chain([
                0x7FFF_FFFF,
                0x8000_0000,
                0x8000_0001,
                0xFFFF_FFFF,
                0xFFFF_FFFE,
                0x4000_0000,
            ]) {
                let c = item.choices(install);
                choices.add(&format!(
                    "{k}/{n:#x}: {} {}",
                    probe(|| c.word(n)),
                    probe(|| c.marks(n))
                ));
            }
        }
        out.push(sweep.line());
        out.push(choices.line());
    }

    // A list read from any thing of the pages (the page walk does so for a link it was given).
    let mut walk = Sweep::new("lists at links");
    for (name, link) in &starts {
        // Only a link to a list names one.
        let Ok(list) = ListMenu::at(install, *link) else {
            let line = format!("{name}: no list");
            out.push(line.clone());
            walk.add(&line);
            continue;
        };
        let line = format!("{name}: {:?} {}", list_name(list), header_line(list));
        out.push(line.clone());
        walk.add(&line);
        for n in 0..6 {
            let (item, line) = item_at(list, n);
            walk.add(&format!("{name} item {n}: {item:?} {line}"));
        }
    }
    out.push(walk.line());

    // The boxes, and the title's for none.
    for link in [PageLink::from_raw(0)]
        .into_iter()
        .chain((0..9).map(PageLink::box_at))
        .chain([
            PageLink::from_raw(6 << 24 | 9),
            PageLink::table(3),
            PageLink::folder_edit(),
        ])
    {
        let line = probe(|| {
            let b = BoxRecord::linked(install, link);
            (b.x(), b.y(), b.width(), b.height())
        });
        out.push(format!("box {:#x} = {line}", link.raw()));
    }

    // The layout's pieces: the count, and every piece the layout and the bytes past it give.
    out.push(format!(
        "piece count = {}",
        probe(|| crate::install::page_records::piece_count(install))
    ));
    let mut pieces = Sweep::new("pieces");
    for n in (0..100u32).chain([0xFFFF_FFFF, 0x1000_0000, 0x0555_5555]) {
        let piece = Piece::nth(install, n);
        let line = format!(
            "{n}: marks={} kind={} text={} item={} x={} y={} width={} height={}",
            probe(|| piece.marks()),
            probe(|| piece.kind()),
            probe(|| piece.text_id()),
            probe(|| item_name(piece.item(install))),
            probe(|| piece.x()),
            probe(|| piece.y()),
            probe(|| piece.width()),
            probe(|| piece.height()),
        );
        if n < 2 || (80..90).contains(&n) || n > 99 {
            out.push(format!("piece {line}"));
        }
        pieces.add(&line);
    }
    out.push(pieces.line());

    out
}

#[test]
fn page_readers_return_what_they_returned() {
    let _lock = crate::test_support::test_lock();
    let lines = characterise();
    let text = lines.join("\n") + "\n";
    if std::env::var_os("RC2K_BLESS").is_some() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/install/page_readers.golden"
        );
        std::fs::write(path, &text).unwrap();
        return;
    }
    for (n, (mine, golden)) in text.lines().zip(GOLDEN.lines()).enumerate() {
        assert_eq!(mine, golden, "line {} of page_readers.golden", n + 1);
    }
    assert_eq!(text.lines().count(), GOLDEN.lines().count());
}
