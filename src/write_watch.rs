//! The audit build's record of the game state's reads and writes (cargo feature `write_watch`;
//! `workers` uses it to find what two drawing parts both reach). Every
//! cell's load and store (`session`'s cells, `pointers`' cell runs, `shared_memory`) calls
//! `read` or `wrote`; without the feature they are empty and cost nothing. With it, while a
//! thread watches (`begin` .. `end`), each place reached is noted by its dword (a byte noted
//! as the dword that holds it: two neighbouring bytes count as one place, which can only add
//! to what is found).
//!
//! The thread census (`RC2K_FIELD_THREADS=<file>`, the watch build only; see `fields`)
//! needs no `begin`: it notes every access on every thread, by the thread's kind, and writes
//! the fields with the kinds of thread that read and wrote them to `<file>` at the end.
//!
//! The field-leaving census (`RC2K_FIELD_LEAVE=<file>`, the watch build only; see `leaves`)
//! notes every by-address read or write of a store whose bytes run past the end of the field
//! it starts in.
//!
//! The call-site census (`RC2K_BY_ADDRESS=<file>`, the watch build only; see `calls`) notes
//! every by-address access of a store, one row per call site, store and field.

/// `len` bytes at `at` read.
#[inline(always)]
pub fn read(_at: *const u8, _len: usize) {
    #[cfg(feature = "write_watch")]
    on::note(false, _at, _len);
}

/// `len` bytes at `at` written.
#[inline(always)]
pub fn wrote(_at: *const u8, _len: usize) {
    #[cfg(feature = "write_watch")]
    on::note(true, _at, _len);
}

#[cfg(feature = "write_watch")]
pub use on::{Seen, begin, end};

/// A step of the frame being measured (`RC2K_FRAME_CENSUS`, the watch build only): until the
/// guard is dropped, what the thread reads and writes is the step's. Steps nest (the sound
/// turn inside the drawing); each access is the innermost's.
#[must_use]
pub struct Step(());

/// `name`'s step begun on this thread (see `Step`).
#[inline(always)]
pub fn step(_name: &'static str) -> Step {
    #[cfg(feature = "write_watch")]
    census::begin(_name);
    Step(())
}

impl Drop for Step {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(feature = "write_watch")]
        census::end();
    }
}

/// A new frame of the race begun (the census's count of frames).
#[inline(always)]
pub fn frame_begun() {
    #[cfg(feature = "write_watch")]
    {
        census::frame_begun();
        fields::frame_begun();
    }
}

/// The thread census written now (`fields`): at the program's end, and every 1,200 frames
/// of the race. Nothing without the watch build and `RC2K_FIELD_THREADS`.
#[inline(always)]
pub fn dump_fields() {
    #[cfg(feature = "write_watch")]
    {
        fields::dump();
        leaves::dump();
        calls::dump();
    }
}

/// `len` bytes at offset `at` of `store` read or written by game address (`pointers`' loads
/// and stores): noted when they run past the end of the field they start in (`leaves`), and
/// every one by its call site (`calls`).
#[inline(always)]
#[cfg_attr(feature = "write_watch", track_caller)]
pub fn by_address(_store: &dyn crate::cells::Store, _at: usize, _len: usize, _write: bool) {
    #[cfg(feature = "write_watch")]
    {
        leaves::note(_store, _at, _len, _write);
        calls::note(_store, _at, _len, _write, std::panic::Location::caller());
    }
}

/// The frame census (`RC2K_FRAME_CENSUS`): for every step of the race's frame, what it reads
/// that another step (or itself, a frame before) wrote: the steps' real inputs and outputs.
/// Every 600 frames each edge "writer -> reader" is logged with the places it hands over, by
/// store and field path (`stores`).
#[cfg(feature = "write_watch")]
mod census {
    use std::cell::RefCell;
    use std::collections::{BTreeSet, HashMap};

    /// Who last wrote a dword: the step, the frame, the step's run.
    #[derive(Clone, Copy)]
    struct Writer {
        step: u16,
        frame: u32,
        run: u32,
    }

    #[derive(Default)]
    struct Census {
        names: Vec<&'static str>,
        /// The steps running (innermost last) and their runs.
        stack: Vec<(u16, u32)>,
        runs: u32,
        frame: u32,
        last: HashMap<usize, Writer>,
        /// (writer, written a frame before, reader) -> the dwords handed over. Writer
        /// `u16::MAX`: nothing in the race wrote it (loaded before).
        edges: HashMap<(u16, bool, u16), BTreeSet<usize>>,
    }

