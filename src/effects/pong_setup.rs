//! `effects::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `effects::windscreen_flakes_setup` with the texture places from the pong's own constants.
//!
//! Caller: `effects::run_pong`.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_setup::setup_with;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;

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
