use crate::framework::GameHook;
use crate::{math, records, strings, text, threading};
use std::sync::LazyLock;

/// Whenever a hook module is registered, it is added to this list.
pub static ALL: LazyLock<Vec<&'static dyn GameHook>> = LazyLock::new(|| {
    [
        threading::HOOKS,
        strings::HOOKS,
        math::HOOKS,
        records::HOOKS,
        text::HOOKS,
    ]
    .into_iter()
    .flatten()
    .copied()
    .collect()
});