    thread_local! {
        static CENSUS: RefCell<Census> = RefCell::new(Census::default());
    }

    fn on() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| std::env::var_os("RC2K_FRAME_CENSUS").is_some())
    }

    pub fn begin(name: &'static str) {
        if !on() {
            return;
        }
        CENSUS.with_borrow_mut(|c| {
            let step = match c.names.iter().position(|&n| n == name) {
                Some(k) => k as u16,
                None => {
                    c.names.push(name);
                    (c.names.len() - 1) as u16
                }
            };
            c.runs += 1;
            let run = c.runs;
            c.stack.push((step, run));
        });
    }

    pub fn end() {
        if !on() {
            return;
        }
        CENSUS.with_borrow_mut(|c| {
            c.stack.pop();
        });
    }

    pub fn frame_begun() {
        if !on() {
            return;
        }
        let report = CENSUS.with_borrow_mut(|c| {
            c.frame += 1;
            c.frame.is_multiple_of(600)
        });
        if report {
            log_edges();
        }
    }

    /// Whether the census is counting on this thread (a step is running here).
    #[inline]
    pub fn active() -> bool {
        on() && CENSUS.with_borrow(|c| !c.stack.is_empty())
    }

    pub fn note(write: bool, at: *const u8, len: usize) {
        CENSUS.with_borrow_mut(|c| {
            let Some(&(step, run)) = c.stack.last() else {
                return;
            };
            let first = at.addr() >> 2;
            let last = (at.addr() + len - 1) >> 2;
            for w in first..=last {
                if write {
                    let frame = c.frame;
                    c.last.insert(w, Writer { step, frame, run });
                    continue;
                }
                let key = match c.last.get(&w) {
                    Some(wr) if wr.run == run => continue,
                    Some(wr) => (wr.step, wr.frame != c.frame, step),
                    None => (u16::MAX, false, step),
                };
                c.edges.entry(key).or_default().insert(w);
            }
        });
    }

    fn log_edges() {
        let lines = CENSUS.with_borrow(|c| {
            let name = |s: u16| {
                if s == u16::MAX {
                    "before"
                } else {
                    c.names[s as usize]
                }
            };
            let mut keys: Vec<_> = c.edges.keys().copied().collect();
            keys.sort();
            let mut lines = vec![format!("census: {} frames", c.frame)];
            for key in keys {
                let (from, before, to) = key;
                let words: Vec<usize> = c.edges[&key].iter().copied().collect();
                let mut text = format!(
                    "census: {} -> {}{}: {} dwords:",
                    name(from),
                    name(to),
                    if before { " (a frame before)" } else { "" },
                    words.len()
                );
                let mut k = 0;
                while k < words.len() {
                    let first = words[k];
                    while k + 1 < words.len() && words[k + 1] == words[k] + 1 {
                        k += 1;
                    }
                    let len = (words[k] - first + 1) * 4;
                    let host = first * 4;
                    if let Some((what, at)) = crate::memory::shared_table::table_of(host) {
                        text += &format!(" [{what}+{at:#x}+{len:#x}]");
                    } else if let Some(found) = crate::stores::find(host) {
                        text += &format!(" {}{}+{len:#x}", found.name, found.field());
                    } else {
                        text += &format!(" host:{host:#x}+{len:#x}");
                    }
                    k += 1;
                }
                lines.push(text);
            }
            lines
        });
        for line in lines {
            crate::framework::log(&line);
        }
    }
}

/// The thread census (`RC2K_FIELD_THREADS=<file>`; `1` writes `memory_fields.csv`): every
/// read and write of the game's state, on any thread, noted by the dword and the kind of
/// thread (main, audio, input, net receive, drawing pool, other: told by the thread's name,
/// `sync::spawn`'s). At the program's end (`standalone::finish`) and every 1,200 frames of
/// the race the file is written, one row per field (store, field path with indices dropped),
/// in the columns
/// `offset,store,field,dwords,read_by,written_by`, the threads joined by `+`. A dword that no
/// moved store holds is named by its shared table, or by its 4 KiB page.
#[cfg(feature = "write_watch")]
mod fields {
    use std::cell::RefCell;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::{Arc, Mutex, OnceLock};

