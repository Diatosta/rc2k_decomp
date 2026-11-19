//! The advanced page's state and actions: the values the rows show (the live ones until a row is
//! stepped), a step of a row, and "apply", which makes the changed rows real: a switch of
//! `var\ral.cfg` is set in the game's variable when it takes effect at once or in the next race,
//! and written to the file as the game reads it; the drawing threads and the renderer go to
//! the options file (saved when the page is left, `ScreenId::changes_settings`). A changed renderer
//! is also swapped in at once on the desktop (`platform::swap_renderer`); the handheld's is only
//! the one tried first at the next start.

use crate::errors::Refused;
use crate::owners::Owners;
use rc2k_core::options::advanced::{
    Kind, ROWS, Renderer, Values, When, flag_number, flag_shown, set_key,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Whether the page has the row that shares the log (phones with Android's share sheet).
pub const PHONE: bool = cfg!(target_os = "android");

/// The part of the page shown: 0 the first `PAGE_ROWS` rows, 1 the rest.
static PAGE: AtomicUsize = AtomicUsize::new(0);

/// The part of the page shown.
pub fn page() -> usize {
    PAGE.load(Ordering::Relaxed)
}

/// The page shows its other part.
pub fn turn_page() -> Result<(), Refused> {
    PAGE.store(1 - page().min(1), Ordering::Relaxed);
    Ok(())
}

/// What the page shows and what the game and the files have been given so far.
struct State {
    shown: Values,
    applied: Values,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// A switch's number in `var\ral.cfg` if it is there (the first line of the key).
fn file_number(text: &[u8], key: &str) -> Option<u32> {
    let mut with_nul = text.to_vec();
    with_nul.push(0);
    let parsed = rc2k_core::files::settings::parse_settings(
        &with_nul,
        &crate::strings::same_text::case_table(),
    )
    .ok()?;
    parsed
        .into_iter()
        .find(|(setting, _)| setting.name() == key.as_bytes())
        .map(|(_, number)| number)
}

fn cfg_path() -> std::path::PathBuf {
    crate::files::host_path(b"var\\ral.cfg")
}

/// The values as the game and its files have them now: the live variables, except for the rows
/// that take effect at the next start, which show what the file says.
fn current(o: Owners<'static>) -> Values {
    let file = std::fs::read(cfg_path()).unwrap_or_default();
    let mut flags = [false; ROWS.len()];
    for (n, row) in ROWS.iter().enumerate() {
        if let Kind::Flag { key, inverted } = row.kind {
            let live = crate::settings::read_settings::value_of(o, key).unwrap_or(0);
            let number = if row.when == When::NextStart {
                file_number(&file, key).unwrap_or(live)
            } else {
                live
            };
            flags[n] = flag_shown(inverted, number);
        }
    }
    Values {
        threads: crate::scene::workers::threads() as u32,
        renderer: Renderer::of(crate::render::choice::wanted().as_deref()),
        quit_pictures: crate::drive::quit_question::pictures_on(),
        smooth_frames: crate::race::blend::smooth_on(),
        flags,
    }
}

/// The renderer the page was opened with or the last Apply asked for.
pub fn applied_renderer(o: Owners<'static>) -> Renderer {
    shown(o);
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .map_or(Renderer::Gpu, |state| state.applied.renderer)
}

/// Drops what was stepped and not applied (the page is opened).
pub fn forget_changes() {
    PAGE.store(0, Ordering::Relaxed);
    if let Ok(mut state) = STATE.lock() {
        *state = None;
    }
}

/// The values the page shows.
pub fn shown(o: Owners<'static>) -> Values {
    let mut state = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state
        .get_or_insert_with(|| {
            let now = current(o);
            State {
                shown: now,
                applied: now,
            }
        })
        .shown
}

/// The item slot `row` of the part shown, stepped one: the part's rows start at its first.
pub fn step(o: Owners<'static>, row: u8, up: bool) -> Result<(), Refused> {
    let row = rc2k_core::options::advanced::rows_on_page(page(), PHONE).start as u8 + row;
    if usize::from(row) >= ROWS.len() {
        return Ok(());
    }
    if matches!(ROWS[usize::from(row)].kind, Kind::ShareLog) {
        // An action: either key, or a tap, opens the share sheet.
        crate::platform::share_log();
        return Ok(());
    }
    shown(o);
    let mut guard = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(state) = guard.as_mut() {
        let most = crate::scene::workers::most_threads() as u32;
        // OpenGL alone is a desktop choice: the handheld's GPU renderer is GL ES already, and a
        // phone's one window is the GPU renderer's.
        let gl = cfg!(not(any(handheld, target_os = "android")));
        state.shown.step(usize::from(row), up, most, gl);
    }
    Ok(())
}

/// Enter on a row of the part shown: the last row (the deletion of all saved data) asks
/// and deletes; the settings' rows are stepped with the left and right keys, so nothing here.
pub fn delete_saved_data(o: Owners<'static>) -> Result<(), Refused> {
    let row = rc2k_core::options::advanced::rows_on_page(page(), PHONE).start + DELETE_SLOT;
    if !matches!(ROWS.get(row).map(|r| r.kind), Some(Kind::DeleteData)) {
        return Ok(());
    }
    crate::saves::delete_all::choose(o)
}

/// The item slot of the part shown that holds the deletion row (on the second part, the one
/// after the settings).
pub const DELETE_SLOT: usize = ROWS.len() - 2 - rc2k_core::options::advanced::PAGE_ROWS;

/// The renderer swapped for `renderer` at once where the platform can (the desktop's); the
/// swap's failure is logged and the page keeps saying "next start" beside the renderer, as the
/// one that runs is not the one chosen.
fn swap_renderer(o: &Owners<'static>, renderer: Renderer) {
    #[cfg(not(handheld))]
    {
        use crate::render::choice::Wish;
        let wish = match renderer {
            Renderer::Gpu => Wish::GpuFirst,
            Renderer::Gl => Wish::Gl,
            Renderer::Software => Wish::Software,
        };
        if let Err(why) = crate::platform::swap_renderer(o, wish) {
            crate::logfile::program_log::log(&format!(
                "advanced: the renderer {} was not swapped in: {why}",
                renderer.name()
            ));
        }
    }
    // The handheld's GL ES context is made once for the program's life (the panel's EGL
    // surface): its renderer is tried first at the next start.
    #[cfg(handheld)]
    let _ = (o, renderer);
}

/// The key of `var\ral.cfg` that turns rain and snow off.
const NO_PRECIPITATION: &str = "noprecipitation";

/// Whether rain and snow are on for the next race (shown by the graphics page).
pub fn rain_and_snow_on(o: Owners<'static>) -> bool {
    let number = crate::settings::read_settings::value_of(o, NO_PRECIPITATION).unwrap_or(0);
    flag_shown(true, number)
}

/// The graphics page's rain and snow switch turned over: the game's variable changes (the next
/// race loaded uses it) and the key in `var\ral.cfg` is written as the game reads it.
pub fn toggle_rain_and_snow(o: Owners<'static>) -> Result<(), Refused> {
    let number = flag_number(true, !rain_and_snow_on(o));
    crate::settings::read_settings::set_value(o, NO_PRECIPITATION, number);
    let path = cfg_path();
    let file = set_key(
        &std::fs::read(&path).unwrap_or_default(),
        NO_PRECIPITATION,
        number,
    );
    if let Err(e) = std::fs::write(&path, &file) {
        crate::logfile::program_log::log(&format!(
            "graphics: var\\ral.cfg could not be written: {e}"
        ));
    }
    Ok(())
}

/// Makes the changed rows real.
pub fn apply(o: Owners<'static>) -> Result<(), Refused> {
    shown(o);
    let mut guard = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(state) = guard.as_mut() else {
        return Ok(());
    };
    let (new, old) = (state.shown, state.applied);
    let mut text: Option<Vec<u8>> = None;
    for (n, row) in ROWS.iter().enumerate() {
        match row.kind {
            Kind::Threads if new.threads != old.threads => {
                let most = crate::scene::workers::most_threads() as u32;
                // The most is the default: nothing is kept for it.
                crate::scene::workers::set_threads((new.threads < most).then_some(new.threads));
            }
            Kind::Renderer if new.renderer != old.renderer => {
                // The setting is kept whatever the swap does: a renderer that did not start
                // now is tried first at the next start.
                crate::render::choice::set_wanted(Some(new.renderer.name().to_owned()));
                swap_renderer(&o, new.renderer);
            }
            Kind::QuitPictures if new.quit_pictures != old.quit_pictures => {
                crate::drive::quit_question::set_pictures_on(new.quit_pictures);
            }
            Kind::SmoothFrames if new.smooth_frames != old.smooth_frames => {
                crate::race::blend::set_smooth(new.smooth_frames);
            }
            Kind::Flag { key, inverted } if new.flags[n] != old.flags[n] => {
                let number = flag_number(inverted, new.flags[n]);
                if row.when != When::NextStart {
                    crate::settings::read_settings::set_value(o, key, number);
                }
                let file =
                    text.get_or_insert_with(|| std::fs::read(cfg_path()).unwrap_or_default());
                *file = set_key(file, key, number);
            }
            _ => {}
        }
    }
    if let Some(file) = text {
        let path = cfg_path();
        if let Err(e) = std::fs::write(&path, &file) {
            crate::logfile::program_log::log(&format!(
                "advanced: var\\ral.cfg could not be written: {e}"
            ));
        }
    }
    crate::logfile::program_log::log(&format!(
        "advanced: applied {new:?} (running renderer: {:?})",
        crate::render::running_wish()
    ));
    state.applied = new;
    Ok(())
}
