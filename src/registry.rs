use crate::framework::GameHook;
use crate::{files, gfx, logfile, math, records, screens, strings, text, threading};
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
        files::HOOKS,
        logfile::HOOKS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect()
});