    /// The kinds of thread, by the bit each has in a mask.
    const KINDS: [&str; 6] = ["main", "audio", "input", "net", "pool", "other"];

    /// What one thread did to each dword: 1 read it, 2 wrote it.
    type Seen = HashMap<usize, u8>;

    /// Every thread's record with the kind of thread (a thread's record outlives it).
    static THREADS: Mutex<Vec<(usize, Arc<Mutex<Seen>>)>> = Mutex::new(Vec::new());

    thread_local! {
        static MINE: RefCell<Option<(usize, Arc<Mutex<Seen>>)>> = const { RefCell::new(None) };
    }

    /// The file to write, if the census is on.
    fn file() -> Option<&'static std::path::Path> {
        static FILE: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
        FILE.get_or_init(|| {
            let value = std::env::var_os("RC2K_FIELD_THREADS")?;
            Some(if value == "1" {
                "memory_fields.csv".into()
            } else {
                value.into()
            })
        })
        .as_deref()
    }

    /// The kind of the running thread, from its name: the slot threads are named by their
    /// routine's address (0x4078DA input, 0x409CCE audio, 0x4183C3 net receive).
    pub(super) fn kind() -> usize {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("").to_ascii_lowercase();
        if name == "main" {
            0
        } else if name.contains("0x409cce") {
            1
        } else if name.contains("0x4078da") {
            2
        } else if name.contains("0x4183c3") {
            3
        } else if name.starts_with("draw ") {
            4
        } else {
            5
        }
    }

    pub fn note(write: bool, at: *const u8, len: usize) {
        if file().is_none() {
            return;
        }
        let first = at.addr() >> 2;
        let last = (at.addr() + len - 1) >> 2;
        MINE.with_borrow_mut(|mine| {
            let (_, seen) = mine.get_or_insert_with(|| {
                let mine = (kind(), Arc::new(Mutex::new(Seen::new())));
                THREADS
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((mine.0, mine.1.clone()));
                mine
            });
            let mut seen = seen
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for w in first..=last {
                *seen.entry(w).or_default() |= if write { 2 } else { 1 };
            }
        });
    }

    pub fn frame_begun() {
        static FRAMES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if n.is_multiple_of(1200) {
            dump();
            super::calls::dump();
        }
    }

    /// A field: where it is (its dword's offset in the store, or a host page), its store,
    /// and its path in the store.
    fn name_of(host: usize) -> (String, String, String) {
        if let Some(found) = crate::stores::find(host) {
            return (
                format!("{:#x}", found.offset),
                found.name.to_owned(),
                strip_indices(&found.field()),
            );
        }
        if let Some((what, at)) = crate::memory::shared_table::table_of(host) {
            return (format!("+{at:#x}"), what.to_owned(), String::new());
        }
        (
            format!("host:{:#x}", host & !0xFFF),
            "unmoved".to_owned(),
            String::new(),
        )
    }

    /// `.a[3].b` as `.a.b`.
    fn strip_indices(path: &str) -> String {
        let mut out = String::new();
        let mut inside = false;
        for c in path.chars() {
            match c {
                '[' => inside = true,
                ']' => inside = false,
                _ if !inside => out.push(c),
                _ => {}
            }
        }
        out
    }

    pub(super) fn kinds(mask: u8) -> String {
        let names: Vec<_> = (0..KINDS.len())
            .filter(|k| mask & (1 << k) != 0)
            .map(|k| KINDS[k])
            .collect();
        names.join("+")
    }

    /// A field's row: the offset of its lowest dword in the store, that dword's host address,
    /// the dwords seen, and the masks of the kinds that read and wrote.
    type Row = (String, usize, usize, u8, u8);

    /// The file written from what every thread has noted so far.
    pub fn dump() {
        let Some(path) = file() else {
            return;
        };
        // dword -> (kinds that read, kinds that wrote)
        let mut all: HashMap<usize, (u8, u8)> = HashMap::new();
        for (kind, seen) in THREADS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            for (&w, &how) in seen
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
            {
                let entry = all.entry(w).or_default();
                if how & 1 != 0 {
                    entry.0 |= 1 << kind;
                }
                if how & 2 != 0 {
                    entry.1 |= 1 << kind;
                }
            }
        }
        let mut rows: BTreeMap<(String, String), Row> = BTreeMap::new();
        for (w, (read, written)) in all {
            let (ida, store, field) = name_of(w * 4);
            let row = rows
                .entry((store, field))
                .or_insert((ida.clone(), w * 4, 0, 0, 0));
            if w * 4 < row.1 {
                row.0 = ida;
                row.1 = w * 4;
            }
            row.2 += 1;
            row.3 |= read;
            row.4 |= written;
        }
        let mut text = String::from("offset,store,field,dwords,read_by,written_by\n");
        for ((store, field), (ida, _, dwords, read, written)) in rows {
            text += &format!(
                "{ida},{store},{field},{dwords},{},{}\n",
                kinds(read),
                kinds(written)
            );
        }
        let _ = std::fs::write(path, text);
    }
}

