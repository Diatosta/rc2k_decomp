//! The game's log (`var\logfile.txt`: `write_line::log_text` and the number loggers) and the
//! program's own (`rc2k.log`: `program_log`), and the load timing report (`load_times`).

pub mod date_text;
pub mod load_times;
pub mod log_float;
pub mod log_hex8;
pub mod log_int;
pub mod log_int_fmt;
pub mod log_write;
pub mod numeric;
pub mod program_log;
pub mod start_log;
pub mod write_line;
