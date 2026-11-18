//! The save code page on the options menu: the state's code as text on the left and as a QR
//! code on the right, and the buttons that copy it, take one in from the clipboard and read
//! one from a camera. A code that comes in is checked whole first; the player is asked, with
//! what is in it beside what is here, before anything is replaced.
//!
//! The text is shown from its first letter and scrolls (Up and Down) when it is longer than
//! the room: a code can be tens of thousands of letters long (every saved game adds some
//! thousands), and a small font would not make that readable.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::platform::ffi::sdl_clipboard;
use crate::save_code::apply::{self, Plan};
use crate::save_code::page_draw as draw;
use crate::save_code::{gather, scan};
use rc2k_core::saves::code::qr::{self, Modules};
use rc2k_core::saves::code::{PlayerState, Summary, Trim};
use rc2k_core::strings::own::id;

const KEY_ENTER: u8 = 0x1C;
const KEY_PAD_ENTER: u8 = 0x60;
const KEY_ESCAPE: u8 = 0x01;
const KEY_UP: u8 = 0x66;
const KEY_DOWN: u8 = 0x6B;
const KEY_LEFT: u8 = 0x68;
const KEY_RIGHT: u8 = 0x69;

const BUTTONS: [u32; 4] = [
    id::SAVE_CODE_COPY,
    id::SAVE_CODE_PASTE,
    id::SAVE_CODE_CAMERA,
    id::SAVE_CODE_BACK,
];
const COPY: usize = 0;
const PASTE: usize = 1;
const CAMERA: usize = 2;
const BACK: usize = 3;

/// What the page shows.
struct Page {
    /// The text of the whole state.
    text: String,
    /// What the QR code holds, and how much of the state it had to leave out.
    modules: Option<Modules>,
    trim: Trim,
    chosen: usize,
    /// The first line of the text shown.
    scroll: usize,
    can_camera: bool,
    /// What happened last, as a line under the text.
    message: String,
}

fn own(o: Owners<'static>, n: u32) -> String {
    rc2k_core::files::ansi::ansi_text(&crate::strings::append_string_by_id::own_text(o.strings, n))
}

fn own_with(o: Owners<'static>, n: u32, with: &str) -> String {
    own(o, n).replace("{}", with)
}

impl Page {
    fn new(o: Owners<'static>) -> Result<Self, Refused> {
        let state = gather::gather(o)?;
        let (qr_text, trim) = state.for_qr();
        Ok(Self {
            text: state.encode(),
            modules: qr::encode(&qr_text),
            trim,
            chosen: BACK,
            scroll: 0,
            can_camera: scan::available(),
            message: String::new(),
        })
    }

    fn usable(&self, button: usize) -> bool {
        match button {
            COPY | PASTE => sdl_clipboard::AVAILABLE,
            CAMERA => self.can_camera,
            _ => true,
        }
    }

    fn step_choice(&mut self, by: isize) {
        let mut at = self.chosen as isize;
        for _ in 0..BUTTONS.len() {
            at = (at + by).rem_euclid(BUTTONS.len() as isize);
            if self.usable(at as usize) {
                self.chosen = at as usize;
                return;
            }
        }
    }

    fn trim_note(&self) -> u32 {
        match self.trim {
            Trim::Whole => id::SAVE_CODE_QR_WHOLE,
            Trim::NoGames => id::SAVE_CODE_QR_NO_GAMES,
            Trim::NoLines => id::SAVE_CODE_QR_NO_LINES,
            Trim::NoRecords => id::SAVE_CODE_QR_NO_RECORDS,
            Trim::NoBestTimes => id::SAVE_CODE_QR_NO_BEST,
        }
    }
}

/// How many lines the notes under the text (or the message in their place) and the keys' line
/// may take.
const NOTE_LINES: i32 = 5;
const KEYS_LINES: i32 = 2;
/// The least room the text keeps beside the QR code.
const LEAST_TEXT: i32 = 250;

/// Where the parts of the page are, for a screen of this size and a QR code of `side` modules.
struct Layout {
    /// The text's panel, the QR code's box (a whole number of pixels to a module), the notes'
    /// column under the text.
    text: [i32; 4],
    code: [i32; 4],
    scale: usize,
    notes: [i32; 4],
    buttons: [[i32; 4]; 4],
    keys_y: i32,
}

