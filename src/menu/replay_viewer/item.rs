//! Draws one item of the replay viewer's list, unless it is absent (a camera button of a part the
//! layout does not have): its picture as a quad (`drive::tyre_quad`) on the item's place at the
//! picture's width (its absolute value), centred (`item_place`). The colour is grey when the item
//! is marked unusable (bit 7), else white, or for the highlighted item red while a camera is
//! edited, else yellow and red by turns (bit 3 of `frame_clock` - `move_clock`).
//!
//! The panel's pictures hold the numbers 1 and 2 only, and the car item's only the first two
//! players alone and together. Where an item needs more (the car item of a further player, pair
//! or group, a camera button that edits another player's camera) pieces of the atlas are laid over
//! its picture in the item's colour (a piece of plain face over the picture's own number, the
//! pair pictures' dividers), and the numbers are drawn over the finished frame (`draw_numbers`)
//! in the game's heavy font, black, as the button names are (`tip`);
//! `rc2k_core::race::viewer_digits` and `rc2k_core::race::car_item` hold what and where.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::viewer_list_store::ViewerItem;
use crate::race::viewer_state_store::{ReplayViewBlock, SwitchTable, ViewerPicture};
use rc2k_core::hud::anchor::Spread;
use rc2k_core::race::car_item::CarPicture;
use rc2k_core::race::viewer_digits::{Overlay, Source};

fn camera_byte(race: &Race, replay: &Replay, byte: usize) -> Result<u32, BadInput> {
    Ok(crate::race::viewer::camera_byte_at(
        race,
        byte.wrapping_add(8 * replay.replay_view().replay_block.get() as usize),
    )? as u32)
}

/// The car item's picture for the layout of the block shown.
fn car_picture(race: &Race, replay: &Replay) -> Result<CarPicture, BadInput> {
    let layout = camera_byte(race, replay, 5)?.min(0xFF) as u8;
    Ok(rc2k_core::race::car_item::car_picture(
        layout,
        crate::race::viewer::layout_shown(race, layout),
        crate::race::split_layout::index(),
    )?)
}

/// The item's picture (a rectangle of the panel's texture).
fn picture(race: &Race, replay: &Replay, index: u32) -> Result<[u32; 4], BadInput> {
    use rc2k_core::race::viewer::switch_byte;
    let panel = race.viewer_panel();
    match ReplayViewBlock::item_picture(replay.replay_view(), index)? {
        ViewerPicture::Rectangle(offset) => panel.rectangle_at(offset),
        ViewerPicture::Switch(SwitchTable::A) => {
            let n = camera_byte(race, replay, switch_byte(index as u8) as usize)?;
            panel.switch_picture(SwitchTable::A.number(), n)
        }
        ViewerPicture::Switch(SwitchTable::B) => {
            panel.switch_picture(SwitchTable::B.number(), car_picture(race, replay)?.base)
        }
        ViewerPicture::Switch(SwitchTable::C) => {
            panel.switch_picture(SwitchTable::C.number(), camera_byte(race, replay, 4)?)
        }
    }
}

/// The item's place on the 800 by 600 layout (left, top, width, height).
fn place(replay: &Replay, index: u32) -> Result<[u32; 4], BadInput> {
    let bytes = ReplayViewBlock::item_place(replay.replay_view(), index)?
        .read_array::<16>("a viewer item's place")?;
    Ok(std::array::from_fn(|k| {
        u32::from_le_bytes(bytes[4 * k..4 * k + 4].try_into().unwrap())
    }))
}

