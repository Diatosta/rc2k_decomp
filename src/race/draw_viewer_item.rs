//! Draws one item of the replay viewer's list, unless hidden (bit 6): its picture as a quad
//! (`drive::tyre_quad`) on the item's place at the picture's width (its absolute value), centred
//! (`item_place`). The colour is grey when the item is marked unusable (bit 7), else white, or for
//! the highlighted item red while a camera is edited, else yellow and red by turns (bit 3 of
//! `frame_clock` - `move_clock`). Called by `viewer_frame` for each item.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::memory::list_handle::Item;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::viewer_state_store::{ReplayViewBlock, ViewerPicture};

fn camera_byte(race: &Race, replay: &Replay, byte: usize) -> Result<u32, BadInput> {
    Ok(crate::race::viewer::camera_byte_at(
        race,
        byte.wrapping_add(8 * replay.replay_view().replay_block.get() as usize),
    )? as u32)
}

/// `item` the list item, `index` its number.
pub fn draw_viewer_item(
    race: &Race,
    list_state: &crate::menu::list_state::ListState,
    item: &Item,
    index: u32,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let replay = &race.replay;
    let panel = race.viewer_panel();
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    let uv = match ReplayViewBlock::item_picture(replay, index)? {
        ViewerPicture::Rectangle(ida) => panel.rectangle(ida)?,
        ViewerPicture::Switch(0) => {
            let n = camera_byte(race, replay, switch_byte(item.count.get()) as usize)?;
            panel.switch_picture(0, n)?
        }
        ViewerPicture::Switch(1) => panel.switch_picture(1, camera_byte(race, replay, 5)?)?,
        ViewerPicture::Switch(_) => panel.switch_picture(2, camera_byte(race, replay, 4)?)?,
    };
    let bytes =
        ReplayViewBlock::item_place(replay, index)?.read_array::<16>("a viewer item's place")?;
    let place: [u32; 4] =
        std::array::from_fn(|k| u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap()));
    let width = (uv[2] as i32).unsigned_abs();
    panel.item_place[2].set(width);
    panel.item_place[0].set(centred(place[0], place[2], width));
    panel.item_place[1].set(place[1]);
    panel.item_place[3].set(place[3]);
    let colour = item_colour(
        item.first.get(),
        index == list_state.choice.get(),
        || race.viewer_buffers().camera_edit.get() != 0,
        || {
            list_state
                .frame_clock
                .get()
                .wrapping_sub(list_state.move_clock.get())
        },
    );
    crate::drive::tyre_quad::tyre_quad(
        uv,
        &panel.item_place,
        colour,
        panel,
        system,
        scene.scratch.game_scratch(),
    )
}