fn layout(w: i32, h: i32, side: Option<usize>) -> Layout {
    let m = draw::MARGIN;
    let lh = draw::LINE_H;
    let keys_y = h - m - lh / 2 - (KEYS_LINES - 1) * lh;
    let buttons_y = keys_y - lh / 2 - m / 2 - draw::BUTTON_H;
    let top = m + lh + m / 2;
    let body_h = (buttons_y - m / 2 - top).max(lh);
    // The QR code takes as much of the right as whole pixels to a module allow.
    let room = body_h.min(w - 2 * m - m - LEAST_TEXT).max(0) as usize;
    let scale = side.and_then(|s| qr::module_pixels(s, room)).unwrap_or(1);
    let code_w = side.map_or(0, |s| ((s + 2 * qr::QUIET) * scale) as i32);
    let left_w = if side.is_some() {
        w - 3 * m - code_w
    } else {
        w - 2 * m
    };
    let notes_h = NOTE_LINES * lh;
    let text_h = (body_h - notes_h - m / 2).max(lh);
    let each = (w - 2 * m - 3 * (m / 2)) / 4;
    Layout {
        text: [m, top, left_w, text_h],
        code: [w - m - code_w, top, code_w, code_w],
        scale,
        notes: [m, top + text_h + m / 2, left_w, notes_h],
        buttons: std::array::from_fn(|n| {
            [
                m + n as i32 * (each + m / 2),
                buttons_y,
                each,
                draw::BUTTON_H,
            ]
        }),
        keys_y,
    }
}

/// `text` drawn in lines of the width of `rect`, from its top, as many as it holds.
fn paragraph(o: Owners<'static>, text: &str, rect: [i32; 4]) {
    let [x, y, w, h] = rect;
    let measure = |s: &str| draw::width(o, s);
    for (n, line) in rc2k_core::fatal::wrap_lines(text, w, &measure)
        .iter()
        .take((h / draw::LINE_H).max(0) as usize)
        .enumerate()
    {
        draw::text(o, line, x, y + n as i32 * draw::LINE_H + draw::LINE_H / 2);
    }
}

fn draw_page(o: Owners<'static>, page: &Page) {
    let (w, h) = draw::screen_size(o);
    let l = layout(w, h, page.modules.as_ref().map(|m| m.side));
    crate::input::osk_draw::fill(o, [0, 0, w, h], draw::SCREEN);
    draw::text(
        o,
        &own(o, id::SAVE_CODE_TITLE),
        draw::MARGIN,
        draw::MARGIN + draw::LINE_H / 2,
    );
    // The text, from its first letter.
    let [x, y, tw, th] = l.text;
    crate::input::osk_draw::fill(o, [x, y, tw, th], draw::PANEL);
    let per_line = draw::letters_per_line(o, tw - 2 * 8);
    let visible = (th / draw::LINE_H).max(1) as usize;
    for (n, line) in draw::lines_of(&page.text, per_line)
        .skip(page.scroll)
        .take(visible)
        .enumerate()
    {
        draw::text(
            o,
            line,
            x + 8,
            y + n as i32 * draw::LINE_H + draw::LINE_H / 2,
        );
    }
    if let Some(modules) = &page.modules {
        draw::qr_code(o, modules, l.code[0], l.code[1], l.scale);
    }
    // Under the text: its length and what the QR code holds, or what happened last in their
    // place (the next key brings them back).
    let [nx, ny, nw, nh] = l.notes;
    if page.message.is_empty() {
        let length = own_with(o, id::SAVE_CODE_LENGTH, &page.text.len().to_string());
        let first = format!("{length}. {}", own(o, page.trim_note()));
        paragraph(o, &first, [nx, ny, nw, nh]);
    } else {
        paragraph(o, &page.message, [nx, ny, nw, nh]);
    }
    for (n, rect) in l.buttons.iter().enumerate() {
        draw::button(
            o,
            *rect,
            &own(o, BUTTONS[n]),
            page.chosen == n,
            page.usable(n),
        );
    }
    let keys_top = l.keys_y - draw::LINE_H / 2;
    let measure = |s: &str| draw::width(o, s);
    for (n, line) in
        rc2k_core::fatal::wrap_lines(&own(o, id::SAVE_CODE_KEYS), w - 2 * draw::MARGIN, &measure)
            .iter()
            .take(KEYS_LINES as usize)
            .enumerate()
    {
        draw::centred(
            o,
            line,
            0,
            w,
            keys_top + n as i32 * draw::LINE_H + draw::LINE_H / 2,
        );
    }
}

