//! The game's log (`var\logfile.txt`): hooks removed in conversion phase 4, 2026-10-22; Rust
//! callers call `write_line::log_text` and the number loggers directly, or through the
//! routes (`crate::routes`).

pub mod date_text;
pub mod log_float;
pub mod log_hex8;
pub mod log_int;
pub mod log_int_fmt;
pub mod log_write;
pub mod numeric;
pub mod start_log;
pub mod write_line;
