//! The dialog the replay viewer opens before it renders a video: a panel in the error dialog's
//! style with rows for the video's size, frame rate, part of the replay, quality and sound, the
//! file it will be written to, and the buttons "render" and "cancel". Up and Down choose a row,
//! Left and Right (or Enter) change its value, Escape cancels; a click chooses a row or presses
//! a button. The logic is `rc2k_core::race::render_dialog`; this file draws it and runs its
//! loop on the game's thread until it is answered.
//!
//! What was chosen last is kept for the next time in `var/rc2k.toml` (`display.video`).

use crate::gfx::typefaces::fonts;
use crate::input::osk_draw::fill;
use crate::owners::Owners;
use crate::text::draw_text::{GameCanvas, draw_text_at};
use rc2k_core::race::render_dialog::{Dialog, Key, Outcome, Part, Row, Settings};
use rc2k_core::strings::own::id;
use std::sync::Mutex;

/// The choices as last made (and read from the options file).
static KEPT: Mutex<Option<rc2k_core::options::model::VideoChoices>> = Mutex::new(None);

/// What the options file keeps of the dialog.
pub fn kept() -> rc2k_core::options::model::VideoChoices {
    KEPT.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default()
}

/// The choices the options file held.
pub fn set_kept(choices: rc2k_core::options::model::VideoChoices) {
    *KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(choices);
}

/// The sizes the graphics page lists: its own list when the page has made it, else the one the
/// page would make (the game's sizes, the monitor's modes, the usual sizes that fit the desktop
/// and the one set by hand), smallest first.
pub fn page_sizes(o: Owners<'static>) -> Vec<(u32, u32)> {
    let choices = &o.gfx.display.choices;
    let mut sizes: Vec<(u32, u32)> = Vec::new();
    if choices.listed.get() == 1 {
        for n in 0..choices.resolutions.get() {
            let (Ok(w), Ok(h)) = (
                choices.list.dword(n, 0, "a screen mode"),
                choices.list.dword(n, 1, "a screen mode"),
            ) else {
                break;
            };
            if w > 0 && h > 0 && !sizes.contains(&(w, h)) {
                sizes.push((w, h));
            }
        }
    } else {
        let modes = &o.gfx.display.modes;
        let table: Vec<(u32, u32)> = (0..8)
            .map(|n| {
                let size = modes.sizes[2 * n + 1].get();
                (size & 0xFFFF, size >> 16)
            })
            .filter(|&(w, h)| w > 0 && h > 0)
            .collect();
        let (found, desktop) = crate::platform::monitor_modes();
        let monitor = rc2k_core::gfx::resolutions::modes_as_sizes(&found);
        sizes = table.clone();
        sizes.extend(rc2k_core::gfx::resolutions::extra_sizes(
            &table,
            &monitor,
            desktop,
            crate::gfx::monitor_sizes::custom(),
        ));
    }
    rc2k_core::gfx::resolutions::sort(&mut sizes);
    sizes.dedup();
    sizes
}

/// The frame rates a video offers on this display.
pub fn page_rates() -> Vec<u32> {
    let refresh = crate::platform::pacing::display_refresh();
    rc2k_core::race::render_dialog::video_rates(refresh)
}

/// The dialog's colours (0xRRGGBB), as the error dialog's: the plain screen, the panel, a row and
/// the row with the highlight.
const SCREEN: u32 = 0x05081A;
const PANEL: u32 = 0x0C1238;
const ROW: u32 = 0x27347A;
const ROW_CHOSEN: u32 = 0xC8202A;
const MARGIN: i32 = 20;
/// The font's own top offset puts its letters a little below the y they are given.
const TEXT_UP: i32 = 12;

const KEY_ENTER: u8 = 0x1C;
const KEY_PAD_ENTER: u8 = 0x60;
const KEY_ESCAPE: u8 = 0x01;
const KEY_UP: u8 = 0x66;
const KEY_DOWN: u8 = 0x6B;
const KEY_LEFT: u8 = 0x68;
const KEY_RIGHT: u8 = 0x69;

