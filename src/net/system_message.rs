//! `net::system_message` (0x41976F): a DirectPlay system message (sender 0) handled, by its
//! first dword: 3 (DPSYS_CREATEPLAYERORGROUP) the sync table to be cleared again
//! (`sync_first` 1); 5 (DPSYS_DESTROYPLAYERORGROUP) the same and, for a player (+4 is 1),
//! that player (+8) taken out of the group (`DeletePlayerFromGroup`, `IDirectPlay4A`
//! +0x1C, `g_netSendTo`); 0x31 (DPSYS_SESSIONLOST) `net::destroy_connection`; 0x101
//! (DPSYS_HOST) this machine the host (`g_netOwnRecord` bit 0x10 set, bit 1 ready cleared),
//! `sync_first` 1 and the stage item open (`stage_item` 1). Others: nothing.
//!
//! Takes `message`.
//! Caller: `net::take_messages`. `DeletePlayerFromGroup` from Rust (`net::dplay`, section
//! 15). Refuses nothing but what `net::destroy_connection` refuses.

use crate::errors::Refused;
use crate::net::destroy_connection::destroy_connection;
use crate::net::store::NetVars;
use crate::net::store::PlayerRecord;
use rc2k_core::net::players::{A_PLAYER, System, made_host};

/// The message is in `crate::net::owner::net().vars.receive_data`.
pub fn system_message(vars: &NetVars) -> Result<(), Refused> {
    let get = |at: usize| vars.receive_data.get_u32(at);
    let kind = get(0);
    // Converted: `rc2k_core::net::players` (`System`, `made_host`).
    match System::of(kind) {
        System::Created => vars.sync_first.set(1),
        System::Destroyed => {
            vars.sync_first.set(1);
            if get(4) == A_PLAYER {
                crate::net::dp_objects::delete_player_from_group(
                    vars.dplay.get(),
                    vars.send_to.get(),
                    get(8),
                );
            }
        }
        System::SessionLost => destroy_connection(vars)?,
        System::Host => {
            vars.own
                .head
                .set(PlayerRecord::STATE, made_host(vars.own.state()));
            vars.sync_first.set(1);
            vars.stage_item.set(1);
        }
        System::Other => {}
    }
    Ok(())
}
