//! Characterisation of the installer pages' readers: `PageLink`, `PageEntry`, `ListMenu`,
//! `ListItem`, `Choices`, `BoxRecord` and `Piece`, over the pages as they start. Every
//! reader is asked for every index it can be given, run-ons included (an item past a list's
//! items, a choice past a table, an entry's word past its answers, a piece past the layout),
//! and what it returns, or that it refuses, is compared with `page_readers.golden`. The
//! golden holds the readable lines (the real items, the edges of each run-on) and, per
//! sweep, a count and a hash of every line.
//!
//! `RC2K_BLESS=1 cargo test page_readers` writes the golden again.

use crate::install::page_links::{ListId, PageLink};
use crate::install::page_records::{BoxRecord, PageEntry, Piece};
use crate::menu::list_handle::{ListItem, ListMenu};
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

fn item_line(item: ListItem) -> String {
    let choices = |item: ListItem| {
        let c = item.choices();
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
        probe(|| crate::install::installer::rb(item.text())),
        probe(|| item.value().get()),
        probe(|| choices(item)),
        probe(|| {
            let e = item.edit_field();
            (
                e.flags(),
                e.most(),
                e.default_text(),
                e.font_kind(),
                e.width(),
            )
        }),
        probe(|| item.pages_offset()),
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
    format!(
        "has_list={} list={}",
        probe(|| entry.has_list()),
        probe(|| entry.list()),
    )
}

fn pages_size() -> usize {
    size_of::<crate::install::vars::InstallerPages>()
}

/// Every probe, as the golden's lines.
fn characterise() -> Vec<String> {
    quiet();
    let install = crate::install::owner::install();
    let mut out: Vec<String> = Vec::new();
    let size = pages_size();

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
                probe(|| link
                    .pages_at()
                    .map(crate::install::pages_map::PagesAt::offset)),
                probe(|| link.place().map(|p| p.read(1, "link").unwrap())),
                probe(|| link.font()),
                probe(|| link.edit_record().is_ok()),
                probe(|| link.list_id()),
                probe(|| link
                    .box_position()
                    .map(crate::install::pages_map::PagesAt::offset)),
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
            probe(|| link.place().map(|p| p.read(1, "typed").unwrap()))
        );
        links.add(&line);
    }
    // Every number of the table kind, list kind and box kind.
    for kind in [1u32, 5, 6, 8] {
        for number in 0..80u32 {
            let link = PageLink::from_raw(kind << 24 | number);
            links.add(&format!(
                "{kind}/{number}: {} {}",
                probe(|| link
                    .pages_at()
                    .map(crate::install::pages_map::PagesAt::offset)),
                probe(|| link.in_pages().map(|p| p.read(1, "link").unwrap())),
            ));
        }
    }
    out.push(links.line());

    // Words of the pages at every byte, in and past the end.
    let mut words = Sweep::new("pages_dword");
    for at in 0..size + 8 {
        words.add(&format!(
            "{at}: {}",
            probe(|| crate::install::pages_map::PagesAt::at_offset(at).dword())
        ));
    }
    for at in [usize::MAX, usize::MAX - 3, size, size - 3, size - 4] {
        out.push(format!(
            "pages_dword({at}) = {}",
            probe(|| crate::install::pages_map::PagesAt::at_offset(at).dword())
        ));
    }
    out.push(words.line());

    // The page entries: both runs, every table entry, any thing of the pages as an entry.
    let mut entries = Sweep::new("entries");
    let mut starts: Vec<(String, PageLink)> = (0..55)
        .map(|i| (format!("table {i}"), PageLink::table(i)))
        .collect();
    starts.extend(ListId::ALL.map(|l| (format!("list {l:?}"), PageLink::list(l))));
    starts.extend((0..9).map(|i| (format!("box {i}"), PageLink::box_at(i))));
    starts.push(("edit".into(), PageLink::folder_edit()));
    for (name, link) in &starts {
        let entry = PageEntry::at(*link);
        let head = format!("{name}: {} entry={entry:?}", entry_line(entry));
        out.push(head.clone());
        entries.add(&head);
        for offset in 0..0xC0u32 {
            let next = probe(|| entry.next(offset));
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
            let next = probe(|| entry.next(offset));
            out.push(format!("  {name} next({offset:#x}) = {next}"));
            entries.add(&format!("{name} {offset}: {next}"));
        }
    }
    for (name, entry) in [
        ("normal", PageEntry::normal_run()),
        ("update", PageEntry::update_run()),
    ] {
        out.push(format!("{name} run: {entry:?} {}", entry_line(entry)));
    }
    out.push(entries.line());

    // The lists: the header, the real items and the run-on past them, every index.
    let limit = (size / 16 + 4) as u32;
    for list_id in ListId::ALL {
        let list = ListMenu::installer(list_id);
        let count = u32::from(list.count());
        out.push(format!(
            "list {list_id:?}: {:?} {}",
            list,
            header_line(list)
        ));
        let mut sweep = Sweep::new(format!("items of {list_id:?}"));
        let mut reach = None;
        for n in 0..limit {
            let line = item_line(list.item(n));
            if n < count + 3 || n + 1 == limit {
                out.push(format!("  {list_id:?} item {n}: {:?} {line}", list.item(n)));
            }
            if reach.is_none() && line.contains("!") && n >= count {
                reach = Some(n);
            }
            sweep.add(&format!("{n}: {:?} {line}", list.item(n)));
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
            let line = format!(
                "{:?} {}",
                probe(|| list.item(n)),
                probe(|| item_line(list.item(n)))
            );
            out.push(format!("  {list_id:?} item {n:#x}: {line}"));
            sweep.add(&format!("{n:#x}: {line}"));
        }
        out.push(format!(
            "  {list_id:?} first item with a refusing field past the count: {reach:?}"
        ));
        // The items by their offset from the list (the pages' tables of marks).
        for k in 0..40u32 {
            let line = probe(|| item_line(list.item_at_offset(0x18 + 0x10 * k)));
            sweep.add(&format!("at_offset {k}: {line}"));
        }
        for offset in [0x19u32, 0x10, 0, 0x17, 0x28 + 1] {
            let line = probe(|| list.item_at_offset(offset));
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
                let c = item.choices();
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
        let list = ListMenu::at(*link).unwrap();
        let line = format!("{name}: {:?} {}", list, header_line(list));
        out.push(line.clone());
        walk.add(&line);
        for n in 0..6 {
            walk.add(&format!(
                "{name} item {n}: {:?} {}",
                list.item(n),
                item_line(list.item(n))
            ));
        }
    }
    out.push(walk.line());

    // The items by their offset in the pages (a piece keeps this).
    let mut items = Sweep::new("items in pages");
    for offset in 0..=(size as u32 + 16) {
        let line = match ListItem::in_pages(offset) {
            Ok(item) => format!("{offset}: {item:?} {}", item_line(item)),
            Err(_) => format!("{offset}: err"),
        };
        items.add(&line);
    }
    for offset in [
        0u32,
        0x10,
        0x18,
        size as u32 - 16,
        size as u32 - 15,
        u32::MAX,
    ] {
        let line = match ListItem::in_pages(offset) {
            Ok(item) => format!("{item:?} {}", item_line(item)),
            Err(_) => "err".into(),
        };
        out.push(format!("in_pages({offset:#x}) = {line}"));
    }
    out.push(items.line());

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
            let b = BoxRecord::linked(link);
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
            probe(|| piece.item()),
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

    // Choice items: none of the starting data is one, so the room list's first item is made
    // one for a while, its table named by each kind of link (and put back).
    let item = &install.pages.room_menu.items[0];
    let saved = (item.first.get(), item.data.get());
    item.first.set(2);
    let tables = [
        PageLink::table(0),
        PageLink::table(54),
        PageLink::list(ListId::Main),
        PageLink::list(ListId::Finish),
        PageLink::list(ListId::AskRegistration),
        PageLink::box_at(8),
        PageLink::folder_edit(),
        PageLink::of_font(crate::gfx::typefaces::fonts::TITLE),
        PageLink::from_raw(0),
        PageLink::from_raw(5 << 24 | 99),
    ];
    for link in tables {
        item.data.set(link.raw());
        let list = ListMenu::installer(ListId::Room);
        let it = list.item(0);
        let mut sweep = Sweep::new(format!("choices at {:#x}", link.raw()));
        out.push(format!("choice item {:#x}: {}", link.raw(), item_line(it)));
        for n in (0..0x2000u32).chain([
            0x7FFF_FFFF,
            0x8000_0000,
            0x8000_0001,
            0xFFFF_FFFF,
            0xFFFF_FFFE,
            0x4000_0000,
            0x1000_0000,
        ]) {
            let line = probe(|| {
                let c = it.choices();
                (c.word(n), c.marks(n))
            });
            if n < 4 || n == 0x1FFF || n > 0x2000 {
                out.push(format!("  choice {n:#x} = {line}"));
            }
            sweep.add(&format!("{n:#x}: {line}"));
        }
        // The table's first bytes, as a count the item gives.
        for len in [0usize, 1, 8, 100, 6704, 6705] {
            let line = probe(|| {
                it.choices()
                    .bytes(len, "test")
                    .map(|b| b.iter().map(|&x| u64::from(x)).sum::<u64>())
            });
            sweep.add(&format!("bytes {len}: {line}"));
        }
        out.push(sweep.line());
    }
    item.first.set(saved.0);
    item.data.set(saved.1);
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