/// The field-leaving census (`RC2K_FIELD_LEAVE=<file>`; `1` writes `field_leaves.csv`):
/// every by-address access (`pointers::load_at` / `store_at`, which the unbounded places,
/// the `*_or_end` reads and the places' reads all end in) whose bytes cover more than the
/// field the first byte is in, with the fields reached, the bytes past the first field's
/// end, the first Rust frame outside the access layers, and how often. Fields are the
/// stores' `Cells` fields with indices dropped (a run over the elements of one array is no
/// leave). Each distinct (store, offset, length, direction) is analysed once per thread, so
/// the run's cost stays that of the field census. File written as `fields`' is, columns
/// `module,site,store,start_field,reached,rw,len,bytes_past,count`.
#[cfg(feature = "write_watch")]
mod leaves {
    use crate::cells::Store;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::{Arc, Mutex, OnceLock};

    /// What one access shape did: how often, from where, and what it leaves (`None`:
    /// nothing).
    struct Shape {
        count: u64,
        site: String,
        leave: Option<Leave>,
    }

    #[derive(Clone)]
    struct Leave {
        store: String,
        start: String,
        reached: String,
        past: usize,
    }

    /// (store's first byte, offset, length, write).
    type Key = (usize, usize, usize, bool);
    type Shapes = HashMap<Key, (Shape, usize)>;

    static THREADS: Mutex<Vec<Arc<Mutex<Shapes>>>> = Mutex::new(Vec::new());

    thread_local! {
        static MINE: RefCell<Option<Arc<Mutex<Shapes>>>> = const { RefCell::new(None) };
    }

