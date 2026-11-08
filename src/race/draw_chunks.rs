//! A check for drawing a view's entries on several threads (the optimisation phase): with
//! `RC2K_CHUNK_CHECK=k` in the environment, `draw_entries` draws its list as `k` runs of
//! entries, each from the state the list began with, as a thread with its own copy of that
//! state would. Its noted drawing is put back in the list's order and the state the runs
//! changed is put together in that order, so the frame comes out as one run's when no run reads
//! what an earlier one wrote. With `RC2K_CHUNK_REVERSE` too, the runs are drawn last first: a
//! run that reads an earlier one's writes then draws differently. The check compares the fan
//! hashes (`RC2K_FAN_HASH`, `render::dump::hash_if_asked`) of a replay played with and without
//! it (`RC2K_SHOT_CLOCKS` holds the clock, so both draw the same frames).
//!
//! The state is every moved range from 0x518000 on (below it the sound's and the input's,
//! which their own threads change meanwhile), the point records and the scene points.

use crate::errors::Refused;
use std::ops::Range;
use std::sync::OnceLock;

/// The runs asked for; 0 or 1 is the plain drawing.
pub fn chunks() -> usize {
    static K: OnceLock<usize> = OnceLock::new();
    *K.get_or_init(|| {
        std::env::var("RC2K_CHUNK_CHECK")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

fn reversed() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_CHUNK_REVERSE").is_some())
}

/// The first IDA address the state reaches.
const FROM: usize = 0x51_8000;
/// A dword the input's thread writes (`dword_518998`): its range is left out.
const INPUT_DWORD: usize = 0x51_8998;

enum Kind {
    Moved(usize),
    Records,
    Points,
}

/// One part of the state and its bytes.
struct Part {
    kind: Kind,
    bytes: Vec<u8>,
    /// Its bytes that are a drawing thread's own (`RC2K_CHUNK_PRIVATE`): only the last run's
    /// changes to them kept.
    private: Vec<bool>,
}

/// The private list's words for the point records and the scene points.
const RECORDS: usize = usize::MAX;
const POINTS: usize = usize::MAX - 1;

/// With `RC2K_CHUNK_PRIVATE=<file>` (IDA places `address+length` in hex, one a line, as the
/// write watch lists them, and the words `records` and `points`): those bytes, and the point
/// records and scene points when named, are each drawing thread's own, the game's thread
/// drawing the last run: after the runs only the last run's changes to them are kept, as when
/// the other threads' copies are dropped.
fn private_places() -> Option<&'static [(usize, usize)]> {
    static PLACES: OnceLock<Option<Vec<(usize, usize)>>> = OnceLock::new();
    PLACES
        .get_or_init(|| {
            let text = std::fs::read_to_string(std::env::var_os("RC2K_CHUNK_PRIVATE")?).ok()?;
            Some(
                text.lines()
                    .filter_map(|l| {
                        let hex = |t: &str| usize::from_str_radix(t.trim_start_matches("0x"), 16);
                        match l.trim() {
                            "records" => Some((RECORDS, 1)),
                            "points" => Some((POINTS, 1)),
                            a => match a.split_once('+') {
                                Some((at, len)) => Some((hex(at).ok()?, hex(len).ok()?)),
                                None => Some((hex(a).ok()?, 1)),
                            },
                        }
                    })
                    .collect(),
            )
        })
        .as_deref()
}

impl Part {
    fn now(&self) -> Vec<u8> {
        match self.kind {
            Kind::Moved(n) => {
                let mut v = vec![0u8; self.bytes.len()];
                crate::moved::load_moved(n, &mut v);
                v
            }
            Kind::Records => crate::scene::point_records::all_bytes(),
            Kind::Points => crate::race::scene_points::all_bytes(),
        }
    }

    fn put(&self, bytes: &[u8]) {
        match self.kind {
            Kind::Moved(n) => crate::moved::store_moved(n, bytes),
            Kind::Records => crate::scene::point_records::put_all_bytes(bytes),
            Kind::Points => crate::race::scene_points::put_all_bytes(bytes),
        }
    }
}

/// The changed bytes of one part: (offset, bytes) runs.
type Runs = Vec<(usize, Vec<u8>)>;

fn changed_runs(was: &[u8], now: &[u8]) -> Runs {
    let mut runs = Vec::new();
    let mut k = 0;
    while k < was.len().min(now.len()) {
        if was[k] != now[k] {
            let s = k;
            while k < was.len() && was[k] != now[k] {
                k += 1;
            }
            runs.push((s, now[s..k].to_vec()));
        } else {
            k += 1;
        }
    }
    runs
}

fn take_state() -> Vec<Part> {
    let mut parts: Vec<Part> = crate::moved::moved_ranges()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.start >= FROM && !r.contains(&INPUT_DWORD))
        .map(|(n, r)| Part {
            kind: Kind::Moved(n),
            bytes: vec![0u8; r.len()],
            private: (r.start..r.end)
                .map(|a| {
                    private_places()
                        .is_some_and(|p| p.iter().any(|&(at, len)| (at..at + len).contains(&a)))
                })
                .collect(),
        })
        .collect();
    parts.push(Part {
        kind: Kind::Records,
        bytes: Vec::new(),
        private: vec![private_places().is_some_and(|p| p.contains(&(RECORDS, 1)))],
    });
    parts.push(Part {
        kind: Kind::Points,
        bytes: Vec::new(),
        private: vec![private_places().is_some_and(|p| p.contains(&(POINTS, 1)))],
    });
    for p in &mut parts {
        p.bytes = p.now();
    }
    parts
}

