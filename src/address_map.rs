//! The registry of game addresses: the game's data holds 32-bit addresses, and each is a
//! number handed out here, never a host address (on 32 bits too since 2026-11-05). Every
//! host range given a name (the program's image, a thread's stack, a block of the pool) gets
//! a game range, handed out from `FIRST` up in granules of 64 KiB (keeping a host address's
//! offset in its granule); `pointers::game_address` names a host place through it. A game
//! address is reached through the store its region names (`pointers`), never turned back into
//! a host pointer: only the tests' raw regions are (`host`, through `DELTA`). A number range
//! with no host memory behind it can be taken as well (`reserve`: the course's places).
//!
//! The x86 tests alone keep the identity (`IDENTITY`: a game address is the host's): their
//! kept assembly is handed game addresses and reaches them as host ones.

use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

/// The x86 tests' identity (see the module's head).
const IDENTITY: bool = cfg!(all(test, target_arch = "x86"));

const GRANULE_BITS: u32 = 16;
const GRANULE: usize = 1 << GRANULE_BITS;
#[cfg(test)]
const GRANULES: usize = 1 << (32 - GRANULE_BITS);
/// The first granule handed out (game 0x0100_0000): small numbers are never addresses.
const FIRST: u32 = 0x0100;
/// The granule past the last one handed out (game 0xFFFF_0000).
const LAST: u32 = 0xFFFF;

/// Per game granule: its host address less its game address, with bit 0 set; 0 when no
/// range covers it (the tests' `host`).
#[cfg(test)]
static DELTA: [AtomicUsize; GRANULES] = [const { AtomicUsize::new(0) }; GRANULES];

/// Bumped when a range is taken off: the threads' caches are then emptied.
static GENERATION: AtomicU32 = AtomicU32::new(0);

/// How many times a range was taken off: a cache of game ranges (`pointers`' stack) is
/// stale when it changed, as a thread's stack may have been given a new range. Compared
/// only (a change sends the caller to lookups that take the locks), so read relaxed.
pub fn generation() -> u32 {
    GENERATION.load(Ordering::Relaxed)
}

/// A host range and its game range.
#[derive(Clone, Copy, Debug)]
struct Range {
    host: usize,
    len: usize,
    game: u32,
}

impl Range {
    /// The game address of `p`, if it lies in the range. Its end is not in it: blocks the
    /// system hands out lie back to back, and the next one's first byte is this one's end
    /// (`end_of` answers an end pointer when no range holds it).
    fn game_of(self, p: usize) -> Option<u32> {
        (p >= self.host && p - self.host < self.len).then(|| self.game + (p - self.host) as u32)
    }

    /// The game address of `p` if it is the range's end (one past its last byte).
    fn end_of(self, p: usize) -> Option<u32> {
        (p == self.host + self.len).then(|| self.game + self.len as u32)
    }

    /// The granules its game range takes: first, count.
    fn granules(self) -> (u32, u32) {
        let first = self.game >> GRANULE_BITS;
        let last = ((self.game as usize + self.len.max(1) - 1) >> GRANULE_BITS) as u32;
        (first, last - first + 1)
    }
}

struct Ranges {
    /// Host start -> range.
    by_host: BTreeMap<usize, Range>,
    /// Game start -> host start.
    by_game: BTreeMap<u32, usize>,
    /// Free granule runs: first -> count.
    free: BTreeMap<u32, u32>,
}

static RANGES: Mutex<Ranges> = Mutex::new(Ranges {
    by_host: BTreeMap::new(),
    by_game: BTreeMap::new(),
    free: BTreeMap::new(),
});

/// The host address of game address `addr` (not 0), or None where no range covers it: the
/// tests' raw regions only.
#[cfg(test)]
#[inline]
pub fn host(addr: u32) -> Option<usize> {
    if IDENTITY {
        return Some(addr as usize);
    }
    let d = DELTA[(addr >> GRANULE_BITS) as usize].load(Ordering::Acquire);
    (d != 0).then(|| (addr as usize).wrapping_add(d & !1))
}

thread_local! {
    /// The thread's stack's range, and the last ranges found with their generation.
    static LOCAL: std::cell::RefCell<Local> = const {
        std::cell::RefCell::new(Local { stack: None, generation: u32::MAX, recent: [None; 4], next: 0 })
    };
}

struct Local {
    stack: Option<Range>,
    generation: u32,
    recent: [Option<Range>; 4],
    next: usize,
}

/// The game address of host address `p` (not 0): in the image, the thread's stack, or a
/// range made for it; None if `p` is in none.
pub fn game_of(p: usize) -> Option<u32> {
    if IDENTITY {
        return Some(p as u32);
    }
    if let Some(at) = image().game_of(p) {
        return Some(at);
    }
    LOCAL.with(|local| {
        let mut local = local.borrow_mut();
        let generation = GENERATION.load(Ordering::Acquire);
        if local.generation != generation {
            local.generation = generation;
            local.recent = [None; 4];
            local.stack = None;
        }
        let stack = *local.stack.get_or_insert_with(|| {
            let (low, high) = crate::pointers::thread_stack();
            map_range(low, high - low)
        });
        if let Some(at) = stack.game_of(p) {
            return Some(at);
        }
        if let Some(at) = local.recent.iter().flatten().find_map(|r| r.game_of(p)) {
            return Some(at);
        }
        let range = {
            let ranges = RANGES
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match ranges.by_host.range(..=p).next_back() {
                Some((_, &r)) if r.game_of(p).is_some() => r,
                // An end pointer: the range that ends there.
                _ => {
                    let at = ranges.by_host.range(..p).next_back()?.1.end_of(p);
                    return at.or_else(|| image().end_of(p)).or_else(|| stack.end_of(p));
                }
            }
        };
        let next = local.next;
        local.recent[next] = Some(range);
        local.next = (next + 1) % 4;
        range.game_of(p)
    })
}

