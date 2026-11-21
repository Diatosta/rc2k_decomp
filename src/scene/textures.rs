//! The tables that say which texture each of the scene's texture slots shows: shared by the
//! functions that fill them.
//!
//! A face names a slot; `WANTED` says which texture file's picture the slot should hold and
//! `LOADED` which it holds. An object's kind (its file's number) has a first slot and a first
//! texture.

pub const SLOTS: u32 = 0x400;
pub const KINDS: u32 = 0x400;
