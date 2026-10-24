//! `screens::draw_repair_times` (0x4A70D7): draws sixteen values of the service area as
//! clocks, in two columns of eight: `dword_75AB60[0 .. 16]`, left column first in each row.
//!
//! In:  nothing
//! Out: what the last `draw_text` leaves: carry, eax = 0, ebx = after the text's NUL,
//! ecx = 441, edx = 452; esi, edi and ebp preserved
//! Rows are at y = 110 .. 284 in steps of 29, and a last one at 452; columns at x = 106 and
//! 441.
//!
//! Screen: seen in a screenshot of the championship's service area (2026-10-10): the time under each of the
//! fourteen parts of the repair section (for example 10:19 under the gearbox, 1:35 under the
//! tyres), and in the last row the 1:20 under "ride height" and "suspension stiffness".

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::text::draw_text::{Canvas, Outcome};

// ---- game data ----

const TIMES_RVA: usize = rva(0x75AB60); // dword_75AB60[16]
const COUNT: usize = 16;
fn time_rva(i: usize) -> usize {
    TIMES_RVA + i * 4
}

// ---- Rust version ----

/// Returns what the last `draw_text` returned.
pub unsafe fn draw_repair_times(canvas: &mut dyn Canvas) -> Result<Outcome, Refused> {
    // Converted: `rc2k_core::screens::service::repair_times`.
    let times: [u32; COUNT] = std::array::from_fn(|i| unsafe { game_read(time_rva(i)) });
    let (last, x) = rc2k_core::screens::service::repair_times(
        &mut crate::screens::game_screen::ServiceScreen::new(canvas),
        &times,
    )?;
    Ok(Outcome {
        too_wide: last.too_wide,
        text: last.after as *const u8,
        x,
        y: last.y,
    })
}
