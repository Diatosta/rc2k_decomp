//! The render states' and the current texture's cache as the drawing sees it: each part of a
//! frame drawn beside the others (`scene::workers::draw_parts`) keeps its own (`OwnCache`,
//! `scene::workers::PartScratch::cache`). The first part's is the game's own (the game's thread
//! draws part 0 and everything outside the parts, so it is always in use); a part other than
//! the first starts from the cache as the list began, and what it leaves is put back into the
//! first's by `run_states` (the states it set) and `merge_texture`.

use std::cell::Cell;

use crate::render::STATES;
use crate::scene::workers::PartScratch;

/// The cache: each state's last value, the texture set last.
#[derive(Clone)]
pub struct Cache {
    kept: [u32; STATES],
    texture: u32,
}

/// What a render state held before anything was set (a state that is not yet known).
const UNSET: u32 = 0xFFFF_FC43;

/// A drawing part's own cache: a part other than the first sets it from the list's cache as its
/// run begins. Plain cells: only the thread that draws the part reaches it
/// (`memory::part_cell::PartCell`), the game's thread the first's.
pub struct OwnCache {
    kept: [Cell<u32>; STATES],
    texture: Cell<u32>,
}

impl OwnCache {
    /// The cache of a part other than the first, set by `begin`.
    pub const fn new() -> Self {
        Self {
            kept: [const { Cell::new(0) }; STATES],
            texture: Cell::new(0),
        }
    }

    /// The first part's: every state and the texture not yet known.
    pub const fn first() -> Self {
        Self {
            kept: [const { Cell::new(UNSET) }; STATES],
            texture: Cell::new(UNSET),
        }
    }
}

/// The cache `part` works on: its own.
fn active(part: &PartScratch) -> &OwnCache {
    &part.cache
}

/// The first part's cache now (`first`: its part's `cache`).
pub fn current(first: &OwnCache) -> Cache {
    Cache {
        kept: std::array::from_fn(|i| first.kept[i].get()),
        texture: first.texture.get(),
    }
}

/// Part `part` (not the first) begins, from `start`.
pub fn begin(part: &PartScratch, start: &Cache) {
    let own = &part.cache;
    for (cell, &v) in own.kept.iter().zip(&start.kept) {
        cell.set(v);
    }
    own.texture.set(start.texture);
}

/// Part `part` (not the first) is over: the cache it leaves.
pub fn end(part: &PartScratch) -> Cache {
    let own = &part.cache;
    current(own)
}

/// Render state `state`'s last value as `part` has it (`state` below `STATES`).
pub fn kept(part: &PartScratch, state: usize) -> u32 {
    active(part).kept[state].get()
}

/// Render state `state` kept as `value` in `part`'s cache.
pub fn set_kept(part: &PartScratch, state: usize, value: u32) {
    active(part).kept[state].set(value);
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
    active(part).texture.get()
}

/// The texture set last in `part`'s cache made `texture`.
pub fn set_texture(part: &PartScratch, texture: u32) {
    crate::d3d::run_states::note_texture_set(part);
    active(part).texture.set(texture);
}

/// Whether two caches have the same texture set last.
pub fn same_texture(a: &Cache, b: &Cache) -> bool {
    a.texture == b.texture
}

/// The texture `run` left into the first part's cache `first` (a piece drawn aside that set
/// one, `run_states::carry_set`).
pub fn put_texture(first: &OwnCache, run: &Cache) {
    first.texture.set(run.texture);
}

/// A part left `run`: its texture goes into the first part's cache (`first`) if it changed
/// from `start`.
pub fn merge_texture(first: &OwnCache, run: &Cache, start: &Cache) {
    if run.texture != start.texture {
        first.texture.set(run.texture);
    }
}

/// The state values a part left.
pub fn left_kept(run: &Cache, state: usize) -> u32 {
    run.kept[state]
}

/// Sets the first part's (`first`) state `state` to `value` (a part's, after it ended).
pub fn put_kept(first: &OwnCache, state: usize, value: u32) {
    first.kept[state].set(value);
}
