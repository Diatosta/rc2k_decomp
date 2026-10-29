//! `effects::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `effects::squares_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_pong`.

use crate::effects::pong::PONG_RULES;
use crate::effects::squares_setup::setup_with;

/// The Rust version (see the header).
pub fn pong_setup() {
    setup_with(PONG_RULES.setup_places.each_ref().map(|f| f.bits()))
}
