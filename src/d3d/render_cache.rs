//! The render states' and the current texture's cache as the drawing sees it: each part of a
//! frame drawn beside the others (`workers::draw_parts`) other than the first keeps its own
//! (`OwnCache`, `workers::PartScratch::cache`), so `Cards3d` itself is one copy (no drawing
//! part writes it). A part starts from the cache as the list began (what a copy of the store
//! was set to before), and what it leaves is put back by `run_states` (the states it set) and
//! `merge_texture`. The first part and code that is not a part (the game's thread) read and
//! write the store's own: part 0's cache is never begun.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

use crate::d3d::cards_store::Cards3d;
use crate::render::STATES;
use crate::workers::PartScratch;

/// The cache: each state's last value, the texture set last.
#[derive(Clone)]
pub struct Cache {
    kept: [u32; STATES],
    texture: u32,
}

/// A drawing part's own cache: in use while a part other than the first is drawn, the
/// store's (`store`) otherwise. Only the thread drawing the part reaches it while it is in use
/// (begun and ended on that thread).
pub struct OwnCache {
    /// The store's own (`Cards3d`).
    store: &'static Cards3d,
    on: AtomicBool,
    kept: [AtomicU32; STATES],
    texture: AtomicU32,
}

impl OwnCache {
    /// A part's cache over the store `store`.
    pub const fn new(store: &'static Cards3d) -> Self {
        Self {
            store,
            on: AtomicBool::new(false),
            kept: [const { AtomicU32::new(0) }; STATES],
            texture: AtomicU32::new(0),
        }
    }

    /// The cache, if in use.
    fn get(&self) -> Option<&Self> {
        self.on.load(Relaxed).then_some(self)
    }

    /// The store the cache is over.
    pub fn store(&self) -> &'static Cards3d {
        self.store
    }
}

/// The store's own cache now.
pub fn current(cards: &Cards3d) -> Cache {
    Cache {
        kept: std::array::from_fn(|i| cards.kept_states[i].get()),
        texture: cards.current_texture.get(),
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

/// Part `part` is over: the cache it leaves.
pub fn end(part: &PartScratch) -> Cache {
    let own = &part.cache;
    if !own.on.swap(false, Relaxed) {
        return current(own.store);
    }
    Cache {
        kept: std::array::from_fn(|i| own.kept[i].load(Relaxed)),
        texture: own.texture.load(Relaxed),
    }
}

/// Render state `state`'s last value as `part` has it (`state` below `STATES`).
pub fn kept(part: &PartScratch, state: usize) -> u32 {
    match part.cache.get() {
        Some(c) => c.kept[state].load(Relaxed),
        None => part.cache.store.kept_states[state].get(),
    }
}

/// Render state `state` kept as `value` in `part`'s cache.
pub fn set_kept(part: &PartScratch, state: usize, value: u32) {
    match part.cache.get() {
        Some(c) => c.kept[state].store(value, Relaxed),
        None => part.cache.store.kept_states[state].set(value),
    }
}

/// Every state's last value as `part` has it.
pub fn all_kept(part: &PartScratch) -> [u32; STATES] {
    match part.cache.get() {
        Some(c) => std::array::from_fn(|i| c.kept[i].load(Relaxed)),
        None => current(part.cache.store).kept,
    }
}

/// The store's states now.
pub fn stored_kept(cards: &Cards3d) -> [u32; STATES] {
    current(cards).kept
}

/// The texture set last as `part` has it.
pub fn texture(part: &PartScratch) -> u32 {
    match part.cache.get() {
        Some(c) => c.texture.load(Relaxed),
        None => part.cache.store.current_texture.get(),
    }
}

/// The texture set last in `part`'s cache made `texture`.
pub fn set_texture(part: &PartScratch, texture: u32) {
    match part.cache.get() {
        Some(c) => c.texture.store(texture, Relaxed),
        None => part.cache.store.current_texture.set(texture),
    }
}

/// A part left `run`: its texture goes into the store's if it changed from `start` (as the
/// copies' byte merge did).
pub fn merge_texture(cards: &Cards3d, run: &Cache, start: &Cache) {
    if run.texture != start.texture {
        cards.current_texture.set(run.texture);
    }
}

/// The state values a part left.
pub fn left_kept(run: &Cache, state: usize) -> u32 {
    run.kept[state]
}

/// Sets the store's state `state` to `value` (a part's, after it ended).
pub fn put_kept(cards: &Cards3d, state: usize, value: u32) {
    cards.kept_states[state].set(value);
}
