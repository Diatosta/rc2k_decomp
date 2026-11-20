//! `effects::pong_setup`: the pong game's frame set up, as `effects::windscreen_flakes_setup` with
//! the texture places from the pong's own constants.

use crate::effects::pong::Pong;
use crate::effects::windscreen_flakes_setup::setup_with;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;

pub(super) fn pong_setup(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    scene: &crate::scene::owner::Scene,
) {
    setup_with(
        buffer,
        fl,
        pong.rules
            .setup_places
            .each_ref()
            .map(crate::memory::session::PlainFloat::get),
        scene,
    );
}
