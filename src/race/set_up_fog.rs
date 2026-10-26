//! `race::set_up_fog` (0x44FBD9): the frame's fog: `scene::restart_clock`, the fog colour
//! into `dword_624000` (`dword_6092CC` << 16 + `dword_6092D0` << 8 + `dword_6092D4`),
//! `race::fog_table`, then the colour with 0xFF000000 to the device (`sub_44E76B`:
//! `dword_5AB964`, then render state 0x22 through `sub_41AE25` and `d3d::set_render_state`;
//! a call into the game, section 15).
//!
//! In:  nothing
//! Out: eax the colour | 0xFF000000; the rest as `fog_table` leaves them (callers
//!      `sub_4557AB` 0x4557E0 and `sub_455F17` 0x455FEA go on with a `call` / `retn`)
//!
//! Refuses (before anything) what `fog_table` would. Hook removed 2026-10-22 (direct calls):
//! Rust callers run its `entry` (`calls::call`, `routes.rs`); the registers above are the
//! original's, for reference.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { set_up_fog(r, &mut fog_part) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
