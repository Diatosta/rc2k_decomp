//! `race::draw_viewer_item` (0x41D94A): one item of the replay viewer's list drawn, unless
//! hidden (bit 6): its picture (`[esi+0xC]`, a texture rectangle; for the three switch
//! items, `off_5C2C70` by the block's camera byte 2 for item kind 0xE or 3 otherwise,
//! `off_5C2C78` by byte 5, `off_5C2C88` by byte 4, the table's entry) as a quad
//! (`drive::tyre_quad`) on the item's place (`[esi+4]`: x, y, w, h) at the picture's
//! width (its absolute value), centred (`dword_5C29FC`); grey (0x666666) when marked
//! unusable (bit 7), else white, or for the highlighted item (ecx `dword_51D407`) red
//! while a camera is edited, else yellow and red by turns (bit 3 of `dword_51D41F -
//! dword_51D423`).
//!
//! In:  esi the item (16 bytes), ecx its number
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_frame` (each item).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::install::vars::INSTALLER_PAGES;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::race::viewer_state_store::{REPLAY_VIEW, ReplayViewBlock, ViewerPicture};
use crate::session::Dword;

fn camera_byte(byte: usize) -> Result<u32, BadInput> {
    Ok(crate::race::viewer::camera_byte_at(
        byte.wrapping_add(8 * REPLAY_VIEW.replay_block.get() as usize),
    )? as u32)
}

/// The Rust version. `item` the list item (the original's esi: its 16 bytes), `index` its number
/// (ecx). The original keeps every register.
pub fn draw_viewer_item(item: &[Dword; 4], index: u32) -> Result<(), Refused> {
    // Its first dword's low bytes: the flags and the switch.
    let byte = |n: u32| (item[0].get() >> (8 * n)) as u8;
    // Converted: `rc2k_core::race::viewer::{switch_byte, centred, item_colour}`.
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    let picture = match ReplayViewBlock::item_picture(index)? {
        ViewerPicture::Rectangle(picture) => picture,
        ViewerPicture::Switch(0) => {
            let n = camera_byte(switch_byte(byte(1)) as usize)?;
            VIEWER_PANEL.switch_picture(0, n)?
        }
        ViewerPicture::Switch(1) => VIEWER_PANEL.switch_picture(1, camera_byte(5)?)?,
        ViewerPicture::Switch(_) => VIEWER_PANEL.switch_picture(2, camera_byte(4)?)?,
    };
    let uv: [u32; 4] = picture.read("a viewer item's picture")?;
    let place: [u32; 4] = ReplayViewBlock::item_place(index)?.read("a viewer item's place")?;
    let width = (uv[2] as i32).unsigned_abs();
    VIEWER_PANEL.item_place[2].set(width); // dword_5C29FC: the quad's place
    VIEWER_PANEL.item_place[0].set(centred(place[0], place[2], width));
    VIEWER_PANEL.item_place[1].set(place[1]);
    VIEWER_PANEL.item_place[3].set(place[3]);
    let colour = item_colour(
        byte(0),
        index == INSTALLER_PAGES.choice.get(),
        || {
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .camera_edit
                .get()
                != 0
        },
        || {
            INSTALLER_PAGES
                .frame_clock
                .get()
                .wrapping_sub(INSTALLER_PAGES.move_clock.get())
        },
    );
    crate::drive::tyre_quad::tyre_quad(uv, &VIEWER_PANEL.item_place, colour)
}
