//! Builds the connection's address and sets DirectPlay to it ("NET: Init connection").
//! Returns true when made.

use crate::errors::Refused;
use crate::net::build_address::build_address;
use crate::net::lobby::{log_bytes, log_hex, log_int_nl};
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{NetTexts, texts};
pub(super) fn init_connection(
    net: &Net,
    lines: &NetTexts,
    connection: u32,
) -> Result<bool, Refused> {
    let link = &net.link;
    log_bytes(&lines.text(texts::NET_INIT_CONNECTION_INDEX), false, 2);
    log_int_nl(connection, 2);
    if !link.has_dplay() || !build_address(net, lines, connection)? {
        return Ok(false);
    }
    let result = {
        let address = link.tables().with(|t| t.address.clone())?;
        crate::net::dp_objects::initialize_connection_with(
            link.dplay(),
            (!address.is_empty()).then_some(&address[..]),
        )
    };
    log_bytes(&lines.text(texts::NET_INIT), false, 2);
    log_hex(result, true);
    Ok(result == 0)
}
