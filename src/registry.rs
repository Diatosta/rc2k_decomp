use crate::framework::Hook;
use crate::{strings, threading};
use std::sync::LazyLock;

/// Whenever a hook is registered, it is added to this list.
pub static ALL: LazyLock<Vec<&'static Hook>> = LazyLock::new(|| {
    [threading::HOOKS, strings::HOOKS].into_iter().flatten().collect()
});