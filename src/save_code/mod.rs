//! The save code (`rc2k_core::saves::code`): the player's state as text and a QR code, and the
//! page on the options menu that shows it, copies it, and takes one in from the clipboard or a
//! camera. `gather` reads the state out of the game, `apply` writes one back the way the game
//! saves it, `page` is what the player sees.

mod apply;
mod gather;
pub(crate) mod page;
mod page_draw;
mod scan;
