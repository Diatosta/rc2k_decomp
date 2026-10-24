//! `menu::replay_item_a` (0x47053E): a replay item of the review menus (see `menu/replay_items.rs`).
//!
//! In:  nothing
//! Out: the registers as its calls leave them; no flags read
//!
//! An item action in six menu tables. Hook removed 2026-10-22 (typed tables): the menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's,
//! for reference.

pub use crate::menu::replay_items::replay_item_a;
