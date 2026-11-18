//! The error dialog: what the player sees where the program used to end with nothing but a
//! line in `rc2k.log` (or a bare message box). A panel with the message and the ways on, drawn
//! with the game's own fonts on a plain screen, answered with the keyboard or the pad
//! (Up / Down choose, Enter confirms, Escape takes the last way).
//!
//! - `fatal` ends the program through it: the message, "Quit", the log kept.
//! - `notice` tells the player of a trouble the game has recovered from, and goes on after
//!   Enter. A notice raised before the window exists (the options file's) waits in a queue for
//!   `show_pending`, which the main loop runs once the screen is there.
//!
//! The dialog runs on the game's thread only, once the owners are published (`publish`); on any
//! other thread, before that, with `RC2K_NO_ERROR_DIALOG` set (the unattended checks; a `remote` test build needs
//! `RC2K_ERROR_DIALOG` to show it), or
//! when the window has been closed, nothing is drawn and the caller carries on as before.

pub mod recover;

use crate::gfx::typefaces::fonts;
use crate::owners::Owners;
use crate::text::draw_text::{GameCanvas, draw_text_at};
use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// What the player chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Quit,
    Continue,
    MainMenu,
}

thread_local! {
    /// The owners, on the game's thread.
    static OWNERS: Cell<Option<Owners<'static>>> = const { Cell::new(None) };
}

/// Notices raised before there was a window to show them in.
static PENDING: Mutex<Vec<Queued>> = Mutex::new(Vec::new());

/// A notice waiting for a screen: its text, or (from a thread that has no language to hand)
/// an own text and what goes in it, put together when it is shown.
enum Queued {
    Text(String),
    Own { id: u32, with: String },
}
/// Whether `PENDING` holds something (read each menu frame).
static WAITING: AtomicBool = AtomicBool::new(false);
/// A dialog is up: an error inside it must not open another.
static SHOWING: AtomicBool = AtomicBool::new(false);

/// Called once on the game's thread: the dialog may be drawn from here on.
pub fn publish(o: Owners<'static>) {
    OWNERS.with(|owners| owners.set(Some(o)));
}

/// Whether the environment asks for no dialogs (unattended runs wait for no key).
fn disabled() -> bool {
    // The test builds (`remote`) are driven by scripts that wait for no key: they show it only
    // when asked to.
    std::env::var_os("RC2K_NO_ERROR_DIALOG").is_some()
        || (cfg!(feature = "remote") && std::env::var_os("RC2K_ERROR_DIALOG").is_none())
}

/// The owners if a dialog can be shown now.
fn ready() -> Option<Owners<'static>> {
    if disabled() || cfg!(test) {
        return None;
    }
    let o = OWNERS.with(Cell::get)?;
    (!o.system.session.flags.quitting.is_set()).then_some(o)
}

/// The own text `id` in the language in use, as text.
fn own(o: Owners<'static>, id: u32) -> String {
    rc2k_core::files::ansi::ansi_text(&crate::strings::append_string_by_id::own_text(
        o.strings, id,
    ))
}

/// The notice text for `id` with its "{}" replaced by `with`, in the language in use.
pub fn text_with(id: u32, with: &str) -> String {
    let template = match OWNERS.with(Cell::get) {
        Some(o) => own(o, id),
        None => rc2k_core::files::ansi::ansi_text(
            rc2k_core::strings::own::text(id, None).unwrap_or_default(),
        ),
    };
    template.replace("{}", with)
}

/// A notice for the player: logged now; shown at once when a window is there, else when
/// `show_pending` runs.
pub fn notice(text: &str) {
    crate::logfile::program_log::log(&format!("notice: {text}"));
    let shown = ready().is_some_and(|o| show(o, DIALOG_NOTICE, text, &[Way::Continue]).is_some());
    if !shown {
        queue(text);
    }
}