/// One frame: drawn on the locked screen and shown. False when the window is gone.
fn frame(o: Owners<'static>, draw_it: impl FnOnce()) -> bool {
    crate::platform::pump_messages_now(o);
    if o.system.session.flags.quitting.is_set() {
        return false;
    }
    if o.gfx.locked.surface.get() == 1 {
        crate::platform::unlock_surface(o.gfx);
    }
    if crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return false;
    }
    draw_it();
    crate::platform::unlock_surface(o.gfx);
    crate::platform::show_frame_now(o)
}

fn take_key(o: Owners<'static>) -> u8 {
    crate::input::menu_pad::take_answer(o.input);
    crate::input::take_key::take_key(o.input)
}

/// The page, until the player goes back.
pub fn run(o: Owners<'static>) -> Result<(), Refused> {
    let mut page = Page::new(o)?;
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    crate::input::typed_text::osk_close();
    loop {
        if !frame(o, || draw_page(o, &page)) {
            return Ok(());
        }
        let (w, h) = draw::screen_size(o);
        let l = layout(w, h, page.modules.as_ref().map(|m| m.side));
        let visible = (l.text[3] / draw::LINE_H).max(1) as usize;
        let per_line = draw::letters_per_line(o, l.text[2] - 16);
        let lines = page.text.len().div_ceil(per_line);
        let jump = visible.saturating_sub(1).max(1);
        let key = take_key(o);
        if key != 0 {
            page.message.clear();
        }
        match key {
            KEY_ESCAPE => return Ok(()),
            KEY_LEFT => page.step_choice(-1),
            KEY_RIGHT => page.step_choice(1),
            KEY_UP => page.scroll = page.scroll.saturating_sub(jump),
            KEY_DOWN => page.scroll = (page.scroll + jump).min(lines.saturating_sub(visible)),
            KEY_ENTER | KEY_PAD_ENTER => match page.chosen {
                BACK => return Ok(()),
                COPY => {
                    page.message = if sdl_clipboard::set_text(&page.text) {
                        own(o, id::SAVE_CODE_COPIED)
                    } else {
                        own(o, id::SAVE_CODE_CLIPBOARD_EMPTY)
                    };
                }
                PASTE => match sdl_clipboard::text() {
                    Some(text) => take_in(o, &mut page, &text)?,
                    None => page.message = own(o, id::SAVE_CODE_CLIPBOARD_EMPTY),
                },
                CAMERA => match scan::run(o) {
                    Ok(Some(text)) => take_in(o, &mut page, &text)?,
                    Ok(None) => page.message.clear(),
                    Err(n) => page.message = own(o, n),
                },
                _ => {}
            },
            _ => {}
        }
    }
}

/// A code that came in: checked whole, then the question, then the replacing.
fn take_in(o: Owners<'static>, page: &mut Page, text: &str) -> Result<(), Refused> {
    let checked = PlayerState::decode(text)
        .map_err(apply::Problem::from)
        .and_then(|state| apply::plan(o, state));
    let plan = match checked {
        Ok(plan) => plan,
        Err(e) => {
            page.message = own_with(o, id::SAVE_CODE_BAD_CODE, &e.to_string());
            return Ok(());
        }
    };
    if !ask(o, &plan) {
        page.message.clear();
        return Ok(());
    }
    match apply::commit(o, plan) {
        Ok(()) => {
            *page = Page::new(o)?;
            page.message = own(o, id::SAVE_CODE_DONE);
        }
        Err(e) => {
            page.message = own_with(o, id::SAVE_CODE_FAILED, &format!("{e:?}"));
        }
    }
    Ok(())
}

/// A date (seconds from 1970) as year-month-day, hours and minutes, in UTC (the game's own
/// time zone is not known to a code made elsewhere).
fn date_text(seconds: u32) -> String {
    let days = i64::from(seconds / 86_400);
    let rest = seconds % 86_400;
    // Days to a civil date.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        rest / 3600,
        rest % 3600 / 60
    )
}

