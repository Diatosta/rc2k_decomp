//! `race::loading_bar_quad` (0x446861): the loading bar, a red (`dword_623DE8` 0xFF0000) flat
//! rectangle along the bottom: x from 0 to the screen's width times `dword_623700` (held to 0 ..
//! 100) over 100, y from 0x24D to 0x258 scaled from 600 to the screen's height; `dword_6240C4` -1;
//! `race::draw_screen_quad`.
//!
//! Caller: `race::loading_bar_frame`. The quad its Rust version.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn loading_bar_quad(race: &Race, system: &crate::system::owner::System) -> Result<(), Refused> {
    let shared = &race.shared;
    let session = &system.session;
    // Converted: `rc2k_core::race::loading::loading_bar`.
    use rc2k_core::race::loading::{BAR_COLOUR, loading_bar};
    let [w, h] = [session.screen.width.get(), session.screen.height.get()];
    let bar = loading_bar(w, h, shared.vars.bar_share.bits());
    // The quad's colour (0x623DE8); x at +0x28 / +0x2C, y at +0x38 / +0x3C.
    crate::race::view_scratch::front_end().quad.corners[0].set(bar.left);
    shared.vars.quad_colour[0].set(BAR_COLOUR);
    crate::race::view_scratch::front_end().quad.corners[4].set(bar.top);
    crate::race::view_scratch::front_end().quad.corners[5].set(bar.bottom);
    crate::race::view_scratch::front_end().quad.corners[1].set(bar.right);
    crate::scene::polygon_work::front_end()
        .texture_page
        .set_bits(0xFFFF_FFFFu32);
    crate::race::frame_draw::draw_screen_quad(
        crate::scene::polygon_work::front_end(),
        race,
        &crate::race::view_scratch::front_end().face,
        crate::scene::owner::scene(),
        crate::race::view_scratch::front_end(),
    )
}