/// A notice that waits for the next menu frame, where a panel over a running race would stall
/// it (a network race's), from a thread that is not the game's: the text is made in the language in
/// use when the notice is shown, not here.
pub fn notice_own_later(key: &str, id: u32, with: &str) {
    if first_time(key) {
        crate::logfile::program_log::log(&format!("notice: {}", text_with(id, with)));
        queue_item(Queued::Own {
            id,
            with: with.to_string(),
        });
    }
}

/// `notice_own_later` for a notice that is wanted every time it happens (not once for the run).
pub fn notice_own_each_time(id: u32, with: &str) {
    crate::logfile::program_log::log(&format!("notice: {}", text_with(id, with)));
    queue_item(Queued::Own {
        id,
        with: with.to_string(),
    });
}

/// `notice_once` for a thread that is not the game's (the audio thread's): the notice waits for
/// the game's thread to show it, in the language in use.
pub fn notice_own_once(key: &str, id: u32, with: &str) {
    if OWNERS.with(Cell::get).is_some() {
        notice_once(key, &text_with(id, with));
    } else {
        notice_own_later(key, id, with);
    }
}

fn queue(text: &str) {
    queue_item(Queued::Text(text.to_string()));
}

fn queue_item(item: Queued) {
    if !cfg!(test) {
        PENDING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(item);
        WAITING.store(true, Ordering::SeqCst);
    }
}

/// Whether `key` is new in this run.
fn first_time(key: &str) -> bool {
    static TOLD: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let mut told = TOLD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if told.iter().any(|k| k == key) {
        return false;
    }
    told.push(key.to_string());
    true
}

/// `notice`, once for each `key` in the run: a file that fails every time it is asked for (a
/// picture of the menus) tells the player once.
pub fn notice_once(key: &str, text: &str) {
    if first_time(key) {
        notice(text);
    }
}

/// The last part of a path (either kind of slash), as text.
pub fn file_name(path: &[u8]) -> String {
    let name = path
        .rsplit(|&b| b == b'\\' || b == b'/')
        .next()
        .unwrap_or(path);
    rc2k_core::files::ansi::ansi_text(name)
}