/// The rows of the question: (label, in the code, now).
fn rows(o: Owners<'static>, code: &Summary, now: &Summary) -> Vec<(String, String, String)> {
    let missing = own(o, id::SAVE_CODE_NOT_HELD);
    let games = |s: &Summary| {
        s.games.as_ref().map(|g| {
            let names: Vec<String> = g
                .iter()
                .map(|(slot, name)| format!("{}: {name}", slot + 1))
                .collect();
            if names.is_empty() {
                "0".to_owned()
            } else {
                format!("{} ({})", names.len(), names.join(", "))
            }
        })
    };
    let count = |n: Option<usize>| n.map(|n| n.to_string());
    let players = |s: &Summary| (!s.players.is_empty()).then(|| s.players.join(", "));
    let row = |label: u32, a: Option<String>, b: Option<String>| {
        (
            own(o, label),
            a.unwrap_or_else(|| missing.clone()),
            b.unwrap_or_else(|| missing.clone()),
        )
    };
    vec![
        row(
            id::SAVE_CODE_ROW_MADE,
            Some(date_text(code.made)),
            Some(date_text(now.made)),
        ),
        row(
            id::SAVE_CODE_ROW_PROGRAM,
            Some(code.program.clone()),
            Some(now.program.clone()),
        ),
        row(id::SAVE_CODE_ROW_PLAYERS, players(code), players(now)),
        row(
            id::SAVE_CODE_ROW_RUNS,
            count(code.runs.map(|r| r as usize)),
            count(now.runs.map(|r| r as usize)),
        ),
        row(
            id::SAVE_CODE_ROW_BEST,
            count(code.best_times),
            count(now.best_times),
        ),
        row(
            id::SAVE_CODE_ROW_SETUPS,
            count(code.setups),
            count(now.setups),
        ),
        row(id::SAVE_CODE_ROW_GAMES, games(code), games(now)),
    ]
}

fn draw_question(o: Owners<'static>, rows: &[(String, String, String)], chosen: usize) {
    let (w, h) = draw::screen_size(o);
    let m = draw::MARGIN;
    crate::input::osk_draw::fill(o, [0, 0, w, h], draw::SCREEN);
    let panel_w = w - 2 * m;
    crate::input::osk_draw::fill(o, [m, m, panel_w, h - 2 * m], draw::PANEL);
    let inner = panel_w - 2 * m;
    let label_w = inner * 3 / 10;
    let col_w = (inner - label_w) / 2;
    let (c1, c2) = (2 * m + label_w, 2 * m + label_w + col_w);
    let mut y = 2 * m + draw::LINE_H / 2;
    draw::text(o, &own(o, id::SAVE_CODE_CONFIRM_TITLE), 2 * m, y);
    y += 2 * draw::LINE_H;
    draw::text(o, &own(o, id::SAVE_CODE_IN_CODE), c1, y);
    draw::text(o, &own(o, id::SAVE_CODE_NOW), c2, y);
    y += draw::LINE_H;
    for (label, code, now) in rows {
        draw::text(o, &draw::fit(o, label, label_w - m / 2), 2 * m, y);
        draw::text(o, &draw::fit(o, code, col_w - m / 2), c1, y);
        draw::text(o, &draw::fit(o, now, col_w - m / 2), c2, y);
        y += draw::LINE_H;
    }
    paragraph(
        o,
        &own(o, id::SAVE_CODE_NOTE),
        [2 * m, y + m / 2, inner, 3 * draw::LINE_H],
    );
    let keys_y = h - 2 * m - draw::LINE_H / 2;
    let ways_y = keys_y - draw::LINE_H / 2 - m / 2 - 2 * draw::BUTTON_H;
    for (n, label) in [id::SAVE_CODE_YES, id::SAVE_CODE_NO]
        .into_iter()
        .enumerate()
    {
        draw::button(
            o,
            [
                2 * m,
                ways_y + n as i32 * draw::BUTTON_H,
                inner,
                draw::BUTTON_H - 6,
            ],
            &own(o, label),
            chosen == n,
            true,
        );
    }
    let keys = draw::fit(o, &own(o, id::SAVE_CODE_CONFIRM_KEYS), inner);
    draw::centred(o, &keys, 0, w, keys_y);
}

/// The question: true if the player says to replace.
fn ask(o: Owners<'static>, plan: &Plan) -> bool {
    let rows = rows(o, &plan.code, &plan.now);
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    let mut chosen = 1;
    loop {
        if !frame(o, || draw_question(o, &rows, chosen)) {
            return false;
        }
        match take_key(o) {
            KEY_UP => chosen = chosen.saturating_sub(1),
            KEY_DOWN => chosen = (chosen + 1).min(1),
            KEY_ENTER | KEY_PAD_ENTER => return chosen == 0,
            KEY_ESCAPE => return false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::date_text;

    #[test]
    fn dates_are_civil_dates() {
        assert_eq!(date_text(0), "1970-01-01 00:00");
        assert_eq!(date_text(1_700_000_000), "2023-11-14 22:13");
        assert_eq!(date_text(951_782_400), "2000-02-29 00:00");
    }
}
