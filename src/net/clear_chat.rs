//! `net::clear_chat` (0x419411): the lobby's chat (`chat`) made eight empty lines:
//! seven 0x0A and a NUL.
//!
//! Caller: `net::start_link` (with the received messages' lock held).
pub fn clear_chat(vars: &crate::net::store::NetVars) {
    // Converted: `rc2k_core::net::chat::EMPTY`.
    if let Err(e) = vars
        .chat
        .write_at(0, &rc2k_core::net::chat::EMPTY, "the chat line")
    {
        crate::fatal::raise::end("the chat line", e)
    }
}
