//! `menu::sealed_player_time` (0x4A3D38): the current player's (`dword_75598C`) time
//! (+0x88) sealed (`records::seal_time`) with bl its +8, cl `dword_75ACD8` and dl eax's low
//! byte; the result also into `dword_7559EC` and `hhmm`.
//!
//! In:  eax (its low byte sealed with the time)
//! Out: eax, ebx the sealed block; the rest kept
//!
//! Callers: `sub_489F76`, `sub_48C8B6`, `menu::enter_record_entry`. The seal is its Rust version.
//! Refuses a current player past 3 or missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::menu::standings::*;
use crate::race::state_store::RACE_STATE;

/// The Rust version (see the header): `byte` the original's eax (its low byte sealed);
/// the sealed block's two dwords (its eax, ebx) out.
pub fn sealed_player_time(byte: u32) -> Result<[u32; 2], Refused> {
    let screens = &menus().screens;
    let n: u32 = screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(n);
    let sealed = {
        crate::records::seal_time::seal_time(
            p.get().time,
            [
                p.get().car_number as u8,
                RACE_STATE.settings.run_kind.get() as u8,
                byte as u8,
            ],
        )
    };

    screens.draw_work.figure.set(sealed[0]);
    screens.draw_work.hhmm.set(sealed[1]);

    Ok(sealed)
}
