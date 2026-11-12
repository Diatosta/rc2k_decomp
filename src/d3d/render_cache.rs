//! The render states' and the current texture's cache as the drawing sees it: each part of a
//! frame drawn beside the others (`workers::draw_parts`) keeps its own (`OwnCache`,
//! `workers::PartScratch::cache`). The first part's is the game's own (the game's thread
//! draws part 0 and everything outside the parts, so it is always in use); a part other than
//! the first starts from the cache as the list began, and what it leaves is put back into the
//! first's by `run_states` (the states it set) and `merge_texture`.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

use crate::render::STATES;
use crate::workers::PartScratch;

/// The cache: each state's last value, the texture set last.
#[derive(Clone)]
pub struct Cache {
    kept: [u32; STATES],
    texture: u32,
}

/// What a render state held before anything was set (a state that is not yet known).
const UNSET: u32 = 0xFFFF_FC43;

/// A drawing part's own cache: in use while a part other than the first is drawn (begun and
/// ended on the thread drawing it), always for the first part. A part other than the first
/// outside its run reads and writes the first's.
pub struct OwnCache {
    on: AtomicBool,
    kept: [AtomicU32; STATES],
    texture: AtomicU32,
}

impl OwnCache {
    /// The cache of a part other than the first: in use from `begin`.
    pub const fn new() -> Self {
        Self {
            on: AtomicBool::new(false),
            kept: [const { AtomicU32::new(0) }; STATES],
            texture: AtomicU32::new(0),
        }
    }

    /// The first part's: in use, every state and the texture not yet known.
    pub const fn first() -> Self {
        Self {
            on: AtomicBool::new(true),
            kept: [const { AtomicU32::new(UNSET) }; STATES],
            texture: AtomicU32::new(UNSET),
        }
    }
}

/// The cache `part` works on: its own while in use, else the first part's.
fn active(part: &PartScratch) -> &OwnCache {
    if part.cache.on.load(Relaxed) {
        &part.cache
    } else {
        &crate::workers::part_scratch(0).cache
    }
}

/// The first part's cache now (`first`: `part_scratch(0).cache`).
pub fn current(first: &OwnCache) -> Cache {
    Cache {
        kept: std::array::from_fn(|i| first.kept[i].load(Relaxed)),
        texture: first.texture.load(Relaxed),
    }
}

/// Part `part` (not the first) begins, from `start`.
pub fn begin(part: &PartScratch, start: &Cache) {
    let own = &part.cache;
    for (cell, &v) in own.kept.iter().zip(&start.kept) {
        cell.store(v, Relaxed);
    }
    own.texture.store(start.texture, Relaxed);
    own.on.store(true, Relaxed);
}

/// Part `part` (not the first) is over: the cache it leaves.
pub fn end(part: &PartScratch) -> Cache {
    let own = &part.cache;
    if !own.on.swap(false, Relaxed) {
        return current(&crate::workers::part_scratch(0).cache);
    }
    current(own)
}

/// Render state `state`'s last value as `part` has it (`state` below `STATES`).
pub fn kept(part: &PartScratch, state: usize) -> u32 {
    active(part).kept[state].load(Relaxed)
}

/// Render state `state` kept as `value` in `part`'s cache.
pub fn set_kept(part: &PartScratch, state: usize, value: u32) {
    active(part).kept[state].store(value, Relaxed);
}

/// Every state's last value as `part` has it.
pub fn all_kept(part: &PartScratch) -> [u32; STATES] {
    current(active(part)).kept
}

/// The first part's states now.
pub fn stored_kept(first: &OwnCache) -> [u32; STATES] {
    current(first).kept
}

/// The texture set last as `part` has it.
pub fn texture(part: &PartScratch) -> u32 {
    active(part).texture.load(Relaxed)
}

/// The texture set last in `part`'s cache made `texture`.
pub fn set_texture(part: &PartScratch, texture: u32) {
    active(part).texture.store(texture, Relaxed);
}

/// A part left `run`: its texture goes into the first part's cache (`first`) if it changed
/// from `start` (as the copies' byte merge did).
pub fn merge_texture(first: &OwnCache, run: &Cache, start: &Cache) {
    if run.texture != start.texture {
        first.texture.store(run.texture, Relaxed);
    }
}

/// The state values a part left.
pub fn left_kept(run: &Cache, state: usize) -> u32 {
    run.kept[state]
}

/// Sets the first part's (`first`) state `state` to `value` (a part's, after it ended).
pub fn put_kept(first: &OwnCache, state: usize, value: u32) {
    first.kept[state].store(value, Relaxed);
}