/// What is laid and drawn over the item's picture: for the car item its layout's
/// (`car_item::car_picture`), for the camera buttons that edit the camera of a player other than
/// their own (the first two buttons' numbers) or are the third and fourth the player's number in
/// place of the icon's.
fn overlay(race: &Race, replay: &Replay, index: u32) -> Result<Overlay, BadInput> {
    use rc2k_core::race::viewer::CAMERA_ITEMS;
    use rc2k_core::race::viewer_digits::camera_digit;
    if index == CAR_ITEM {
        return Ok(car_picture(race, replay)?.overlay);
    }
    let Some(button) = CAMERA_ITEMS.iter().position(|&item| item == index) else {
        return Ok(Overlay::default());
    };
    let layout = camera_byte(race, replay, 5)?.min(0xFF) as u8;
    let buttons = crate::race::viewer::camera_buttons(race, layout)?;
    Ok(buttons[button]
        .and_then(|b| camera_digit(button, b.player))
        .map_or_else(Overlay::default, |(number, own)| {
            Overlay::over_own(number, own)
        }))
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
    if !rc2k_core::race::viewer::item_drawn(index, item.first.get()) {
        return Ok(());
    }
    let replay = &race.replay;
    let panel = race.viewer_panel();
    use rc2k_core::race::viewer::{centred, item_colour};
    let uv = picture(race, replay, index)?;
    let place = place(replay, index)?;
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
        Spread::Centred,
    )?;
    // The pieces in the item's colour, each stretched from its rectangle of the atlas onto its
    // place on the picture.
    let (left, top) = (panel.item_place[0].get(), panel.item_place[1].get());
    for piece in overlay(race, replay, index)?.pieces {
        let [x, y, ..] = match piece.source {
            Source::Own => uv,
            Source::CarPicture(n) => panel.switch_picture(SwitchTable::B.number(), n)?,
        };
        let [rx, ry, rw, rh] = piece.rect;
        let [px, py, w, h] = piece.place;
        crate::drive::tyre_quad::quad_of(
            [x + rx, y + ry, rw, rh],
            [left.wrapping_add(px), top.wrapping_add(py), w, h],
            colour,
            panel,
            system,
            scene.scratch.game_scratch(),
            Spread::Centred,
        )?;
    }
    Ok(())
}

/// How far right and down, in screen pixels, the game's textured quads land from their geometry
/// (measured: the atlas' own digits sit half a pixel right of and below where the atlas puts
/// them); a number laid by the geometry is moved the same way to meet them.
const QUAD_OFFSET: f32 = 0.5;

/// The car item's index in the list (the item whose picture is the layout's).
const CAR_ITEM: u32 = 15;

/// The numbers of the items that carry any (`overlay`), drawn over the finished frame in the game's
/// heavy font (`gfx::notice_text`) as solid black shapes, each where its spot on the picture is.
/// The pictures lie where the items' quads are: the layout's x is mapped to the screen as the
/// quads' are, its y by the screen's height over 600.
pub(super) fn draw_numbers(
    o: crate::owners::Owners<'static>,
    list: &crate::race::viewer_list_store::ViewerList,
) -> Result<(), Refused> {
    use rc2k_core::race::viewer::{centred, item_drawn};
    let (race, replay) = (o.race, &o.race.replay);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get(), screen.height.get());
    let (factor, shift) = Spread::Centred.across(w, h);
    let down = h as f32 / 600.0;
    let mut numbers = Vec::new();
    for (index, item) in (0..).zip(&list.items) {
        if !item_drawn(index, item.first.get()) {
            continue;
        }
        let overlay = overlay(race, replay, index)?;
        if overlay.numbers.is_empty() {
            continue;
        }
        let place = place(replay, index)?;
        let width = (picture(race, replay, index)?[2] as i32).unsigned_abs();
        let (left, top) = (centred(place[0], place[2], width), place[1]);
        for (digit, spot) in overlay.numbers {
            let centre_x = ((f64::from(left) + f64::from(spot.centre_x)) * factor + shift) as f32
                + QUAD_OFFSET;
            let ink_top = (top as f32 + spot.top_y) * down + QUAD_OFFSET;
            let size = [spot.scale * factor as f32, spot.scale * down];
            numbers.push((digit, [centre_x, ink_top], size));
        }
    }
    if numbers.is_empty() {
        return Ok(());
    }
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return Ok(());
    }
    for (digit, at, size) in numbers {
        crate::gfx::notice_text::draw_shape(o, &[b'0' + digit], 0, at, size);
    }
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
    Ok(())
}
