//! What each control of the controller buttons page's shown page is bound to, and the rows' own
//! items: "no binding" (string 0x3BD), a button's number, a hat direction's name, or an axis's
//! name with its side. The row being learnt is left out. Refuses no current player.

use crate::errors::{BadInput, Refused};
use crate::screens::game_screen::ControlsScreen;
use crate::text::draw_text::Canvas;

/// The directions' names (`ControlsScreen.stick_names[0]`).
fn direction_names(
    controls: &crate::screens::controls_store::ControlsScreen,
) -> rc2k_core::screens::controls::DirectionNames {
    rc2k_core::screens::controls::DirectionNames {
        pad: controls.pad(),
        stick: controls.stick(),
    }
}

/// The size of a button picture on the page: the picture's square, which the outline nearly
/// fills.
const PICTURE: i32 = 32;
/// Where the pictures go: the bindings' x, and the first row's text y (the picture centres on the
/// text's capitals), a row down at a time.
const PICTURE_X: i32 = 0x1B8;
const TEXT_Y: i32 = 0xD5;
const ROW_STEP: i32 = 0x1B;

/// The picture `row`'s binding is shown as (a menu control still on its default shows that
/// default): a button or a hat direction of a controller whose
/// player has its pictures on, or an axis of a gamepad (a stick pushed one way, along its whole
/// axis, or a trigger). The axes of other controllers, whose meaning is not known, are told in
/// words.
fn binding_picture(
    o: crate::owners::Owners<'static>,
    player: crate::menu::standings::PlayerCell,
    row: rc2k_core::input::profile::Row,
) -> Option<(
    rc2k_core::input::icons::Family,
    rc2k_core::input::icons::Icon,
)> {
    use rc2k_core::input::profile::{Shown, shown_for};
    let current = o.menus.screens.event.read().current_player;
    let controller = player.get().controller;
    let (pictures, _) = crate::input::profiles::pictures(o.input, current, controller)?;
    let shown = crate::input::profiles::with_profile(o.input, current, controller, |p| {
        // A menu control the profile leaves unbound shows the default the menus take.
        shown_for(p, row, pictures.kind)
    })?;
    let icon = match shown {
        Shown::Button(number) => pictures.digital(rc2k_core::input::profile::Digital::Button {
            button: (number - 1) as u8,
        }),
        Shown::Hat(direction) => {
            pictures.digital(rc2k_core::input::profile::Digital::Hat { hat: 0, direction })
        }
        Shown::Axis(axis, side) if pictures.kind == rc2k_core::input::profile::Kind::Pad => {
            rc2k_core::input::icons::Icon::of_pad_axis(axis, side)
        }
        _ => None,
    }?;
    crate::gfx::button_pictures::has(pictures.family, icon).then_some((pictures.family, icon))
}

