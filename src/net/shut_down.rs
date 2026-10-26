//! `net::shut_down` (0x4184D6): the network at the game's end: `net::stop`, then the
//! received messages' mutex (`dword_52AB6C`) and the receiving thread's event
//! (`dword_52AB68`) taken out (`xchg` with 0) and closed (`sub_411092`, `sub_41104F`:
//! `CloseHandle` if not 0).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller in the standalone, `start`'s way out (0x4010EE; the
//! original's other, `sub_4104F2`, is not reached). Refuses what `net::stop` refuses.

use crate::errors::Refused;
use crate::net::store::NET;
use std::sync::atomic::Ordering;

/// The Rust version.
pub fn shut_down() -> Result<(), Refused> {
    crate::net::stop::stop()?;
    for cell in [&NET.mutex, &NET.event] {
        let handle = cell.atomic().swap(0, Ordering::SeqCst);
        if handle != 0 {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(crate::pointers::host(handle) as _)
            };
        }
    }
    Ok(())
}
