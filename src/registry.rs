use crate::fatal;
use crate::framework::GameHook;
use std::sync::LazyLock;

/// Whenever a hook module is registered, it is added to this list.
pub static ALL: LazyLock<Vec<&'static dyn GameHook>> =
    LazyLock::new(|| [fatal::HOOKS].into_iter().flatten().copied().collect());
