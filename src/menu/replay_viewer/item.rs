//! Draws one item of the replay viewer's list, unless hidden (bit 6): its picture as a quad
//! (`drive::tyre_quad`) on the item's place at the picture's width (its absolute value), centred
//! (`item_place`). The colour is grey when the item is marked unusable (bit 7), else white, or for
//! the highlighted item red while a camera is edited, else yellow and red by turns (bit 3 of
//! `frame_clock` - `move_clock`).

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::viewer_list_store::ViewerItem;
use crate::race::viewer_state_store::{ReplayViewBlock, SwitchTable, ViewerPicture};

fn camera_byte(race: &Race, replay: &Replay, byte: usize) -> Result<u32, BadInput> {
    Ok(crate::race::viewer::camera_byte_at(
        race,
        byte.wrapping_add(8 * replay.replay_view().replay_block.get() as usize),
    )? as u32)
}

/// `item` the list item, `index` its number.
pub(super) fn draw_viewer_item(
    race: &Race,
    list_state: &crate::menu::list_state::ListState,
    item: &ViewerItem,
    index: u32,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let replay = &race.replay;
    let panel = race.viewer_panel();
    use rc2k_core::race::viewer::{centred, item_colour, switch_byte};
    let uv = match ReplayViewBlock::item_picture(replay.replay_view(), index)? {
        ViewerPicture::Rectangle(offset) => panel.rectangle_at(offset)?,
        ViewerPicture::Switch(SwitchTable::A) => {
            let n = camera_byte(race, replay, switch_byte(index as u8) as usize)?;
            panel.switch_picture(SwitchTable::A.number(), n)?
        }
        ViewerPicture::Switch(SwitchTable::B) => {
            let layout = camera_byte(race, replay, 5)?;
            panel.switch_picture(
                SwitchTable::B.number(),
                rc2k_core::race::viewer::layout_picture(layout.min(0xFF) as u8),
            )?
        }
        ViewerPicture::Switch(SwitchTable::C) => {
            panel.switch_picture(SwitchTable::C.number(), camera_byte(race, replay, 4)?)?
        }
    };
    let bytes = ReplayViewBlock::item_place(replay.replay_view(), index)?
        .read_array::<16>("a viewer item's place")?;
    let place: [u32; 4] =
        std::array::from_fn(|k| u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap()));
    let width = (uv[2] as i32).unsigned_abs();
    panel.item_place[2].set(width);
    panel.item_place[0].set(centred(place[0], place[2], width));
    panel.item_place[1].set(place[1]);
    panel.item_place[3].set(place[3]);
    let colour = item_colour(
        item.first.get(),
        index == list_state.choice.get() && !crate::race::render_row::selected(race),
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
        rc2k_core::hud::anchor::Spread::Centred,
    )
}
