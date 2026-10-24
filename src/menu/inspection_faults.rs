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
use crate::framework::{game_read, game_write, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAGS_RVA: usize = rva(0x759C76); // dword_759C76: 13
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn inspection_faults() -> Result<(), Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::inspection::faults`.
    let p = player(n);
    let flags = rc2k_core::menu::inspection::faults(|k| past_limit(p, k as u32));
    unsafe { game_write(FLAGS_RVA, flags) };
    Ok(())
}
