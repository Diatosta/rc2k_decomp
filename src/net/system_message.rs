//! Handles a DirectPlay system message by its first dword: a player or group made, the sync
//! table is to be cleared again; destroyed, the same and the player is removed from the group;
//! session lost, `net::destroy_connection`; host, this machine becomes the host and the stage item
//! opens. Others do nothing.

use crate::errors::Refused;
use crate::net::destroy_connection::destroy_connection;
use crate::net::owner::Net;
use rc2k_core::net::message::SystemMessage;
use rc2k_core::net::players::{A_PLAYER, System};

/// The message is in `crate::net::owner::net().vars.receiver.receive_data`.
pub fn system_message(
    net: &Net,
    lines: &crate::race::viewer_screen_store::NetTexts,
) -> Result<(), Refused> {
    let (vars, link) = (&net.vars, &net.link);
    let bytes = vars
        .receiver
        .receive_data
        .read_at(0, 12, "a system message")?;
    let message = SystemMessage::parse(&bytes);
    match message.kind {
        System::Created => vars.sync_first.set(1),
        System::Destroyed => {
            vars.sync_first.set(1);
            if message.player_type == A_PLAYER {
                crate::net::dp_objects::delete_player_from_group(
                    link.dplay(),
                    link.send_to(),
                    message.id,
                );
            }
        }
        System::SessionLost => destroy_connection(net, lines)?,
        System::Host => {
            vars.own.set_state(vars.own.state().made_host());
            vars.sync_first.set(1);
            vars.stage_item.set(1);
        }
        System::Other => {}
    }
    Ok(())
}
