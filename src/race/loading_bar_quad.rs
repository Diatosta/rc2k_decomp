//! `race::loading_bar_quad` (0x446861): the loading bar, a red (`quad_colour` 0xFF0000) flat
//! rectangle along the bottom: x from 0 to the screen's width times `bar_share` (held to 0 ..
//! 100) over 100, y from 0x24D to 0x258 scaled from 600 to the screen's height; `texture_page` -1;
//! `race::draw_screen_quad`.
//!
//! Caller: `race::loading_bar_frame`. The quad is drawn by `scene`.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn loading_bar_quad(
    race: &Race,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let shared = &race.shared;
    let session = &system.session;
    // Converted: `rc2k_core::race::loading::loading_bar`.
    use rc2k_core::race::loading::{BAR_COLOUR, loading_bar};
    let [w, h] = [
        session.screen.get().width.get(),
        session.screen.get().height.get(),
    ];
    let bar = loading_bar(w, h, shared.vars.get().bar_share.get());
    let scratch = crate::race::view_scratch::front_end();
    let polygon = &scratch.polygon;
    // The quad's colour (0x623DE8); x at +0x28 / +0x2C, y at +0x38 / +0x3C.
    scratch.quad.corners[0].set(bar.left);
    scratch.quad.colour[0].set(BAR_COLOUR);
    scratch.quad.corners[4].set(bar.top);
    scratch.quad.corners[5].set(bar.bottom);
    scratch.quad.corners[1].set(bar.right);
    polygon.texture_page.set(0xFFFF_FFFFu32);
    crate::race::frame_draw::draw_screen_quad(
        polygon,
        crate::workers::game_scratch(),
        &scratch.face,
        crate::workers::DrawReads::on_game_thread(crate::workers::LentOwners {
            race,
            scene,
            system,
        }),
        session.screen.get(),
        scratch,
    )
}
