//! `screens::gearing_text` (0x4699E0): string 0x21F + the current player's `gearing`, as
//! a text of the car choice.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_7554F0` = the current player's (`dword_75598C`) `gearing`; the string and a NUL into
//! `byte_A8BF54`. In two entry lists (0x7593FE, 0x75941A). Refuses a current player past four, a
//! text longer than its place (0x100). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::player::{Player, player_at};
use crate::screens::car_caption::put_string;

pub const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x21F;

const _: () = assert!(std::mem::offset_of!(Player, gearing) == 0x30);

pub fn current() -> Result<u32, Refused> {
    let n: u32 = { EVENT_STATE.current_player.get() };
    let p = player_at(n);
    if n >= 4 || p == 0 {
        return Err(BadInput("a current player past the four").into());
    }
    Ok(p)
}

pub unsafe fn gearing_text() -> Result<(), Refused> {
    let value = unsafe { (*(current()? as *const Player)).gearing };
    unsafe {
        put_string(
            FIRST_STRING.wrapping_add(value),
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .title
                .addr(),
            TEXT_LEN,
        )?;
        SCREEN_FLOW.shown_gearing.set(value);
    }
    Ok(())
}

/// `menu::next_gearing` / `menu::previous_gearing`: `dword_7554F0` on (past 2 to 0) or back
/// (below 0 to 2, both signed), the current player's `gearing` it, and its text.
pub unsafe fn step_gearing(on: bool) -> Result<(), Refused> {
    let p = current()?;
    unsafe {
        let kept = SCREEN_FLOW.shown_gearing.get() as i32;
        // Converted: `rc2k_core::screens::captions::step_gearing`.
        let value = rc2k_core::screens::captions::step_gearing(kept, on);
        put_string(
            FIRST_STRING.wrapping_add(value),
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .title
                .addr(),
            TEXT_LEN,
        )?;
        SCREEN_FLOW.shown_gearing.set(value);
        (*(p as *mut Player)).gearing = value;
    }
    Ok(())
}
