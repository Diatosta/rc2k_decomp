use crate::framework::GameHook;

pub mod date_text;
pub mod log_float;
pub mod log_hex2;
pub mod log_hex4;
pub mod log_hex8;
pub mod log_int;
pub mod log_int_fmt;
pub mod log_write;
pub mod numeric;
pub mod start_log;
pub mod write_line;

pub static HOOKS: &[&dyn GameHook] = &[
    &write_line::HOOK,
    &log_write::HOOK,
    &log_int::HOOK,
    &log_int_fmt::HOOK,
    &log_float::HOOK,
    &log_hex8::HOOK,
    &log_hex2::HOOK,
    &log_hex4::HOOK,
    &date_text::HOOK,
];