/// Every part back to its bytes at the start, writing only what changed.
fn restore(parts: &[Part]) {
    for p in parts {
        let now = p.now();
        if now != p.bytes && now.len() == p.bytes.len() {
            p.put(&p.bytes);
        }
    }
}

/// The `count` entries drawn by `draw` (a range of them at a time) as `chunks()` runs.
pub fn draw_in_chunks(
    count: usize,
    mut draw: impl FnMut(Range<usize>) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let k = chunks().min(count).max(1);
    let ranges: Vec<Range<usize>> = (0..k).map(|i| i * count / k..(i + 1) * count / k).collect();
    let start = take_state();
    let mark = crate::platform_3d::noted_len();
    let flushes = crate::platform_3d::flushes();
    let mut noted: Vec<Vec<crate::render::Command>> = (0..k).map(|_| Vec::new()).collect();
    let mut changes: Vec<Vec<Runs>> = (0..k).map(|_| Vec::new()).collect();
    let order: Vec<usize> = if reversed() {
        (0..k).rev().collect()
    } else {
        (0..k).collect()
    };
    #[cfg(feature = "write_watch")]
    let mut seen: Vec<crate::write_watch::Seen> = Vec::new();
    for i in order {
        restore(&start);
        #[cfg(feature = "write_watch")]
        crate::write_watch::begin();
        draw(ranges[i].clone())?;
        #[cfg(feature = "write_watch")]
        seen.push(crate::write_watch::end());
        if crate::platform_3d::flushes() != flushes {
            crate::framework::log("draw_chunks: a flush inside a run (the check is void)");
        }
        noted[i] = crate::platform_3d::take_noted_from(mark);
        changes[i] = start
            .iter()
            .map(|p| changed_runs(&p.bytes, &p.now()))
            .collect();
    }
    // The runs' changes in the list's order: the last run's write of a byte is the one kept,
    // as when drawn one after the other.
    for (n, p) in start.iter().enumerate() {
        let mut v = p.bytes.clone();
        let own = |i: usize| {
            p.private
                .get(i)
                .or(p.private.first())
                .copied()
                .unwrap_or(false)
        };
        for (r, c) in changes.iter().enumerate() {
            for (at, bytes) in &c[n] {
                for (i, &b) in (*at..).zip(bytes) {
                    if r == k - 1 || !own(i) {
                        v[i] = b;
                    }
                }
            }
        }
        if p.now() != v {
            p.put(&v);
        }
    }
    for n in noted {
        crate::platform_3d::put_noted(n);
    }
    #[cfg(feature = "write_watch")]
    shared::add(&seen);
    Ok(())
}

/// With the feature `write_watch`: the places one run writes and another reads or writes
/// (what threads drawing the runs at once would race on), gathered over the lists and
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
        let mut text = String::from("draw_chunks: shared places:");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_runs_are_the_differing_bytes() {
        let was = [1, 2, 3, 4, 5, 6];
        let now = [1, 9, 9, 4, 5, 7];
        assert_eq!(
            changed_runs(&was, &now),
            vec![(1, vec![9, 9]), (5, vec![7])]
        );
        assert!(changed_runs(&was, &was).is_empty());
    }
}
