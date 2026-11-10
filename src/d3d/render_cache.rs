//! The render states' and the current texture's cache as the drawing sees it: each part of a
//! frame drawn beside the others (`workers::draw_parts`) keeps its own on its thread, so
//! `Cards3d` itself is one copy (no drawing part writes it). A part starts from the cache as
//! the list began (what a copy of the store was set to before), and what it leaves is put
//! back by `run_states` (the states it set) and `merge_texture`. Code that is not a part
//! (the game's thread, run 0 included) reads and writes the store's own.

use std::cell::RefCell;

use crate::d3d::owner::d3d;
use crate::render::STATES;

/// The cache: each state's last value, the texture set last.
#[derive(Clone)]
pub struct Cache {
    kept: [u32; STATES],
    texture: u32,
}

thread_local! {
    /// The running part's cache, if the thread is drawing a part other than run 0.
    static OWN: RefCell<Option<Cache>> = const { RefCell::new(None) };
}

/// The store's own cache now.
pub fn current() -> Cache {
    let cards = &d3d().cards;
    Cache {
        kept: std::array::from_fn(|i| cards.kept_states[i].get()),
        texture: cards.current_texture.get(),
    }
}

/// A part begins on this thread, from `start`.
pub fn begin(start: &Cache) {
    OWN.set(Some(start.clone()));
}

/// The part on this thread is over: the cache it leaves.
pub fn end() -> Cache {
    OWN.take().unwrap_or_else(current)
}

/// Render state `state`'s last value (`state` below `STATES`).
pub fn kept(state: usize) -> u32 {
    OWN.with_borrow(|own| match own {
        Some(c) => c.kept[state],
        None => d3d().cards.kept_states[state].get(),
    })
}

/// Render state `state` kept as `value`.
pub fn set_kept(state: usize, value: u32) {
    OWN.with_borrow_mut(|own| match own {
        Some(c) => c.kept[state] = value,
        None => d3d().cards.kept_states[state].set(value),
    });
}

/// Every state's last value.
pub fn all_kept() -> [u32; STATES] {
    OWN.with_borrow(|own| match own {
        Some(c) => c.kept,
        None => current().kept,
    })
}

/// The texture set last.
pub fn texture() -> u32 {
    OWN.with_borrow(|own| match own {
        Some(c) => c.texture,
        None => d3d().cards.current_texture.get(),
    })
}

/// The texture set last made `texture`.
pub fn set_texture(texture: u32) {
    OWN.with_borrow_mut(|own| match own {
        Some(c) => c.texture = texture,
        None => d3d().cards.current_texture.set(texture),
    });
}

/// A part left `run`: its texture goes into the store's if it changed from `start` (as the
/// copies' byte merge did).
pub fn merge_texture(run: &Cache, start: &Cache) {
    if run.texture != start.texture {
        d3d().cards.current_texture.set(run.texture);
    }
}

/// The state values a part left.
pub fn left_kept(run: &Cache, state: usize) -> u32 {
    run.kept[state]
}

/// Sets the store's state `state` to `value` (a part's, after it ended).
pub fn put_kept(state: usize, value: u32) {
    d3d().cards.kept_states[state].set(value);
}
