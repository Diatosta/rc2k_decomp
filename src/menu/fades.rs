//! The inspection's two fades (`menu::fade_in`, `menu::fade_out`): frames of
//! `menu::fade_frame` (`screens::run_frame`) for `dword_75AB44` milliseconds, the fade
//! quad's see-through `dword_75AB50` going from 0 up to (in) or from `dword_75AA94` down
//! (out) with the time gone (`dword_75AB4C`, from `dword_75AB48`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

pub const START_RVA: usize = rva(0x75AB48); // dword_75AB48
pub const GONE_RVA: usize = rva(0x75AB4C); // dword_75AB4C
pub const SEE_RVA: usize = rva(0x75AB50); // dword_75AB50
pub const LENGTH_RVA: usize = rva(0x75AB44); // dword_75AB44
pub const FULL_RVA: usize = rva(0x75AA94); // dword_75AA94
const RUN_FRAME: usize = rva(0x405A71); // screens::run_frame
const FADE_FRAME: usize = rva(0x4A1E60); // menu::fade_frame
// system::tick_count
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};

/// A call by place: 1 `system::tick_count` (eax), 2 `screens::run_frame` (eax the frame).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A fade: `out` the way. Every register kept (`pusha` .. `popa`).
pub unsafe fn fade(entry: &Regs, out: bool, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::fades` (`too_short`, `first_see`, `step`, `over`).
    use rc2k_core::menu::fades::{self, Stop};
    let length: u32 = unsafe { game_read(LENGTH_RVA) };
    if fades::too_short(length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    let mut g = *entry;
    unsafe {
        part(1, &mut g)?;
        game_write(START_RVA, g.eax);
        game_write(SEE_RVA, fades::first_see(out, game_read(FULL_RVA)));
        g.eax = game_addr(FADE_FRAME) as u32;
        part(2, &mut g)?;
        loop {
            part(1, &mut g)?;
            let gone = g.eax.wrapping_sub(game_read(START_RVA));
            g.eax = gone;
            game_write(GONE_RVA, gone);
            // The length and the full see-through read only where the original reads them.
            let made = if gone != 0 || out {
                fades::step(gone, game_read(LENGTH_RVA), || game_read(FULL_RVA), out)
            } else {
                Ok(None)
            };
            match made {
                Ok(Some(s)) => {
                    (g.eax, g.ebx, g.edx, g.ebp) = (s.eax, s.ebx, s.edx, s.ebp);
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
            g.eax = game_addr(FADE_FRAME) as u32;
            part(2, &mut g)?;
            g.eax = game_read(GONE_RVA);
            if fades::over(g.eax, game_read(LENGTH_RVA)) {
                break;
            }
        }
    }
    Ok(*entry)
}

/// In the game: the clock's Rust version, the frame through the game (platform).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => r.eax = crate::system::tick_count::tick_count(),
        _ => crate::calls::call(RUN_FRAME, r),
    }
    Ok(())
}

// ---- the test: a copy whose clock and frame calls go to stand-ins ----
