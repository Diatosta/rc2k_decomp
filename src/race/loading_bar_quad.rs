//! The loading bar's quad: a red flat rectangle along the bottom of the screen, x from 0 to the
//! screen's width times the share (0 ..= 100) over 100, y from 0x24D to 0x258 scaled from 600 to
//! the screen's height, drawn by `race::draw_screen_quad`.

use crate::errors::Refused;
use crate::race::owner::Race;

pub(super) fn loading_bar_quad(
    race: &Race,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
    share: u32,
) -> Result<(), Refused> {
    let session = &system.session;
    // Logic in `rc2k_core::race::loading::loading_bar`.
    use rc2k_core::race::loading::{BAR_COLOUR, loading_bar};
    let [w, h] = [
        session.screen.get().width.get(),
        session.screen.get().height.get(),
    ];
    let bar = loading_bar(w, h, share);
    let scratch = scene.scratch.front_end();
    let polygon = &scratch.polygon;
    // The quad's colour; x at corners 0 / 1, y at 4 / 5.
    scratch.quad.borrow_mut().corners[0] = bar.left;
    scratch.quad.borrow_mut().colour[0] = BAR_COLOUR;
    scratch.quad.borrow_mut().corners[4] = bar.top;
    scratch.quad.borrow_mut().corners[5] = bar.bottom;
    scratch.quad.borrow_mut().corners[1] = bar.right;
    polygon.texture_page.set(0xFFFF_FFFFu32);
    crate::race::frame_draw::draw_screen_quad(
        polygon,
        scene.scratch.game_scratch(),
        &scratch.face,
        crate::scene::workers::DrawReads::on_game_thread(crate::scene::workers::LentOwners {
            race,
            scene,
            system,
            net,
        }),
        session.screen.get(),
        scratch,
    )
}