fn key_of(code: u8) -> Option<Key> {
    match code {
        KEY_UP => Some(Key::Up),
        KEY_DOWN => Some(Key::Down),
        KEY_LEFT => Some(Key::Left),
        KEY_RIGHT => Some(Key::Right),
        KEY_ENTER | KEY_PAD_ENTER => Some(Key::Enter),
        KEY_ESCAPE => Some(Key::Escape),
        _ => None,
    }
}

/// The own text `id` in the language in use, as text.
fn own(o: Owners<'static>, id: u32) -> String {
    rc2k_core::files::ansi::ansi_text(&crate::strings::append_string_by_id::own_text(
        o.strings, id,
    ))
}

/// Where the video goes, as the dialog shows it.
pub struct Target<'a> {
    pub folder: &'a str,
    pub file: &'a str,
}

/// The dialog up until it is answered: the settings chosen to render with, or none when it was
/// cancelled (or the window closed, or there was no screen to draw on).
pub fn ask(o: Owners<'static>, marks: bool, target: &Target) -> Option<Settings> {
    let mut dialog = Dialog::new(
        Settings::from_kept(&kept()),
        marks,
        page_sizes(o),
        page_rates(),
    );
    let encoder = crate::video::encoder_found();
    // Keys and pad answers from before are not answers.
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    let mut was_down = true;
    // A tap on a row or a button is a click there (`platform::touch_menu`).
    #[cfg(touch_screen)]
    let _taps = crate::platform::touch_menu::InstallerPage::begin();
    loop {
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() {
            return None;
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return None;
        }
        let (places, panel) = draw(o, &dialog, encoder, target);
        crate::platform::unlock_surface(o.gfx);
        if !crate::platform::show_frame_now(o) {
            return None;
        }
        crate::input::menu_pad::take_answer(o.input);
        let mut outcome = Outcome::Stay;
        if let Some(key) = key_of(crate::input::take_key::take_key(o.input)) {
            outcome = dialog.key(key);
        }
        // The mouse: a click on a row chooses it; on a value it also steps it, and on a button it
        // presses it.
        let (buttons, x, y) = crate::input::keys::unpack_mouse(&o.input.devices);
        // The rows are where the panel was laid out.
        let (x, y) = panel.point(x, y);
        let down = buttons & 1 != 0;
        if down
            && !was_down
            && let Some(row) = Dialog::row_at(&places, x, y)
        {
            dialog.select(row);
            outcome = dialog.key(Key::Enter);
        }
        was_down = down;
        #[cfg(touch_screen)]
        if let Some((tx, ty)) = crate::platform::touch_menu::take_installer_tap()
            && let Some(row) = Dialog::row_at(&places, tx as i32, ty as i32)
        {
            crate::platform::touch_menu::tap_used();
            dialog.select(row);
            outcome = tapped_outcome(&mut dialog, &places, row, tx);
        }
        match outcome {
            Outcome::Stay => {}
            Outcome::Render => {
                let chosen = dialog.settings;
                set_kept(chosen.to_kept());
                return Some(chosen);
            }
            Outcome::Cancel => return None,
        }
    }
}

/// What a tap on `row` at `x` does: a button is pressed; on a setting row a tap on the left
/// part steps it back and one on the rest steps it on.
#[cfg(any(touch_screen, test))]
fn tapped_outcome(dialog: &mut Dialog, places: &[(Row, [i32; 4])], row: Row, x: f32) -> Outcome {
    let key = match places.iter().find(|(r, _)| *r == row) {
        Some((_, [l, _, w, _])) if !matches!(row, Row::Render | Row::Cancel) => {
            if x < (*l + *w / 3) as f32 {
                Key::Left
            } else {
                Key::Right
            }
        }
        _ => Key::Enter,
    };
    dialog.key(key)
}

