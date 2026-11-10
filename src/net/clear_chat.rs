//! `net::clear_chat` (0x419411): the lobby's chat (`byte_52A508`) made eight empty lines:
//! seven 0x0A and a NUL.
//!
//! Caller: `net::start_link` (with the received messages' lock held).
pub fn clear_chat() {
    // Converted: `rc2k_core::net::chat::EMPTY`.
    if let Err(e) = crate::net::owner::NET_OWNER.vars.chat.write_at(
        0,
        &rc2k_core::net::chat::EMPTY,
        "the chat line",
    ) {
        crate::fatal::raise::end("the chat line", e)
    }
}