/// The text of `row` (without its NUL): "no binding" (string 0x3BD), a button's name (string
/// 0x3C0) and number, a hat direction's name, or an axis as its short name, "(+x)" with its side
/// (none for a centred axis, the steering).
pub fn binding_text(
    o: crate::owners::Owners<'static>,
    controls: &crate::screens::controls_store::ControlsScreen,
    player: crate::menu::standings::PlayerCell,
    row: rc2k_core::input::profile::Row,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<Vec<u8>, Refused> {
    use rc2k_core::input::profile::{HatDirection, Kind, Shown, shown_for};
    let current = o.menus.screens.event.read().current_player;
    let controller = player.get().controller;
    let sdl_kind = crate::input::profiles::controller_kind(o.input, controller).map(|(k, _)| k);
    let (profile_shown, kind) = if controller == 0 {
        (Shown::Nothing, Kind::Joystick)
    } else {
        crate::input::profiles::with_profile(o.input, current, controller, |p| {
            let kind = if p.wheel { Kind::Wheel } else { Kind::Joystick };
            // The menu controls' defaults depend on whether SDL reads it as a gamepad.
            (shown_for(p, row, sdl_kind.unwrap_or(kind)), kind)
        })
        .unwrap_or((Shown::Nothing, Kind::Joystick))
    };
    let pad = sdl_kind.unwrap_or(kind);
    let mut canvas = crate::text::draw_text::GameCanvas::new(gfx);
    let mut s = ControlsScreen::new(&mut canvas, o);
    use rc2k_core::screens::screen::Screen;
    Ok(match profile_shown {
        Shown::Nothing => s.string_bytes(0x3BD),
        Shown::Button(number) => {
            let mut out = s.string_bytes(0x3C0);
            out.push(b' ');
            out.extend(rc2k_core::strings::format::decimal(
                number,
                rc2k_core::strings::format::Pad::None,
            ));
            out
        }
        Shown::Hat(direction) => {
            let names = direction_names(controls).pad;
            let code = match direction {
                HatDirection::Up => 1,
                HatDirection::Down => 2,
                HatDirection::Left => 4,
                HatDirection::Right => 8,
            };
            s.string_bytes(names[code] as u32)
        }
        Shown::Axis(axis, side) => {
            let mut out = b"(".to_vec();
            match side {
                Some(true) => out.push(b'+'),
                Some(false) => out.push(b'-'),
                None => {}
            }
            out.extend(rc2k_core::input::profile::axis_label(pad, axis).bytes());
            out.push(b')');
            out
        }
    })
}

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_control_bindings(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let controls = &menus.screens.controls;
    let Some(player) =
        crate::menu::player::player(menus, menus.screens.event.read().current_player)
    else {
        return Err(BadInput("no current player").into());
    };
    let learning: u32 = controls.learning.get();
    let learning_row: u32 = controls.learned.get();
    use crate::menu::axes_items::{
        ROWS_PER_PAGE, page_shown, row_at, row_label, set_look, shifter_on, shown_look,
    };
    use rc2k_core::strings::own::id;
    // The rows of the page shown: each item's label, hidden past the last row, and the binding's
    // text beside it (not for the row being learnt, whose prompt is drawn instead).
    let mut texts: [Vec<u8>; ROWS_PER_PAGE] = Default::default();
    let mut learnt = None;
    let mut pictures = Vec::new();
    let shifter = shifter_on(menus, o.input);
    for (slot, text) in texts.iter_mut().enumerate() {
        let row = row_at(controls, shifter, slot);
        let label = row.map_or(id::BLANK, row_label);
        set_look(menus, 2 + slot as u32, shown_look(label), row.is_none())?;
        if let Some(row) = row {
            let learning_this = learning == 1 && row.control() == learning_row;
            if learning_this {
                learnt = Some(slot);
            } else if let Some(picture) = binding_picture(o, player, row) {
                pictures.push((slot, picture));
            }
            if pictures.last().is_none_or(|&(s, _)| s != slot) {
                *text = binding_text(o, controls, player, row, gfx)?;
            }
        }
    }
    // "more controls" on every page but the last, which turns back to the first.
    let last = crate::menu::axes_items::page_count(shifter) - 1;
    let more = if page_shown(controls, shifter) < last {
        id::MORE_CONTROLS
    } else {
        id::FIRST_CONTROLS
    };
    set_look(menus, 2 + ROWS_PER_PAGE as u32, shown_look(more), false)?;
    // The logic is in `rc2k_core::screens::controls::control_bindings`; a binding shown as a
    // picture has no text, and its picture is drawn after the texts.
    let too_wide = rc2k_core::screens::controls::control_bindings(
        &mut ControlsScreen::new(canvas, o),
        &texts,
        learnt,
    )?;
    for (slot, (family, icon)) in pictures {
        crate::gfx::button_hints::draw_beside_text(
            o,
            family,
            icon,
            PICTURE_X,
            TEXT_Y + ROW_STEP * slot as i32,
            PICTURE,
        );
    }
    Ok(too_wide)
}