/// One frame of the dialog; where each row was drawn (for the mouse).
fn draw(
    o: Owners<'static>,
    dialog: &Dialog,
    encoder: bool,
    target: &Target,
) -> (Vec<(Row, [i32; 4])>, crate::gfx::ui_scale::Panel) {
    use crate::gfx::button_hints::width_of;
    let mut canvas = GameCanvas::new(o.gfx);
    let screen = o.system.session.screen.get();
    // Laid out on the virtual screen, enlarged to the real one at the end.
    let view = crate::gfx::ui_scale::View::now(o);
    let (w, h) = (view.width, view.height);
    // The background at the real size, the panel on the virtual screen.
    fill(o, [0, 0, view.screen.0, view.screen.1], SCREEN);
    let measure = |s: &str| {
        width_of(
            fonts::LIST,
            &rc2k_core::files::ansi::ansi_bytes(s),
            &mut GameCanvas::new(o.gfx),
        )
    };
    let panel_w = (w - 40).min(720);
    let inner = panel_w - 2 * MARGIN;
    // A short screen gets closer rows.
    let compact = h < 560;
    let (line_h, row_h) = if compact {
        (26, 32)
    } else if cfg!(touch_screen) {
        // Taller rows for a finger.
        (30, 46)
    } else {
        (30, 38)
    };
    // The buttons' height (more than a row's on a touch screen); the space around them.
    let button_h = if cfg!(touch_screen) && !compact {
        56
    } else {
        row_h - 6
    };
    let note: Vec<String> = if encoder {
        Vec::new()
    } else {
        rc2k_core::fatal::wrap_lines(&own(o, id::RENDER_NO_ENCODER), inner, &measure)
    };
    let keys = own(o, id::RENDER_KEYS);
    let keys_lines = rc2k_core::fatal::wrap_lines(&keys, panel_w - 20, &measure);
    let panel_h = (MARGIN
        + line_h * 2
        + row_h * 6
        + line_h * 2
        + note.len() as i32 * line_h
        + button_h
        + 6
        + line_h * keys_lines.len() as i32
        + MARGIN)
        .min(h - 10);
    let panel = view.panel(panel_w, panel_h);
    let (left, top) = panel.origin();
    fill(o, [left, top, panel_w, panel_h], PANEL);
    let put = |text: &str, x: i32, y: i32, canvas: &mut GameCanvas| {
        let mut bytes = rc2k_core::files::ansi::ansi_bytes(text);
        bytes.push(0);
        let _ = draw_text_at(fonts::LIST, &bytes[..], x, y - TEXT_UP, canvas);
    };
    let mut places = Vec::new();
    let s = &dialog.settings;
    put(
        &own(o, id::RENDER_TITLE),
        left + MARGIN,
        top + MARGIN + line_h / 2,
        &mut canvas,
    );
    let mut y = top + MARGIN + line_h * 2;
    let size_text = |(sw, sh): (u32, u32)| format!("{sw} x {sh}");
    let rows: [(Row, u32, String); 6] = [
        (
            Row::Size,
            id::RENDER_SIZE,
            match s.size {
                Some(size) => size_text(size),
                None => own(o, id::RENDER_SCREEN)
                    .replace("{}", &size_text((screen.width.get(), screen.height.get()))),
            },
        ),
        (Row::FrameRate, id::RENDER_FPS, s.fps.to_string()),
        (
            Row::Part,
            id::RENDER_PART,
            own(
                o,
                match s.part {
                    Part::Whole => id::RENDER_WHOLE,
                    Part::FromHere => id::RENDER_FROM_HERE,
                    Part::Marked => id::RENDER_MARKED,
                },
            ),
        ),
        (
            Row::Quality,
            id::RENDER_QUALITY,
            own(o, id::RENDER_QUALITIES + s.quality.index() as u32),
        ),
        (
            Row::Encoder,
            id::RENDER_ENCODER,
            own(o, id::RENDER_ENCODERS + s.encoder.index() as u32),
        ),
        (
            Row::Sound,
            id::RENDER_SOUND,
            own(o, if s.sound { id::ON } else { id::OFF }),
        ),
    ];
    for (row, label, value) in rows {
        let chosen = dialog.current() == row;
        fill(
            o,
            [left + MARGIN, y, inner, row_h - 6],
            if chosen { ROW_CHOSEN } else { ROW },
        );
        let ty = y + (row_h - 6) / 2;
        put(&own(o, label), left + MARGIN + 10, ty, &mut canvas);
        let shown = format!("< {value} >");
        put(
            &shown,
            left + MARGIN + inner - 10 - measure(&shown),
            ty,
            &mut canvas,
        );
        places.push((row, [left + MARGIN, y, inner, row_h - 6]));
        y += row_h;
    }
    // Where the video goes: shown, not chosen.
    for (label, value) in [
        (id::RENDER_FOLDER, target.folder),
        (id::RENDER_FILE, target.file),
    ] {
        let ty = y + line_h / 2;
        put(&own(o, label), left + MARGIN + 10, ty, &mut canvas);
        put(
            value,
            left + MARGIN + inner - 10 - measure(value),
            ty,
            &mut canvas,
        );
        y += line_h;
    }
    for line in &note {
        put(line, left + MARGIN + 10, y + line_h / 2, &mut canvas);
        y += line_h;
    }
    // The buttons share the line under the rows.
    let by = top + panel_h - MARGIN - line_h * keys_lines.len() as i32 - button_h - 6;
    let gap = if cfg!(touch_screen) { 24 } else { 10 };
    let half = (inner - gap) / 2;
    for (n, (row, label)) in [
        (Row::Render, id::RENDER_GO),
        (Row::Cancel, id::RENDER_CANCEL),
    ]
    .into_iter()
    .enumerate()
    {
        let x = left + MARGIN + n as i32 * (half + gap);
        fill(
            o,
            [x, by, half, button_h],
            if dialog.current() == row {
                ROW_CHOSEN
            } else {
                ROW
            },
        );
        let text = own(o, label);
        put(
            &text,
            x + (half - measure(&text)) / 2,
            by + button_h / 2,
            &mut canvas,
        );
        places.push((row, [x, by, half, button_h]));
    }
    let mut ky = top + panel_h - MARGIN - line_h * keys_lines.len() as i32 + line_h / 2;
    for line in &keys_lines {
        put(line, left + (panel_w - measure(line)) / 2, ky, &mut canvas);
        ky += line_h;
    }
    panel.finish(o, SCREEN);
    (places, panel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog() -> Dialog {
        Dialog::new(
            Settings::from_kept(&rc2k_core::options::model::VideoChoices::default()),
            false,
            vec![(800, 600), (1280, 720)],
            vec![30, 60],
        )
    }

    /// A tap presses a button, and steps a setting row back on its left part, on from the rest.
    #[test]
    fn taps_press_buttons_and_step_rows() {
        let places = [
            (Row::Size, [0, 0, 300, 40]),
            (Row::Render, [0, 100, 140, 50]),
            (Row::Cancel, [160, 100, 140, 50]),
        ];
        let mut d = dialog();
        d.select(Row::Render);
        assert_eq!(
            tapped_outcome(&mut d, &places, Row::Render, 10.0),
            Outcome::Render
        );
        d.select(Row::Cancel);
        assert_eq!(
            tapped_outcome(&mut d, &places, Row::Cancel, 200.0),
            Outcome::Cancel
        );
        let first = d.settings.size;
        d.select(Row::Size);
        assert_eq!(
            tapped_outcome(&mut d, &places, Row::Size, 250.0),
            Outcome::Stay
        );
        let on = d.settings.size;
        assert_eq!(
            tapped_outcome(&mut d, &places, Row::Size, 10.0),
            Outcome::Stay
        );
        assert_eq!(d.settings.size, first);
        assert_ne!(on, first);
    }
}
