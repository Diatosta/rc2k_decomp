//! `net::system_message` (0x41976F): a DirectPlay system message (sender 0) handled, by its
//! first dword: 3 (DPSYS_CREATEPLAYERORGROUP) the sync table to be cleared again
//! (`dword_52AD8C` 1); 5 (DPSYS_DESTROYPLAYERORGROUP) the same and, for a player (+4 is 1),
//! that player (+8) taken out of the group (`DeletePlayerFromGroup`, `IDirectPlay4A`
//! +0x1C, `g_netSendTo`); 0x31 (DPSYS_SESSIONLOST) `net::destroy_connection`; 0x101
//! (DPSYS_HOST) this machine the host (`g_netOwnRecord` bit 0x10 set, bit 1 ready cleared),
//! `dword_52AD8C` 1 and the stage item open (`dword_52AC2C` 1). Others: nothing.
//!
//! In:  `message` (ebx)
//! Out: nothing (every register kept, `pusha` .. `popa`; its flags are read by no caller:
//!      the one caller goes on to its loop's test)
//! Caller: `net::take_messages`. `DeletePlayerFromGroup` from Rust (`net::dplay`, section
//! 15). Refuses nothing but what `net::destroy_connection` refuses.

use crate::errors::Refused;
use crate::net::destroy_connection::destroy_connection;
use crate::net::dplay::{DELETE_PLAYER_FROM_GROUP, method};
use crate::net::lobby::{DPLAY, OWN, SEND_TO, STAGE_ITEM, SYNC_FIRST, g, gb, p, pb};
use rc2k_core::net::players::{A_PLAYER, System, made_host};

/// The Rust version.
pub fn system_message(message: u32) -> Result<(), Refused> {
    let get = |at: u32| unsafe { (at as *const u32).read_volatile() };
    let kind = get(message);
    // Converted: `rc2k_core::net::players` (`System`, `made_host`).
    match System::of(kind) {
        System::Created => p(SYNC_FIRST, 1),
        System::Destroyed => {
            p(SYNC_FIRST, 1);
            if get(message + 4) == A_PLAYER {
                unsafe {
                    method(
                        g(DPLAY),
                        DELETE_PLAYER_FROM_GROUP,
                        &[g(SEND_TO), get(message + 8)],
                    )
                };
            }
        }
        System::SessionLost => destroy_connection()?,
        System::Host => {
            pb(OWN, made_host(gb(OWN)));
            p(SYNC_FIRST, 1);
            p(STAGE_ITEM, 1);
        }
        System::Other => {}
    }
    Ok(())
}
