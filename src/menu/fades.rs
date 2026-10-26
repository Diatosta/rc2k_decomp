//! The inspection's two fades (`menu::fade_in`, `menu::fade_out`): frames of
//! `menu::fade_frame` (`screens::run_frame`) for `dword_75AB44` milliseconds, the fade
//! quad's see-through `dword_75AB50` going from 0 up to (in) or from `dword_75AA94` down
//! (out) with the time gone (`dword_75AB4C`, from `dword_75AB48`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::state_store::MENU_STATE;

// system::tick_count
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};

/// `screens::run_frame` with `menu::fade_frame` (0x4A1E60): the messages, then (unless
/// quitting) the frame.
fn frame() -> Result<(), Refused> {
    match crate::platform::run_frame_now(crate::menu::fade_frame::fade_frame) {
        Some(made) => made.map(|_| ()),
        None => Ok(()),
    }
}

/// A fade: `out` the way. Every register kept (`pusha` .. `popa`).
pub fn fade(out: bool) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::fades` (`too_short`, `first_see`, `step`, `over`).
    use rc2k_core::menu::fades::{self, Stop};
    let length: u32 = MENU_STATE.menus.fade_length.get();
    if fades::too_short(length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    {
        MENU_STATE
            .menus
            .fade_start
            .set(crate::system::tick_count::tick_count());
        MENU_STATE
            .menus
            .fade_see
            .set(fades::first_see(out, MENU_STATE.service_ui.fade_full.get()));
        frame()?;
        loop {
            let gone = crate::system::tick_count::tick_count()
                .wrapping_sub(MENU_STATE.menus.fade_start.get());
            MENU_STATE.menus.fade_gone.set(gone);
            // The length and the full see-through read only where the original reads them.
            let made = if gone != 0 || out {
                fades::step(
                    gone,
                    MENU_STATE.menus.fade_length.get(),
                    || MENU_STATE.service_ui.fade_full.get(),
                    out,
                )
            } else {
                Ok(None)
            };
            match made {
                Ok(Some(s)) => {
                    if let Some(see) = s.see {
                        MENU_STATE.menus.fade_see.set(see);
                    }
                }
                Ok(None) => {}
                Err(Stop::DivideByZero) => return Err(DIVIDE_BY_ZERO.into()),
                Err(Stop::ShortStep) => {
                    return Err(BadInput("a fade's step of 0 ms: the original divides by 0").into());
                }
            }
            frame()?;
            if fades::over(
                MENU_STATE.menus.fade_gone.get(),
                MENU_STATE.menus.fade_length.get(),
            ) {
                break;
            }
        }
    }
    Ok(())
}
