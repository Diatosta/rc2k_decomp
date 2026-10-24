//! The inspection's two fades (`menu::fade_in`, `menu::fade_out`): frames of
//! `menu::fade_frame` (`screens::run_frame`) for `dword_75AB44` milliseconds, the fade
//! quad's see-through `dword_75AB50` going from 0 up to (in) or from `dword_75AA94` down
//! (out) with the time gone (`dword_75AB4C`, from `dword_75AB48`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{game_read, game_write, rva};

pub const START_RVA: usize = rva(0x75AB48); // dword_75AB48
pub const GONE_RVA: usize = rva(0x75AB4C); // dword_75AB4C
pub const SEE_RVA: usize = rva(0x75AB50); // dword_75AB50
pub const LENGTH_RVA: usize = rva(0x75AB44); // dword_75AB44
pub const FULL_RVA: usize = rva(0x75AA94); // dword_75AA94
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
pub unsafe fn fade(out: bool) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::fades` (`too_short`, `first_see`, `step`, `over`).
    use rc2k_core::menu::fades::{self, Stop};
    let length: u32 = unsafe { game_read(LENGTH_RVA) };
    if fades::too_short(length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    unsafe {
        game_write(START_RVA, crate::system::tick_count::tick_count());
        game_write(SEE_RVA, fades::first_see(out, game_read(FULL_RVA)));
        frame()?;
        loop {
            let gone = crate::system::tick_count::tick_count().wrapping_sub(game_read(START_RVA));
            game_write(GONE_RVA, gone);
            // The length and the full see-through read only where the original reads them.
            let made = if gone != 0 || out {
                fades::step(gone, game_read(LENGTH_RVA), || game_read(FULL_RVA), out)
            } else {
                Ok(None)
            };
            match made {
                Ok(Some(s)) => {
                    if let Some(see) = s.see {
                        game_write(SEE_RVA, see);
                    }
                }
                Ok(None) => {}
                Err(Stop::DivideByZero) => return Err(DIVIDE_BY_ZERO.into()),
                Err(Stop::ShortStep) => {
                    return Err(BadInput("a fade's step of 0 ms: the original divides by 0").into());
                }
            }
            frame()?;
            if fades::over(game_read(GONE_RVA), game_read(LENGTH_RVA)) {
                break;
            }
        }
    }
    Ok(())
}