    fn file() -> Option<&'static std::path::Path> {
        static FILE: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
        FILE.get_or_init(|| {
            let value = std::env::var_os("RC2K_FIELD_LEAVE")?;
            Some(if value == "1" {
                "field_leaves.csv".into()
            } else {
                value.into()
            })
        })
        .as_deref()
    }

    /// `.a[3].b` as `.a.b`.
    pub(super) fn strip(path: &str) -> String {
        let mut out = String::new();
        let mut inside = false;
        for c in path.chars() {
            match c {
                '[' => inside = true,
                ']' => inside = false,
                _ if !inside => out.push(c),
                _ => {}
            }
        }
        out
    }

    fn field(store: &dyn Store, offset: usize) -> String {
        let mut path = String::new();
        store.field_path_at(offset, &mut path);
        strip(&path)
    }

    /// What `[at, at + len)` of the store leaves, or `None` when it stays in its first field.
    fn analyse(store: &dyn Store, at: usize, len: usize) -> Option<Leave> {
        let size = store.size();
        if at >= size {
            return None;
        }
        let end = (at + len).min(size);
        let start = field(store, at);
        let stride = (len / 64).max(1);
        let mut first_other = None;
        let mut reached: Vec<String> = Vec::new();
        let mut p = at;
        while p < end {
            let f = field(store, p);
            if f != start {
                first_other.get_or_insert(p);
                if !reached.contains(&f) {
                    reached.push(f);
                }
            }
            p += stride;
        }
        let last = field(store, end - 1);
        if last != start {
            first_other.get_or_insert(end - 1);
            if !reached.contains(&last) {
                reached.push(last);
            }
        }
        let other = first_other?;
        // Where the first field really ends: between the last probe that was in it and the
        // first that was not.
        let (mut lo, mut hi) = (other.saturating_sub(stride).max(at), other);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if field(store, mid) == start {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        if at + len > size {
            reached.push("<past the store>".into());
        }
        Some(Leave {
            store: store.type_name().to_owned(),
            start,
            reached: reached.join(";"),
            past: at + len - hi,
        })
    }

    /// The first frame of the backtrace outside the access layers (`pointers`, `session`,
    /// this file, the standard library), as `function @ src/path.rs:line`.
    pub(super) fn site() -> String {
        let text = std::backtrace::Backtrace::force_capture().to_string();
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            let Some((_, name)) = line.trim().split_once(": ") else {
                continue;
            };
            let Some(at) = lines.peek().and_then(|l| l.trim().strip_prefix("at ")) else {
                continue;
            };
            let layers = [
                "/rustc/",
                "/.cargo/",
                "src/pointers.rs",
                "src/pointers/",
                "src/write_watch.rs",
                "src/session.rs",
            ];
            if layers.iter().any(|l| at.contains(l)) {
                continue;
            }
            let file = at.rsplit_once("/src/").map_or(at, |(_, f)| f);
            return format!("{name} @ {file}");
        }
        "?".into()
    }

    pub fn note(store: &dyn Store, at: usize, len: usize, write: bool) {
        if len == 0 || file().is_none() {
            return;
        }
        let key = (store.first().addr(), at, len, write);
        let mine = MINE.with_borrow_mut(|m| {
            m.get_or_insert_with(|| {
                let mine = Arc::new(Mutex::new(Shapes::new()));
                THREADS
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(mine.clone());
                mine
            })
            .clone()
        });
        let mut shapes = mine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((shape, _)) = shapes.get_mut(&key) {
            shape.count += 1;
            return;
        }
        let leave = analyse(store, at, len);
        let site = site();
        shapes.insert(
            key,
            (
                Shape {
                    count: 1,
                    site,
                    leave,
                },
                len,
            ),
        );
    }

    /// The module of a site (`f @ race/frame_views.rs:12:5` -> `race`; a file directly in
    /// `src/` is its own module).
    pub(super) fn module_of(site: &str) -> &str {
        let file = site.rsplit_once(" @ ").map_or(site, |(_, f)| f);
        let first = file.split('/').next().unwrap_or(file);
        first.split_once(".rs").map_or(first, |(m, _)| m)
    }

    /// A row of the file: (site, store, start, reached, write, len).
    type RowKey = (String, String, String, String, bool, usize);

    pub fn dump() {
        let Some(path) = file() else {
            return;
        };
        // A row -> (bytes past, count).
        let mut rows: BTreeMap<RowKey, (usize, u64)> = BTreeMap::new();
        // site -> (shapes, accesses, shapes that leave)
        let mut sites: BTreeMap<String, (u64, u64, u64)> = BTreeMap::new();
        for mine in THREADS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            let shapes = mine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (&(_, _, _, write), (shape, len)) in shapes.iter() {
                let entry = sites.entry(shape.site.clone()).or_default();
                entry.0 += 1;
                entry.1 += shape.count;
                entry.2 += u64::from(shape.leave.is_some());
                let Some(l) = &shape.leave else {
                    continue;
                };
                let row = rows
                    .entry((
                        shape.site.clone(),
                        l.store.clone(),
                        l.start.clone(),
                        l.reached.clone(),
                        write,
                        *len,
                    ))
                    .or_default();
                row.0 = row.0.max(l.past);
                row.1 += shape.count;
            }
        }
        let mut text =
            String::from("module,site,store,start_field,reached,rw,len,bytes_past,count\n");
        for ((site, store, start, reached, write, len), (past, count)) in rows {
            text += &format!(
                "{},\"{site}\",{store},{start},\"{reached}\",{},{len},{past},{count}\n",
                module_of(&site),
                if write { "w" } else { "r" }
            );
        }
        let _ = std::fs::write(path, text);
        // Every site that reached a store by address, leaving or not.
        let mut text = String::from("module,site,shapes,accesses,leaving_shapes\n");
        for (site, (shapes, accesses, leaving)) in sites {
            text += &format!(
                "{},\"{site}\",{shapes},{accesses},{leaving}\n",
                module_of(&site)
            );
        }
        let _ = std::fs::write(path.with_extension("sites.csv"), text);
    }
}

