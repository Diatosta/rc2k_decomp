//! The save code page on the options menu: the state's code as text on the left and as a QR
//! code on the right, and the buttons that copy it, take one in from the clipboard and read
//! one from a camera. A code that comes in is checked whole first; the player is asked, with
//! what is in it beside what is here, before anything is replaced.
//!
//! The text is shown from its first letter and scrolls (Page Up and Page Down) when it is
//! longer than the room: a code can be tens of thousands of letters long, and a small font
//! would not make that readable. A code longer than one QR code holds is shown as numbered
//! parts, one QR code at a time, stepped with Left and Right.
//!
//! The QR codes are the large ones (few, for a camera of 1080p or better) until the player
//! chooses the small ones (more, for a camera that cannot read the large); the same button goes
//! back. A screen too small to draw a large code at three pixels a module starts with the small
//! ones.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::platform::ffi::sdl_clipboard;
use crate::save_code::apply::{self, Plan};
use crate::save_code::page_draw as draw;
use crate::save_code::{gather, scan};
use rc2k_core::saves::code::parts::Density;
use rc2k_core::saves::code::qr::{self, Modules};
use rc2k_core::saves::code::{PlayerState, Summary, parts};
use rc2k_core::strings::own::id;

const KEY_ENTER: u8 = 0x1C;
const KEY_PAD_ENTER: u8 = 0x60;
const KEY_ESCAPE: u8 = 0x01;
const KEY_UP: u8 = 0x66;
const KEY_DOWN: u8 = 0x6B;
const KEY_LEFT: u8 = 0x68;
const KEY_RIGHT: u8 = 0x69;
const KEY_PAGE_UP: u8 = 0x67;
const KEY_PAGE_DOWN: u8 = 0x6C;
/// The shoulder buttons of a pad give this key (`Input::take_shoulder` says which).
const KEY_TAB: u8 = 0x0F;
/// The right stick scrolls a line, then again after this long, then every `STICK_EVERY`.
const STICK_AFTER: std::time::Duration = std::time::Duration::from_millis(400);
const STICK_EVERY: std::time::Duration = std::time::Duration::from_millis(70);

const BUTTONS: usize = 5;
const COPY: usize = 0;
const PASTE: usize = 1;
const CAMERA: usize = 2;
const DENSITY: usize = 3;
const BACK: usize = 4;
/// The fewest pixels a module of a large code takes for the page to start with large codes.
const LEAST_LARGE_SCALE: usize = 3;

