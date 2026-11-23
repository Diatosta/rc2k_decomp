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
    // The car picture of a layout showing the third to the eighth car: the first car's with its
    // digit replaced.
    let mut digit = None;
    let uv = match ReplayViewBlock::item_picture(replay.replay_view(), index)? {
        ViewerPicture::Rectangle(offset) => panel.rectangle_at(offset)?,
        ViewerPicture::Switch(SwitchTable::A) => {
            let n = camera_byte(race, replay, switch_byte(index as u8) as usize)?;
            panel.switch_picture(SwitchTable::A.number(), n)?
        }
        ViewerPicture::Switch(SwitchTable::B) => {
            let layout = camera_byte(race, replay, 5)?;
            digit = rc2k_core::race::viewer::layout_digit(layout.min(0xFF) as u8);
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
        index == list_state.read().choice && !crate::race::render_row::selected(race),
        || race.viewer_buffers().camera_edit.get() != 0,
        || {
            list_state
                .read()
                .frame_clock
                .wrapping_sub(list_state.read().move_clock)
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
    )?;
    if digit.is_none() {
        return Ok(());
    }
    // The picture's own digit is covered with a piece of the box's face in the item's colour; the
    // number itself is drawn over the finished frame (`draw_car_digit`).
    use rc2k_core::race::viewer::{DIGIT_PATCH, FACE_SAMPLE};
    let [x, y, ..] = uv;
    let face = [
        x + FACE_SAMPLE[0],
        y + FACE_SAMPLE[1],
        FACE_SAMPLE[2],
        FACE_SAMPLE[3],
    ];
    let [px, py, w, h] = DIGIT_PATCH;
    crate::drive::tyre_quad::quad_of(
        face,
        [
            panel.item_place[0].get().wrapping_add(px),
            panel.item_place[1].get().wrapping_add(py),
            w,
            h,
        ],
        colour,
        panel,
        system,
        scene.scratch.game_scratch(),
        rc2k_core::hud::anchor::Spread::Centred,
    )
}

/// The car item's index in the list (the item whose picture is the layout's).
const CAR_ITEM: u32 = 15;

/// The number of the car the car item shows, when its picture has none of its own: drawn in the
/// game's heavy font (the race's notice font), black, centred on the box of the first car's
/// picture whose own digit `draw_viewer_item` covered. Over the finished frame, as the button
/// names are (`tip`); the layout's own x is mapped to the screen as the item's quad is.
pub(super) fn draw_car_digit(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    use rc2k_core::hud::anchor::Spread;
    use rc2k_core::race::viewer::{centred, digit_spot, layout_digit};
    let race = o.race;
    let replay = &race.replay;
    let layout = camera_byte(race, replay, 5)?;
    let Some(digit) = layout_digit(layout.min(0xFF) as u8) else {
        return Ok(());
    };
    let panel = race.viewer_panel();
    let uv = panel.switch_picture(
        SwitchTable::B.number(),
        rc2k_core::race::viewer::layout_picture(0),
    )?;
    let bytes = ReplayViewBlock::item_place(replay.replay_view(), CAR_ITEM)?
        .read_array::<16>("a viewer item's place")?;
    let place: [u32; 4] =
        std::array::from_fn(|k| u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap()));
    let width = (uv[2] as i32).unsigned_abs();
    let (centre2, top) = digit_spot(centred(place[0], place[2], width), place[1]);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get(), screen.height.get());
    let (factor, shift) = Spread::Centred.across(w, h);
    // The glyph is centred across an area as wide as the glyph's own room, 48 units.
    let (area_w, centre) = (
        (f64::from(width) * factor).round() as i32,
        f64::from(centre2) / 2.0 * factor + shift,
    );
    let area = [
        (centre - f64::from(area_w) / 2.0).round() as i32,
        0,
        area_w,
        h as i32,
    ];
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return Ok(());
    }
    crate::gfx::notice_text::draw_in_tinted(o, &[b'0' + digit], area, top, 255, Some(0));
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
    Ok(())
}
