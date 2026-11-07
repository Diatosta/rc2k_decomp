//! `effects::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `effects::windscreen_flakes_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::run_pong`.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_setup::setup_with;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;

/// The Rust version (see the header).
pub fn pong_setup(pong: &Pong, fl: &WindscreenFlakes, scene: &crate::scene::owner::Scene) {
    setup_with(
        fl,
        pong.rules
            .setup_places
            .each_ref()
            .map(super::super::session::Float::bits),
        scene,
    );
}
