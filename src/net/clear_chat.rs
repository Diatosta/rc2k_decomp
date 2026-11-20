//! Empties the lobby's chat: eight empty lines.
pub(super) fn clear_chat(vars: &crate::net::store::NetShared) {
    // The logic is in `rc2k_core::net::chat::EMPTY`.
    if let Err(e) = vars
        .chat
        .write_at(0, &rc2k_core::net::chat::EMPTY, "the chat line")
    {
        crate::fatal::raise::end("the chat line", e)
    }
}