/// The notices that waited for a window, shown now.
pub fn show_pending() {
    if !WAITING.load(Ordering::SeqCst) {
        return;
    }
    // Only the game's thread shows them, once the screen is there.
    let Some(o) = ready() else {
        return;
    };
    WAITING.store(false, Ordering::SeqCst);
    let waiting = std::mem::take(
        &mut *PENDING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    for item in waiting {
        let text = match item {
            Queued::Text(text) => text,
            Queued::Own { id, with } => text_with(id, &with),
        };
        let _ = show(o, DIALOG_NOTICE, &text, &[Way::Continue]);
    }
}

/// An error that ends the program: shown with "Quit" when the dialog can be, and returns after
/// the player has answered. The caller ends the program as it did before.
pub fn fatal(text: &str) {
    if let Some(o) = ready()
        && show(o, DIALOG_ERROR, text, &[Way::Quit]).is_none()
        && !text.is_empty()
    {
        // Too early for the game's fonts or the screen: the system's own box says it.
        crate::platform::os::message_box(
            crate::platform::os::Icon::Warning,
            b"Rally Championship",
            &rc2k_core::files::ansi::ansi_bytes(text),
        );
    }
}

/// A trouble the game goes back to the main menu from: shown with "Main menu" and "Quit".
/// True when the player chose to go on at the main menu (also where no dialog can be shown:
/// an unattended run goes on).
pub fn ask_back(text: &str) -> bool {
    crate::logfile::program_log::log(&format!("error: {text}"));
    // The trouble's own words are the program's (English); the lead-in is in the language in use.
    let shown = text_with(rc2k_core::strings::own::id::BACK_TO_MENU, text);
    ready().and_then(|o| show(o, DIALOG_NOTICE, &shown, &[Way::MainMenu, Way::Quit]))
        != Some(Way::Quit)
}

const DIALOG_ERROR: u32 = rc2k_core::strings::own::id::DIALOG_ERROR;
const DIALOG_NOTICE: u32 = rc2k_core::strings::own::id::DIALOG_NOTICE;

/// Colours (0xRRGGBB): the plain screen, the panel, a way on and the one chosen.
const SCREEN: u32 = 0x05081A;
const PANEL: u32 = 0x0C1238;
const WAY: u32 = 0x27347A;
const WAY_CHOSEN: u32 = 0xC8202A;

const MARGIN: i32 = 20;
const LINE_H: i32 = 30;
const WAY_H: i32 = 40;
/// The font's own top offset puts its letters a little below the y they are given.
const TEXT_UP: i32 = 12;

/// The keys that answer (the game's key codes).
const KEY_ENTER: u8 = 0x1C;
const KEY_PAD_ENTER: u8 = 0x60;
const KEY_ESCAPE: u8 = 0x01;
const KEY_UP: u8 = 0x66;
const KEY_DOWN: u8 = 0x6B;

/// The answer a key gives with `ways` and `chosen`: (the way chosen, the way confirmed).
fn after_key(key: u8, chosen: usize, ways: &[Way]) -> (usize, Option<Way>) {
    match key {
        KEY_UP => (chosen.saturating_sub(1), None),
        KEY_DOWN => ((chosen + 1).min(ways.len() - 1), None),
        KEY_ENTER | KEY_PAD_ENTER => (chosen, Some(ways[chosen])),
        KEY_ESCAPE => (chosen, ways.last().copied()),
        _ => (chosen, None),
    }
}

fn way_text(o: Owners<'static>, way: Way) -> String {
    use rc2k_core::strings::own::id;
    own(
        o,
        match way {
            Way::Quit => id::DIALOG_QUIT,
            Way::Continue => id::DIALOG_CONTINUE,
            Way::MainMenu => id::DIALOG_MENU,
        },
    )
}

/// The dialog up until a way is confirmed. `ways` is not empty.
/// The way chosen; none when there was no screen to draw on yet.
fn show(o: Owners<'static>, title_id: u32, message: &str, ways: &[Way]) -> Option<Way> {
    if SHOWING.swap(true, Ordering::SeqCst) {
        return Some(ways[0]);
    }
    let answer = run(o, title_id, message, ways);
    SHOWING.store(false, Ordering::SeqCst);
    answer
}

fn run(o: Owners<'static>, title_id: u32, message: &str, ways: &[Way]) -> Option<Way> {
    let title = own(o, title_id);
    let log_line = if title_id == DIALOG_ERROR {
        own(o, rc2k_core::strings::own::id::DIALOG_LOG)
    } else {
        String::new()
    };
    let keys_line = own(o, rc2k_core::strings::own::id::DIALOG_KEYS);
    let labels: Vec<String> = ways.iter().map(|&w| way_text(o, w)).collect();
    // Keys and pad answers from before are not answers.
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    crate::input::typed_text::osk_close();
    let mut chosen = 0;
    let mut drawn = false;
    // What the player is given when the window closes or the screen goes: the last way.
    let gone = |drawn: bool| drawn.then(|| ways[ways.len() - 1]);
    loop {
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() {
            return gone(drawn);
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return gone(drawn);
        }
        let text_drawn = draw(o, &title, message, &log_line, &keys_line, &labels, chosen);
        crate::platform::unlock_surface(o.gfx);
        if !text_drawn && !drawn {
            return None;
        }
        drawn = true;
        if !crate::platform::show_frame_now(o) {
            return gone(drawn);
        }
        crate::input::menu_pad::take_answer(o.input);
        let key = crate::input::take_key::take_key(o.input);
        let (now, done) = after_key(key, chosen, ways);
        chosen = now;
        if let Some(way) = done {
            return Some(way);
        }
    }
}

fn draw(
    o: Owners<'static>,
    title: &str,
    message: &str,
    log_line: &str,
    keys_line: &str,
    labels: &[String],
    chosen: usize,
) -> bool {
    use crate::gfx::button_hints::width_of;
    use crate::input::osk_draw::fill;
    let mut canvas = GameCanvas::new(o.gfx);
    let screen = o.system.session.screen.get();
    let (w, h) = (screen.width.get() as i32, screen.height.get() as i32);
    fill(o, [0, 0, w, h], SCREEN);
    let panel_w = (w - 40).min(720);
    // A canvas is only a handle on the game's fonts: one for each measure.
    let measure = |s: &str| {
        width_of(
            fonts::LIST,
            &rc2k_core::files::ansi::ansi_bytes(s),
            &mut GameCanvas::new(o.gfx),
        )
    };
    let inner = panel_w - 2 * MARGIN;
    let body: Vec<String> = {
        let mut lines = Vec::new();
        for part in [message, log_line] {
            if part.is_empty() {
                continue;
            }
            lines.extend(rc2k_core::fatal::wrap_lines(part, inner, &measure));
            lines.push(String::new());
        }
        lines
    };
    let panel_h = (MARGIN
        + LINE_H * 2
        + body.len() as i32 * LINE_H
        + labels.len() as i32 * WAY_H
        + LINE_H
        + MARGIN)
        .min(h - 20);
    let (left, top) = ((w - panel_w) / 2, (h - panel_h) / 2);
    fill(o, [left, top, panel_w, panel_h], PANEL);
    let all_drawn = Cell::new(true);
    let put = |text: &str, x: i32, y: i32, canvas: &mut GameCanvas| {
        let mut bytes = rc2k_core::files::ansi::ansi_bytes(text);
        bytes.push(0);
        if draw_text_at(fonts::LIST, &bytes[..], x, y - TEXT_UP, canvas).is_err() {
            // The fonts are not built yet (a start-up that failed early).
            all_drawn.set(false);
        }
    };
    let mut y = top + MARGIN;
    put(title, left + MARGIN, y + LINE_H / 2, &mut canvas);
    y += LINE_H * 2;
    for line in &body {
        if y + LINE_H > top + panel_h - labels.len() as i32 * WAY_H - LINE_H {
            break;
        }
        put(line, left + MARGIN, y + LINE_H / 2, &mut canvas);
        y += LINE_H;
    }
    let mut y = top + panel_h - MARGIN - LINE_H - labels.len() as i32 * WAY_H;
    for (n, label) in labels.iter().enumerate() {
        let colour = if n == chosen { WAY_CHOSEN } else { WAY };
        fill(o, [left + MARGIN, y, inner, WAY_H - 6], colour);
        let x = left + (panel_w - measure(label)) / 2;
        put(label, x, y + (WAY_H - 6) / 2, &mut canvas);
        y += WAY_H;
    }
    let x = left + (panel_w - measure(keys_line)) / 2;
    put(
        keys_line,
        x,
        top + panel_h - MARGIN - LINE_H / 2,
        &mut canvas,
    );
    all_drawn.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_move_confirm_and_leave() {
        let ways = [Way::Continue, Way::Quit];
        assert_eq!(after_key(KEY_DOWN, 0, &ways), (1, None));
        assert_eq!(after_key(KEY_DOWN, 1, &ways), (1, None));
        assert_eq!(after_key(KEY_UP, 0, &ways), (0, None));
        assert_eq!(after_key(KEY_ENTER, 1, &ways), (1, Some(Way::Quit)));
        assert_eq!(after_key(KEY_ESCAPE, 0, &ways), (0, Some(Way::Quit)));
        assert_eq!(after_key(0, 0, &ways), (0, None));
        assert_eq!(
            after_key(KEY_PAD_ENTER, 0, &[Way::Quit]),
            (0, Some(Way::Quit))
        );
    }

    #[test]
    fn nothing_is_shown_without_a_window() {
        notice("a notice in a test");
        fatal("an error in a test");
        assert!(PENDING.lock().unwrap().is_empty());
    }
}
