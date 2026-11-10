//! The audit build's record of the game state's reads and writes (cargo feature `write_watch`;
//! `workers` uses it to find what two drawing parts both reach). Every
//! cell's load and store (`session`'s cells, `pointers`' cell runs, `shared_memory`) calls
//! `read` or `wrote`; without the feature they are empty and cost nothing. With it, while a
//! thread watches (`begin` .. `end`), each place reached is noted by its dword (a byte noted
//! as the dword that holds it: two neighbouring bytes count as one place, which can only add
//! to what is found).

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
    census::frame_begun();
}

/// The frame census (`RC2K_FRAME_CENSUS`): for every step of the race's frame, what it reads
/// that another step (or itself, a frame before) wrote: the steps' real inputs and outputs.
/// Every 600 frames each edge "writer -> reader" is logged with the places it hands over, by
/// IDA address (`tools/frame_census.py` names them).
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
                    } else {
                        let game = crate::address_map::game_of(host).map(|g| g as usize);
                        match game.and_then(crate::moved::ida_of) {
                            Some(ida) => text += &format!(" {ida:#x}+{len:#x}"),
                            None => text += &format!(" host:{host:#x}+{len:#x}"),
                        }
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
/// logged as IDA ranges (the moved ranges) or game addresses (Rust's own state) at the 300th.
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
            let game = crate::address_map::game_of(host).map(|g| g as usize);
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
            match game.and_then(crate::moved::ida_of) {
                Some(ida) => text += &format!(" {ida:#x}+{len:#x}"),
                None => text += &format!(" game:{:#x}+{len:#x}", game.unwrap_or(0)),
            }
            k += 1;
        }
        crate::framework::log(&text);
    }
}
