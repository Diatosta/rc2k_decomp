//! The stages' built-in records: which stages make up which rally, their names' text numbers,
//! lengths and record holders (`stages_store`), the look-ups by stage, and `seal_time`, which
//! scrambles a time so that a stored record is tied to its entry.

pub(crate) mod default_stage_records;
pub(crate) mod find_record;
pub(crate) mod find_stage_record;
pub(crate) mod seal_time;
pub(crate) mod stages_store;
pub(crate) mod trakpak_entry;
