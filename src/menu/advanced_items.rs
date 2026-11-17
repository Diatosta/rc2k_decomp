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

/// Row `row` one step.
pub fn step(o: Owners<'static>, row: u8, up: bool) -> Result<(), Refused> {
    shown(o);
    let mut guard = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(state) = guard.as_mut() {
        let most = crate::scene::workers::most_threads() as u32;
        // OpenGL alone is a desktop choice: the handheld's GPU renderer is GL ES already.
        let gl = cfg!(not(all(target_os = "linux", target_arch = "aarch64")));
        state.shown.step(usize::from(row), up, most, gl);
    }
    Ok(())
}

/// The renderer swapped for `renderer` at once where the platform can (the desktop's); the
/// swap's failure is logged and the page keeps saying "next start" beside the renderer, as the
/// one that runs is not the one chosen.
fn swap_renderer(o: &Owners<'static>, renderer: Renderer) {
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
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
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let _ = (o, renderer);
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
