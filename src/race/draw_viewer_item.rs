//! `race::draw_viewer_item` (0x41D94A): one item of the replay viewer's list drawn, unless hidden
//! (bit 6): its picture as a quad (`drive::tyre_quad`) on the item's place at the picture's width
//! (its absolute value), centred (`item_place`); grey (0x666666) when marked unusable (bit 7),
//! else white, or for the highlighted item red while a camera is edited, else yellow and red by
//! turns (bit 3 of `frame_clock` - `move_clock`).
//!
//! Takes: the item (16 bytes), its number
//!
//! Caller: `race::viewer_frame` (each item).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::install::owner::Install;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::viewer_state_store::{ReplayViewBlock, ViewerPicture};
use crate::session::Dword;

fn camera_byte(replay: &Replay, byte: usize) -> Result<u32, BadInput> {
    Ok(crate::race::viewer::camera_byte_at(
        byte.wrapping_add(8 * replay.replay_view.replay_block.get() as usize),
    )? as u32)
}

/// `item` the list item, `index` its number.
pub fn draw_viewer_item(
    race: &Race,
    install: &Install,
    item: &[Dword; 4],
    index: u32,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let replay = &race.replay;
    let panel = race.viewer_panel();
    // Its first dword's low bytes: the flags and the switch.
    let byte = |n: u32| (item[0].get() >> (8 * n)) as u8;
    // Converted: `rc2k_core::race::viewer::{switch_byte, centred, item_colour}`.
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    let uv = match ReplayViewBlock::item_picture(index)? {
        ViewerPicture::Rectangle(ida) => panel.rectangle(ida)?,
        ViewerPicture::Switch(0) => {
            let n = camera_byte(replay, switch_byte(byte(1)) as usize)?;
            panel.switch_picture(0, n)?
        }
        ViewerPicture::Switch(1) => panel.switch_picture(1, camera_byte(replay, 5)?)?,
        ViewerPicture::Switch(_) => panel.switch_picture(2, camera_byte(replay, 4)?)?,
    };
    let bytes = ReplayViewBlock::item_place(index)?.read_array::<16>("a viewer item's place")?;
    let place: [u32; 4] =
        std::array::from_fn(|k| u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap()));
    let width = (uv[2] as i32).unsigned_abs();
    panel.item_place[2].set(width); // ViewerAndKinds.viewer_panel.item_place[0]: the quad's place
    panel.item_place[0].set(centred(place[0], place[2], width));
    panel.item_place[1].set(place[1]);
    panel.item_place[3].set(place[3]);
    let colour = item_colour(
        byte(0),
        index == install.pages.choice.get(),
        || race.viewer_buffers().camera_edit.get() != 0,
        || {
            install
                .pages
                .frame_clock
                .get()
                .wrapping_sub(install.pages.move_clock.get())
        },
    );
    crate::drive::tyre_quad::tyre_quad(
        uv,
        &panel.item_place,
        colour,
        panel,
        system,
        crate::workers::game_scratch().vertices,
    )
}