/// The call-site census (`RC2K_BY_ADDRESS=<file>`; `1` writes `by_address.csv`): every
/// by-address read and write of a store (`pointers::load_at` / `store_at`), counted per call
/// site, store and field (indices dropped). The site is the first caller outside the access
/// layers that `#[track_caller]` reaches (`path:line`); where the chain breaks inside
/// `pointers` or `session` (a trait method, a helper) it is the first frame of a backtrace
/// outside them (`function @ path:line:col`), found once per place and direction, and
/// `via path:line` past 2,000 such lookups. Written at the program's end and every 1,200
/// frames of the race, in the columns
/// `module,site,store,field,reads,writes,read_bytes,write_bytes,threads`, the threads
/// (kinds as in `fields`) joined by `+`. A store the registry (`stores`) lacks is named by
/// its type.
#[cfg(feature = "write_watch")]
mod calls {
    use crate::cells::Store;
    use std::cell::RefCell;
    use std::collections::{BTreeMap, HashMap};
    use std::panic::Location;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Arc, Mutex, OnceLock};

    /// What one (store, offset, direction, call site) did.
    struct Row {
        store: String,
        field: String,
        site: String,
        write: bool,
        count: u64,
        bytes: u64,
    }

    /// (the store's first byte, offset, write, the location's file, its line).
    type Key = (usize, usize, bool, usize, u32);

    /// One thread's rows and its kind (`fields::kind`).
    struct Mine {
        kind: usize,
        rows: HashMap<Key, Row>,
    }

    static THREADS: Mutex<Vec<Arc<Mutex<Mine>>>> = Mutex::new(Vec::new());
    /// The backtraces taken for sites in the access layers.
    static BACKTRACES: AtomicU32 = AtomicU32::new(0);

    thread_local! {
        static MINE: RefCell<Option<Arc<Mutex<Mine>>>> = const { RefCell::new(None) };
    }

    fn file() -> Option<&'static std::path::Path> {
        static FILE: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
        FILE.get_or_init(|| {
            let value = std::env::var_os("RC2K_BY_ADDRESS")?;
            Some(if value == "1" {
                "by_address.csv".into()
            } else {
                value.into()
            })
        })
        .as_deref()
    }

    /// Whether a location is inside the access layers (`leaves::site` skips the same).
    fn in_layers(file: &str) -> bool {
        [
            "src/pointers.rs",
            "src/pointers/",
            "src/write_watch.rs",
            "src/session.rs",
        ]
        .iter()
        .any(|l| file.ends_with(l) || file.contains(l))
    }

    fn site_of(at: &Location<'static>) -> String {
        if !in_layers(at.file()) {
            return format!("{}:{}", at.file().trim_start_matches("src/"), at.line());
        }
        if BACKTRACES.fetch_add(1, Ordering::Relaxed) < 2000 {
            let site = super::leaves::site();
            if site != "?" {
                return site;
            }
        }
        format!("via {}:{}", at.file(), at.line())
    }

    pub fn note(
        store: &dyn Store,
        at: usize,
        len: usize,
        write: bool,
        from: &'static Location<'static>,
    ) {
        if len == 0 || file().is_none() {
            return;
        }
        let key = (
            store.first().addr(),
            at,
            write,
            from.file().as_ptr().addr(),
            from.line(),
        );
        let mine = MINE.with_borrow_mut(|m| {
            m.get_or_insert_with(|| {
                let mine = Arc::new(Mutex::new(Mine {
                    kind: super::fields::kind(),
                    rows: HashMap::new(),
                }));
                THREADS
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(mine.clone());
                mine
            })
            .clone()
        });
        let mut mine = mine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(row) = mine.rows.get_mut(&key) {
            row.count += 1;
            row.bytes += len as u64;
            return;
        }
        let mut path = String::new();
        if at < store.size() {
            store.field_path_at(at, &mut path);
        }
        let row = Row {
            store: crate::stores::name_of(store)
                .unwrap_or_else(|| store.type_name())
                .to_owned(),
            field: super::leaves::strip(&path),
            site: site_of(from),
            write,
            count: 1,
            bytes: len as u64,
        };
        mine.rows.insert(key, row);
    }

    /// A line of the file: (site, store, field).
    type RowKey = (String, String, String);

    /// What the threads did at one: (reads, writes, bytes read, bytes written, kinds).
    type Total = (u64, u64, u64, u64, u8);

    pub fn dump() {
        let Some(path) = file() else {
            return;
        };
        let mut totals: BTreeMap<RowKey, Total> = BTreeMap::new();
        for mine in THREADS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            let mine = mine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for row in mine.rows.values() {
                let t = totals
                    .entry((row.site.clone(), row.store.clone(), row.field.clone()))
                    .or_default();
                if row.write {
                    t.1 += row.count;
                    t.3 += row.bytes;
                } else {
                    t.0 += row.count;
                    t.2 += row.bytes;
                }
                t.4 |= 1 << mine.kind;
            }
        }
        let mut text =
            String::from("module,site,store,field,reads,writes,read_bytes,write_bytes,threads\n");
        for ((site, store, field), (reads, writes, read_bytes, write_bytes, kinds)) in totals {
            text += &format!(
                "{},\"{site}\",{store},{field},{reads},{writes},{read_bytes},{write_bytes},{}\n",
                super::leaves::module_of(&site),
                super::fields::kinds(kinds)
            );
        }
        let _ = std::fs::write(path, text);
    }
}

