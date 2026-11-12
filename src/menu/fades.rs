//! The inspection's two fades (`menu::fade_in`, `menu::fade_out`): frames of
//! `menu::fade_frame` (`screens::run_frame`) for `state.menus.fade_length` milliseconds, the fade
//! quad's see-through going from 0 up to (in) or from the fade's full see-through down
//! (out) with the time gone since the fade started.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::owner::Menus;

// system::tick_count
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};

/// `screens::run_frame` with `menu::fade_frame` (0x4A1E60): the messages, then (unless
/// quitting) the frame.
fn frame(menus: &Menus) -> Result<(), Refused> {
    match crate::platform::run_frame_now(|| crate::menu::fade_frame::fade_frame(menus)) {
        Some(made) => made.map(|_| ()),
        None => Ok(()),
    }
}

/// A fade: `out` the way.
pub fn fade(menus: &Menus, out: bool) -> Result<(), Refused> {
    let screens = &menus.screens;
    // The logic is in `rc2k_core::menu::fades` (`too_short`, `first_see`, `step`, `over`).
    use rc2k_core::menu::fades::{self, Stop};
    let length: u32 = screens.state.menus.fade_length.get();
    if fades::too_short(length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    {
        screens
            .state
            .menus
            .fade_start
            .set(crate::system::tick_count::tick_count());
        screens.state.menus.fade_see.set(fades::first_see(
            out,
            screens.state.service_ui.fade_full.get(),
        ));
        frame(menus)?;
        loop {
            let gone = crate::system::tick_count::tick_count()
                .wrapping_sub(screens.state.menus.fade_start.get());
            screens.state.menus.fade_gone.set(gone);
            // The length and the full see-through read only where the original reads them.
            let made = if gone != 0 || out {
                fades::step(
                    gone,
                    screens.state.menus.fade_length.get(),
                    || screens.state.service_ui.fade_full.get(),
                    out,
                )
            } else {
                Ok(None)
            };
            match made {
                Ok(Some(see)) => screens.state.menus.fade_see.set(see),
                Ok(None) => {}
                Err(Stop::DivideByZero) => return Err(DIVIDE_BY_ZERO.into()),
                Err(Stop::ShortStep) => {
                    return Err(BadInput("a fade's step of 0 ms: the original divides by 0").into());
                }
            }
            frame(menus)?;
            if fades::over(
                screens.state.menus.fade_gone.get(),
                screens.state.menus.fade_length.get(),
            ) {
                break;
            }
        }
    }
    Ok(())
}
