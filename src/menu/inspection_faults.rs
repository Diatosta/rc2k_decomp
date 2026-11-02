//! `menu::inspection_faults` (0x4A1733): the inspection's thirteen fault flags
//! (`dword_759C76` ..) cleared, then each 1 for its inspected part of the current player
//! (`dword_75598C`) at or past its limit (`menu::standings::INSPECTED`, in order).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. Refuses a current player past 3 or missing,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::standings::*;
use crate::session::write_prefix;

/// The Rust version (see the header): the registers it leaves.
pub fn inspection_faults() -> Result<(), Refused> {
    let n: u32 = EVENT_STATE.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::inspection::faults`.
    let p = player(n);
    let fault = crate::menu::event_flow::Fault::default();
    let flags = rc2k_core::menu::inspection::faults(|k| fault.ok(past_limit(p, k as u32)));
    fault.check()?;
    write_prefix(&MENU_LISTS.inspection_faults, &flags);
    Ok(())
}