#[cfg(feature = "write_watch")]
mod on {
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;

    /// The dwords (host address / 4) read and written while watched.
    #[derive(Default)]
    pub struct Seen {
        pub read: HashSet<usize>,
        pub written: HashSet<usize>,
    }

    thread_local! {
        static ON: Cell<bool> = const { Cell::new(false) };
        static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
    }

    pub fn note(write: bool, at: *const u8, len: usize) {
        if len != 0 {
            super::fields::note(write, at, len);
        }
        if len != 0 && super::census::active() {
            super::census::note(write, at, len);
        }
        if !ON.get() || len == 0 {
            return;
        }
        let first = at.addr() >> 2;
        let last = (at.addr() + len - 1) >> 2;
        SEEN.with_borrow_mut(|s| {
            let set = if write { &mut s.written } else { &mut s.read };
            set.extend(first..=last);
        });
    }

    /// The thread's watch begun (what it saw before forgotten).
    pub fn begin() {
        SEEN.with_borrow_mut(|s| *s = Seen::default());
        ON.set(true);
    }

    /// The thread's watch ended: what it saw.
    pub fn end() -> Seen {
        ON.set(false);
        SEEN.with_borrow_mut(std::mem::take)
    }
}

/// The places one part writes and another reads or writes (what threads drawing the parts at
/// once race on), gathered over the drawings and
/// logged as store field paths (`stores`) or host addresses (Rust's own state) at the 300th.
#[cfg(feature = "write_watch")]
pub(crate) mod shared {
    use std::collections::BTreeSet;
    use std::sync::Mutex;

    static SHARED: Mutex<(BTreeSet<usize>, u32)> = Mutex::new((BTreeSet::new(), 0));

    pub fn add(seen: &[crate::write_watch::Seen]) {
        let mut s = SHARED.lock().unwrap();
        for (i, a) in seen.iter().enumerate() {
            for w in &a.written {
                if seen
                    .iter()
                    .enumerate()
                    .any(|(j, b)| j != i && (b.written.contains(w) || b.read.contains(w)))
                {
                    s.0.insert(*w);
                }
            }
        }
        s.1 += 1;
        if !s.1.is_multiple_of(300) {
            return;
        }
        let mut text = String::from("workers: shared places:");
        let words: Vec<usize> = s.0.iter().copied().collect();
        let mut k = 0;
        while k < words.len() {
            let first = words[k];
            while k + 1 < words.len() && words[k + 1] == words[k] + 1 {
                k += 1;
            }
            let len = (words[k] - first + 1) * 4;
            let host = first * 4;
            if let Some((slot, at)) = crate::race::view_scratch::place_of(host) {
                let window = crate::race::view_scratch::window_range();
                if window.contains(&at) {
                    // The view's window (`scene::view_window`), from its own start.
                    text += &format!(
                        " [view slot {slot} window +{:#x}+{len:#x}]",
                        at - window.start
                    );
                } else {
                    text += &format!(" [view slot {slot} +{at:#x}+{len:#x}]");
                }
                k += 1;
                continue;
            }
            if let Some((what, at)) = crate::memory::shared_table::table_of(host) {
                text += &format!(" [{what} +{at:#x}+{len:#x}]");
                k += 1;
                continue;
            }
            match crate::stores::find(host) {
                Some(found) => text += &format!(" {}{}+{len:#x}", found.name, found.field()),
                None => text += &format!(" host:{host:#x}+{len:#x}"),
            }
            k += 1;
        }
        crate::framework::log(&text);
    }
}
