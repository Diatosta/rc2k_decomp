//! The game's log (`var\logfile.txt`: `write_line::log_text` and the number loggers) and the
//! program's own (`rc2k.log`: `crate::program_log`), and the load timing report (`load_times`).

mod date_text;
pub(crate) mod load_times;
pub(crate) mod log_float;
pub(crate) mod log_hex8;
pub(crate) mod log_int;
pub(crate) mod log_int_fmt;
pub(crate) mod log_write;
pub(crate) mod numeric;
pub(crate) mod start_log;
pub(crate) mod write_line;
