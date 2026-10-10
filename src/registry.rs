use crate::framework::GameHook;
use crate::{
    files, gfx, input, logfile, math, records, screens, service, strings, text, threading,
};
use std::sync::LazyLock;

/// Whenever a hook module is registered, it is added to this list.
pub static ALL: LazyLock<Vec<&'static dyn GameHook>> = LazyLock::new(|| {
    [
        threading::HOOKS,
        strings::HOOKS,
        math::HOOKS,
        records::HOOKS,
        text::HOOKS,
        gfx::HOOKS,
        screens::HOOKS,
        service::HOOKS,
        input::HOOKS,
        files::HOOKS,
        logfile::HOOKS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect()
});

/// Whether a name or prefix typed in the console selects the hook called `name`: `all` and a
/// prefix select every hook they match, but the full name of a hook selects only that hook,
/// even if other names start with it.
pub fn selects(pattern: &str, name: &str) -> bool {
    if ALL.iter().any(|h| h.name() == pattern) {
        return name == pattern;
    }
    pattern == "all" || name.starts_with(pattern)
}