/// What the page shows.
struct Page {
    /// The text of the whole state.
    text: String,
    /// The texts of the QR codes of the density shown, in order: the whole text when one holds
    /// it, else its parts. Empty when the state is too long for the most parts.
    codes: Vec<String>,
    density: Density,
    /// The codes of the other density.
    other: Vec<String>,
    /// Which of them the page shows, and its modules.
    shown: usize,
    modules: Option<Modules>,
    /// The side of the first code's modules: the box the codes are drawn in (the parts are
    /// of nearly one size).
    box_side: Option<usize>,
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
        let text = gather::gather(o)?.encode();
        let large = parts::split(&text, Density::Dense).unwrap_or_default();
        let small = parts::split(&text, Density::Small).unwrap_or_default();
        let (w, h) = draw::screen_size(o);
        let side = large.first().and_then(|t| qr::encode(t)).map(|m| m.side);
        let fits = layout(w, h, side).scale >= LEAST_LARGE_SCALE;
        let (codes, other, density) = if fits || small.is_empty() {
            (large, small, Density::Dense)
        } else {
            (small, large, Density::Small)
        };
        let mut page = Self {
            text,
            codes,
            density,
            other,
            shown: 0,
            modules: None,
            box_side: None,
            chosen: BACK,
            scroll: 0,
            can_camera: scan::available(),
            message: String::new(),
        };
        page.show(0);
        page.box_side = page.modules.as_ref().map(|m| m.side);
        Ok(page)
    }

    /// Makes code `n` the one shown.
    fn show(&mut self, n: usize) {
        self.shown = n;
        self.modules = self.codes.get(n).and_then(|t| qr::encode(t));
    }

    /// Switches between the large and the small codes, starting at the first.
    fn switch_density(&mut self) {
        std::mem::swap(&mut self.codes, &mut self.other);
        self.density = self.density.other();
        self.show(0);
        self.box_side = self.modules.as_ref().map(|m| m.side);
    }

    /// Steps the QR code shown by `by`, round the ends.
    fn step_code(&mut self, by: isize) {
        let n = self.codes.len() as isize;
        if n > 1 {
            self.show((self.shown as isize + by).rem_euclid(n) as usize);
        }
    }

    fn usable(&self, button: usize) -> bool {
        match button {
            COPY | PASTE => sdl_clipboard::AVAILABLE,
            CAMERA => self.can_camera,
            // Another density helps when it makes other codes.
            DENSITY => !self.other.is_empty() && self.other != self.codes,
            _ => true,
        }
    }

    fn step_choice(&mut self, by: isize) {
        let mut at = self.chosen as isize;
        for _ in 0..BUTTONS {
            at = (at + by).rem_euclid(BUTTONS as isize);
            if self.usable(at as usize) {
                self.chosen = at as usize;
                return;
            }
        }
    }

    /// What the line under the text says of the QR codes.
    fn codes_note(&self, o: Owners<'static>) -> String {
        let held = match self.codes.len() {
            0 => return own(o, id::SAVE_CODE_TOO_LONG),
            1 => own(o, id::SAVE_CODE_QR_WHOLE),
            n => own_with(o, id::SAVE_CODE_PARTS_NOTE, &n.to_string()),
        };
        if !self.usable(DENSITY) {
            return held;
        }
        let hint = match self.density {
            Density::Dense => id::SAVE_CODE_LARGE_NOTE,
            Density::Small => id::SAVE_CODE_SMALL_NOTE,
        };
        format!("{held} {}", own(o, hint))
    }

    /// The label of the density button: what it switches to.
    fn density_label(&self, o: Owners<'static>) -> String {
        own(
            o,
            match self.density {
                Density::Dense => id::SAVE_CODE_SMALL_BUTTON,
                Density::Small => id::SAVE_CODE_LARGE_BUTTON,
            },
        )
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
    buttons: [[i32; 4]; BUTTONS],
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
    let each = (w - 2 * m - (BUTTONS as i32 - 1) * draw::BUTTON_GAP) / BUTTONS as i32;
    Layout {
        text: [m, top, left_w, text_h],
        code: [w - m - code_w, top, code_w, code_w],
        scale,
        notes: [m, top + text_h + m / 2, left_w, notes_h],
        buttons: std::array::from_fn(|n| {
            [
                m + n as i32 * (each + draw::BUTTON_GAP),
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
    let l = layout(w, h, page.box_side);
    crate::input::osk_draw::fill(o, [0, 0, w, h], draw::SCREEN);
    draw::text(
        o,
        &own(o, id::SAVE_CODE_TITLE),
        draw::MARGIN,
        draw::MARGIN + draw::LINE_H / 2,
    );
    if page.codes.len() > 1 {
        let place = own(o, id::SAVE_CODE_PART_OF)
            .replacen("{}", &(page.shown + 1).to_string(), 1)
            .replacen("{}", &page.codes.len().to_string(), 1);
        draw::text(
            o,
            &place,
            w - draw::MARGIN - draw::width(o, &place),
            draw::MARGIN + draw::LINE_H / 2,
        );
    }
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
        // The first code is the largest, so every one fits the box at its scale.
        draw::qr_code(o, modules, l.code[0], l.code[1], l.scale);
    }
    // Under the text: its length and what the QR code holds, or what happened last in their
    // place (the next key brings them back).
    let [nx, ny, nw, nh] = l.notes;
    if page.message.is_empty() {
        let length = own_with(o, id::SAVE_CODE_LENGTH, &page.text.len().to_string());
        let first = format!("{length}. {}", page.codes_note(o));
        paragraph(o, &first, [nx, ny, nw, nh]);
    } else {
        paragraph(o, &page.message, [nx, ny, nw, nh]);
    }
    for (n, rect) in l.buttons.iter().enumerate() {
        let label = match n {
            COPY => own(o, id::SAVE_CODE_COPY),
            PASTE => own(o, id::SAVE_CODE_PASTE),
            CAMERA => own(o, id::SAVE_CODE_CAMERA),
            DENSITY => page.density_label(o),
            _ => own(o, id::SAVE_CODE_BACK),
        };
        let label = draw::fit(o, &label, rect[2] - 8);
        draw::button(o, *rect, &label, page.chosen == n, page.usable(n));
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
/// The page's sideways drags follow the finger (they step through its codes, not a carousel),
/// until the page ends.
struct DirectSideways;

impl DirectSideways {
    fn begin() -> Self {
        #[cfg(touch_screen)]
        crate::platform::touch_menu::set_direct_sideways(true);
        DirectSideways
    }
}

impl Drop for DirectSideways {
    fn drop(&mut self) {
        #[cfg(touch_screen)]
        crate::platform::touch_menu::set_direct_sideways(false);
    }
}

pub fn run(o: Owners<'static>) -> Result<(), Refused> {
    let _direct = DirectSideways::begin();
    // A tap on one of the page's buttons presses it (`platform::touch_menu`).
    #[cfg(touch_screen)]
    let _taps = crate::platform::touch_menu::InstallerPage::begin();
    let mut page = Page::new(o)?;
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    crate::input::typed_text::osk_close();
    // The way the right stick leans and when it scrolls next.
    let mut leaning = 0;
    let mut next_line = std::time::Instant::now();
    loop {
        if !frame(o, || draw_page(o, &page)) {
            return Ok(());
        }
        let (w, h) = draw::screen_size(o);
        let l = layout(w, h, page.box_side);
        let visible = (l.text[3] / draw::LINE_H).max(1) as usize;
        let per_line = draw::letters_per_line(o, l.text[2] - 16);
        let lines = page.text.len().div_ceil(per_line);
        let jump = visible.saturating_sub(1).max(1);
        let mut key = take_key(o);
        // A tap on a usable button chooses it and presses it; one on a button that cannot be
        // used, or anywhere else, does nothing.
        #[cfg(touch_screen)]
        if let Some((x, y)) = crate::platform::touch_menu::take_installer_tap() {
            let usable: [bool; BUTTONS] = std::array::from_fn(|n| page.usable(n));
            if let Some(n) = tapped(&l.buttons, &usable, x, y) {
                page.chosen = n;
                key = KEY_ENTER;
            }
        }
        // The pad's left shoulder pages up and its right one down (a keyboard's Tab, down).
        if key == KEY_TAB {
            key = if o.input.take_shoulder() == 1 {
                KEY_PAGE_UP
            } else {
                KEY_PAGE_DOWN
            };
        }
        // The right stick scrolls line by line, repeating while it is held.
        let lean = o.input.scroll_stick();
        let now = std::time::Instant::now();
        if lean == 0 {
            leaning = 0;
        } else if lean != leaning || now >= next_line {
            next_line = now
                + if lean != leaning {
                    STICK_AFTER
                } else {
                    STICK_EVERY
                };
            leaning = lean;
            page.scroll = (page.scroll as isize + lean as isize)
                .clamp(0, lines.saturating_sub(visible) as isize)
                as usize;
        }
        if key != 0 {
            page.message.clear();
        }
        match key {
            KEY_ESCAPE => return Ok(()),
            KEY_LEFT if page.codes.len() > 1 => page.step_code(-1),
            KEY_RIGHT if page.codes.len() > 1 => page.step_code(1),
            KEY_LEFT | KEY_UP => page.step_choice(-1),
            KEY_RIGHT | KEY_DOWN => page.step_choice(1),
            KEY_PAGE_UP => page.scroll = page.scroll.saturating_sub(jump),
            KEY_PAGE_DOWN => page.scroll = (page.scroll + jump).min(lines.saturating_sub(visible)),
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
                DENSITY => {
                    if page.usable(DENSITY) {
                        page.switch_density();
                    }
                }
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
        row(id::SAVE_CODE_ROW_GAMES, games(code), games(now)),
        row(
            id::SAVE_CODE_ROW_SETUPS,
            count(code.setups),
            count(now.setups),
        ),
        row(
            id::SAVE_CODE_ROW_BEST,
            count(code.best_times),
            count(now.best_times),
        ),
    ]
}

/// The question's two buttons (Yes, No), one above the other near the bottom of a screen of
/// `w` x `h`: the rectangles both the drawing and the taps use.
fn question_ways(w: i32, h: i32) -> [[i32; 4]; 2] {
    let m = draw::MARGIN;
    let inner = w - 4 * m;
    let keys_y = h - 2 * m - draw::LINE_H / 2;
    let pitch = draw::BUTTON_H - 6 + draw::WAYS_GAP;
    let ways_y = keys_y - draw::LINE_H / 2 - m / 2 - 2 * pitch;
    std::array::from_fn(|n| [2 * m, ways_y + n as i32 * pitch, inner, draw::BUTTON_H - 6])
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
    for (n, (rect, label)) in question_ways(w, h)
        .into_iter()
        .zip([id::SAVE_CODE_YES, id::SAVE_CODE_NO])
        .enumerate()
    {
        draw::button(o, rect, &own(o, label), chosen == n, true);
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
    #[cfg(touch_screen)]
    let _taps = crate::platform::touch_menu::InstallerPage::begin();
    loop {
        if !frame(o, || draw_question(o, &rows, chosen)) {
            return false;
        }
        // A tap on a button answers with it.
        #[cfg(touch_screen)]
        if let Some((x, y)) = crate::platform::touch_menu::take_installer_tap() {
            let (w, h) = draw::screen_size(o);
            if let Some(n) = tapped(&question_ways(w, h), &[true; 2], x, y) {
                return n == 0;
            }
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

/// The button a tap at (`x`, `y`) in the frame's pixels lands on, if it is one that can be used.
#[cfg(any(touch_screen, test))]
fn tapped(rects: &[[i32; 4]], usable: &[bool], x: f32, y: f32) -> Option<usize> {
    rects.iter().zip(usable).position(|(r, &ok)| {
        ok && x >= r[0] as f32
            && x < (r[0] + r[2]) as f32
            && y >= r[1] as f32
            && y < (r[1] + r[3]) as f32
    })
}

#[cfg(test)]
mod tests {
    use super::date_text;
    use super::{BUTTONS, draw, layout, question_ways, tapped};

    /// The buttons keep clear of each other, of the notes above and of the screen's edges, at
    /// the sizes the page is shown at (800 x 600, a phone's wide screen, a tall one).
    #[test]
    fn buttons_fit_and_do_not_touch() {
        for (w, h) in [(800, 600), (1300, 600), (1000, 600), (640, 480)] {
            let l = layout(w, h, Some(41));
            for (n, b) in l.buttons.iter().enumerate() {
                assert!(b[0] >= 0 && b[0] + b[2] <= w && b[1] >= 0 && b[1] + b[3] <= h);
                assert!(b[3] == draw::BUTTON_H);
                if n + 1 < BUTTONS {
                    let next = l.buttons[n + 1];
                    assert!(next[0] - (b[0] + b[2]) >= draw::BUTTON_GAP, "{w}x{h}");
                }
                // The notes (five lines) end above the buttons.
                assert!(l.notes[1] + l.notes[3] <= b[1], "{w}x{h}");
                assert!(l.text[3] >= draw::LINE_H);
            }
            let [yes, no] = question_ways(w, h);
            assert!(
                no[1] - (yes[1] + yes[3]) >= draw::WAYS_GAP.min(6),
                "{w}x{h}"
            );
            assert!(no[1] + no[3] <= h);
        }
    }

    #[cfg(touch_screen)]
    #[test]
    fn touch_buttons_are_tall_and_far_apart() {
        let [yes, no] = question_ways(800, 600);
        assert!(yes[3] * 2 >= draw::LINE_H * 3);
        assert!((no[1] - (yes[1] + yes[3])) * 2 >= yes[3]);
        assert!(layout(800, 600, Some(41)).buttons[0][3] * 2 >= draw::LINE_H * 3);
    }

    #[test]
    fn a_tap_lands_on_a_usable_button_only() {
        let rects = [[10, 10, 50, 20], [70, 10, 50, 20]];
        assert_eq!(tapped(&rects, &[true, true], 20.0, 15.0), Some(0));
        assert_eq!(tapped(&rects, &[true, true], 80.0, 15.0), Some(1));
        assert_eq!(tapped(&rects, &[false, true], 20.0, 15.0), None);
        assert_eq!(tapped(&rects, &[true, true], 65.0, 15.0), None);
        assert_eq!(tapped(&rects, &[true, true], 20.0, 35.0), None);
    }

    #[test]
    fn dates_are_civil_dates() {
        assert_eq!(date_text(0), "1970-01-01 00:00");
        assert_eq!(date_text(1_700_000_000), "2023-11-14 22:13");
        assert_eq!(date_text(951_782_400), "2000-02-29 00:00");
    }
}
