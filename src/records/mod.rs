//! The stages' built-in records: which stages make up which rally, their names' text numbers,
//! lengths and record holders (`stages_store`), the look-ups by stage, and `seal_time`, which
//! scrambles a time so that a stored record is tied to its entry.

pub mod default_stage_records;
pub mod find_record;
pub mod find_stage_record;
pub mod seal_time;
pub mod stages_store;
pub mod trakpak_entry;
