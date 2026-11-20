//! The controller axes pages' drawing and rows: the list shows an item (its label) for each axis
//! the controller has and hides the others, and draws the live bars and the wheel's values; one
//! axis's page draws the axis, its settings and the live bar
//! (`rc2k_core::screens::axes_page`).
//! Refuses no current player record, or no input device for a player that has one.

use crate::errors::Refused;
use crate::input::owner::Input;
use crate::menu::axes_items::{set_look, shown_look};
use crate::menu::item_at::item_at;
use crate::screens::controller::controller_page_refuses;
use crate::strings::append_string_by_id::own_text;
use rc2k_core::input::axes_page::{Column, MAX_DEAD_ZONE, ROWS, Words, position, rows, value_text};
use rc2k_core::input::profile::{Kind, Profile};
use rc2k_core::screens::axes_page::{AxisPage, List, ROW_SPACING, axes_list, axis_page};
use rc2k_core::screens::controller_page::{Font, Work};
use rc2k_core::strings::own::id;

type Canvas<'a> = &'a mut dyn crate::text::draw_text::Canvas;

/// The controller's kind and number of axes, the player's profile for it and its reading now;
/// none (with "none" drawn) when the controller is gone.
struct Shown {
    kind: Kind,
    axes: usize,
    profile: Profile,
    raw: [i32; 8],
}

fn shown(o: crate::owners::Owners<'static>, input: &Input) -> Result<Option<Shown>, Refused> {
    let menus = o.menus;
    let player = controller_page_refuses(menus, input)?;
    let controller = player.get().controller;
    let current = menus.screens.event.read().current_player;
    let kind = crate::input::profiles::controller_kind(input, controller);
    let profile = crate::input::profiles::with_profile(input, current, controller, |p| p.clone());
    let reading = crate::input::profiles::first_reading(input).unwrap_or_default();
    Ok(match (kind, profile) {
        (Some((kind, axes)), Some(profile)) => Some(Shown {
            kind,
            axes,
            profile,
            raw: reading.axes,
        }),
        _ => None,
    })
}

fn words(o: crate::owners::Owners<'static>) -> [Vec<u8>; 6] {
    let own = |n: u32| own_text(o.strings, n);
    [
        own(id::ON),
        own(id::OFF),
        own(id::DEGREES),
        own(id::LINEAR),
        own(id::GENTLE),
        own(id::QUICK),
    ]
}

fn none_note<'a>(o: crate::owners::Owners<'static>, canvas: Canvas<'a>) -> Result<(), Refused> {
    use rc2k_core::screens::screen::Screen;
    let mut screen = crate::screens::game_screen::GameScreen::<
        Font,
        Work,
        crate::screens::game_screen::HeldText,
    >::new(canvas, o);
    let none = rc2k_core::screens::notes::NO_CONTROLLER;
    screen
        .string(Font::Value, none.id, none.x, none.y)
        .map(|_| ())
}

/// The axes list: items 2 .. 7 are the axes' rows, 8 the wheel switch and 9 its full rotation;
/// the last two are drawn straight after the last axis row the controller has.
pub fn draw_axes_page(
    o: crate::owners::Owners<'static>,
    input: &Input,
    canvas: Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let Some(now) = shown(o, input)? else {
        for item in 2..=9 {
            set_look(menus, item, shown_look(id::BLANK), true)?;
            item_at(menus, item)?.raised.set(0);
        }
        return none_note(o, canvas);
    };
    let label_base = if now.kind == Kind::Pad {
        id::PAD_AXIS
    } else {
        id::JOYSTICK_AXIS
    };
    let shown_rows = rows(now.axes);
    let mut axes = Vec::new();
    for axis in 0..6u32 {
        let visible = (axis as usize) < shown_rows;
        set_look(menus, 2 + axis, shown_look(label_base + axis), !visible)?;
        if visible {
            axes.push(position(&now.profile, axis as u8, now.raw[axis as usize]));
        }
    }
    // The wheel's rows follow the last axis row shown, with no empty places between.
    let raised = (ROWS - shown_rows) as u32 * ROW_SPACING as u32;
    for item in [8, 9] {
        item_at(menus, item)?.raised.set(raised);
    }
    set_look(menus, 8, shown_look(id::WHEEL), false)?;
    set_look(menus, 9, shown_look(id::FULL_ROTATION), !now.profile.wheel)?;
    let own = |n: u32| own_text(o.strings, n);
    let list = List {
        axes,
        wheel: own(if now.profile.wheel { id::YES } else { id::NO }),
        rotation: now.profile.wheel.then(|| {
            let mut text = rc2k_core::strings::format::decimal(
                now.profile.full_rotation_degrees,
                rc2k_core::strings::format::Pad::None,
            );
            text.push(b' ');
            text.extend(own(id::DEGREES));
            text
        }),
    };
    let mut screen = crate::screens::game_screen::GameScreen::<
        Font,
        Work,
        crate::screens::game_screen::HeldText,
    >::new(canvas, o);
    axes_list(&mut screen, &list)
}

/// One axis's page: items 2 .. 7 are the axis, the four settings and the live position.
pub fn draw_axis_page(
    o: crate::owners::Owners<'static>,
    input: &Input,
    canvas: Canvas,
) -> Result<(), Refused> {
    let Some(now) = shown(o, input)? else {
        return none_note(o, canvas);
    };
    let axis = o
        .menus
        .screens
        .controls
        .axis
        .get()
        .min(rows(now.axes).max(1) as u32 - 1);
    let label_base = if now.kind == Kind::Pad {
        id::PAD_AXIS
    } else {
        id::JOYSTICK_AXIS
    };
    let [on, off, degrees, linear, gentle, quick] = words(o);
    let words = Words {
        on: &on,
        off: &off,
        degrees: &degrees,
        linear: &linear,
        gentle: &gentle,
        quick: &quick,
    };
    let settings = now.profile.settings(axis as u8);
    let rotation = now
        .profile
        .wheel
        .then_some(now.profile.full_rotation_degrees);
    // A wheel's range text (per cent and degrees) leaves no room for its slider.
    let levels = [
        Some(settings.dead_zone * 1000 / MAX_DEAD_ZONE),
        None,
        rotation.is_none().then_some(settings.range),
        None,
    ];
    let page = AxisPage {
        axis: own_text(o.strings, label_base + axis),
        values: Column::ALL.map(|c| value_text(&settings, c, rotation, &words)),
        levels,
        at: position(&now.profile, axis as u8, now.raw[axis as usize]),
    };
    let mut screen = crate::screens::game_screen::GameScreen::<
        Font,
        Work,
        crate::screens::game_screen::HeldText,
    >::new(canvas, o);
    axis_page(&mut screen, &page)
}
