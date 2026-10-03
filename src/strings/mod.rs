use crate::framework::Hook;

mod maybe_str_append;

pub static HOOKS: &[Hook] = &[Hook {
    name: "strings::maybe_str_append",
    rva: 0x401F7D - 0x400000,
    stats: &maybe_str_append::STATS,
    install: maybe_str_append::install,
    set_enabled: maybe_str_append::set_enabled,
    self_test: Some(maybe_str_append::self_test),
}];