/// The program's own image (code, constants, statics), its range made at the first use.
fn image() -> Range {
    static IMAGE: OnceLock<Range> = OnceLock::new();
    *IMAGE.get_or_init(|| {
        let (start, end) = crate::pointers::own_image();
        map_range(start, end - start)
    })
}

/// A game range for `len` bytes of host memory from `host` (its game address returned): the
/// range that holds them if there is one; else a new one, after taking off the stale ranges
/// they overlap (memory given back and taken again).
pub fn map(host: usize, len: usize) -> u32 {
    if IDENTITY {
        return host as u32;
    }
    map_range(host, len).game
}

/// A game range of `len` bytes with no host memory behind it: numbers only, each reached
/// through the store that answers for it (the course's places, `race::course_places`).
pub fn reserve(len: usize) -> u32 {
    if IDENTITY {
        // Untouched pages: numbers no other memory has (nothing reaches them as bytes).
        let at = crate::os::map_pages(len).expect("pages for the course's places");
        return at.expose_provenance() as u32;
    }
    let mut ranges = RANGES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ranges.free.is_empty() && ranges.by_host.is_empty() {
        ranges.free.insert(FIRST, LAST - FIRST);
    }
    take_granules(&mut ranges, len.max(1).div_ceil(GRANULE) as u32) << GRANULE_BITS
}

/// `count` free granules taken: the first one's number.
fn take_granules(ranges: &mut Ranges, count: u32) -> u32 {
    let Some((&first, &run)) = ranges.free.iter().find(|&(_, &n)| n >= count) else {
        crate::fatal::raise::end(
            "pointers: the game's 4 GiB of addresses are all handed out",
            crate::errors::BadInput("no game addresses left"),
        )
    };
    ranges.free.remove(&first);
    if run > count {
        ranges.free.insert(first + count, run - count);
    }
    first
}

fn map_range(host: usize, len: usize) -> Range {
    let mut ranges = RANGES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if ranges.free.is_empty() && ranges.by_host.is_empty() {
        ranges.free.insert(FIRST, LAST - FIRST);
    }
    if let Some((_, &r)) = ranges.by_host.range(..=host).next_back()
        && host >= r.host
        && host + len <= r.host + r.len
    {
        return Range {
            host,
            len,
            game: r.game + (host - r.host) as u32,
        };
    }
    let stale: Vec<usize> = ranges
        .by_host
        .range(..host + len.max(1))
        .rev()
        .take_while(|(_, r)| r.host + r.len > host)
        .map(|(&h, _)| h)
        .collect();
    for h in stale {
        take_off(&mut ranges, h);
    }
    let offset = host & (GRANULE - 1);
    let count = (offset + len.max(1)).div_ceil(GRANULE) as u32;
    let first = take_granules(&mut ranges, count);
    let game = (first << GRANULE_BITS) | offset as u32;
    #[cfg(test)]
    {
        let delta = host.wrapping_sub(game as usize) | 1;
        for g in first..first + count {
            DELTA[g as usize].store(delta, Ordering::Release);
        }
    }
    let range = Range { host, len, game };
    ranges.by_host.insert(host, range);
    ranges.by_game.insert(game, host);
    range
}

/// The range made at game address `game` taken off (its memory given back).
pub fn unmap(game: u32) {
    if IDENTITY {
        return;
    }
    let mut ranges = RANGES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(&h) = ranges.by_game.get(&game) {
        take_off(&mut ranges, h);
    }
}

fn take_off(ranges: &mut Ranges, host: usize) {
    let Some(r) = ranges.by_host.remove(&host) else {
        return;
    };
    ranges.by_game.remove(&r.game);
    let (first, count) = r.granules();
    #[cfg(test)]
    for g in first..first + count {
        DELTA[g as usize].store(0, Ordering::Release);
    }
    // The run given back, joined with free neighbours.
    let (mut first, mut count) = (first, count);
    if let Some((&before, &n)) = ranges.free.range(..first).next_back()
        && before + n == first
    {
        ranges.free.remove(&before);
        first = before;
        count += n;
    }
    if let Some(n) = ranges.free.remove(&(first + count)) {
        count += n;
    }
    ranges.free.insert(first, count);
    GENERATION.fetch_add(1, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_translates_both_ways_and_is_given_back() {
        let buf = vec![0u8; 3 * GRANULE + 17];
        let host = buf.as_ptr().expose_provenance() + 5;
        let game = map(host, buf.len() - 5);
        assert_eq!(game_of(host + 100), Some(game + 100));
        assert_eq!(super::host(game + 100), Some(host + 100));
        assert_eq!(map(host + 8, 10), game + 8);
        // A range right after it: its first byte is the first range's end.
        let next = Box::new([0u8; 64]);
        let next_host = next.as_ptr().expose_provenance();
        let first = map(next_host, 32);
        let second = map(next_host + 32, 32);
        assert_eq!(game_of(next_host + 32), Some(second));
        assert_eq!(game_of(next_host + 31), Some(first + 31));
        unmap(first);
        unmap(second);
        if !IDENTITY {
            assert_eq!(game as usize % GRANULE, host % GRANULE);
            assert_ne!(game as usize, host, "a number, not the host address");
            unmap(game);
            assert_eq!(super::host(game), None);
        }
    }
}
