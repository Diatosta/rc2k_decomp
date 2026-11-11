//! `net::init_connection` (0x418794): "NET: Init connection index" and the number logged;
//! with DirectPlay's object there, the connection's address built (`net::build_address`)
//! and DirectPlay set to it (`InitializeConnection`, `address`, flags 0); "NET: Init"
//! and its result logged.
//!
//! Takes `connection`. Returns true when made.
//! Caller: `net::connecting`.

use crate::errors::Refused;
use crate::net::build_address::build_address;
use crate::net::lobby::{log, log_hex, log_int_nl};
use crate::net::owner::Net;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};
pub fn init_connection(net: &Net, connection: u32) -> Result<bool, Refused> {
    let vars = &net.vars;
    log(
        VIEWER_PANEL.text_ptr(texts::NET_INIT_CONNECTION_INDEX),
        false,
        2,
    );
    log_int_nl(connection, 2);
    if vars.dplay.get() == 0 || !build_address(net, connection)? {
        return Ok(false);
    }
    let result = {
        let address = crate::net::tables::NET_TABLES.with(|t| t.address.clone())?;
        crate::net::dp_objects::initialize_connection_with(
            vars.dplay.get(),
            (!address.is_empty()).then_some(&address[..]),
        )
    };
    log(VIEWER_PANEL.text_ptr(texts::NET_INIT), false, 2);
    log_hex(result, true);
    Ok(result == 0)
}
