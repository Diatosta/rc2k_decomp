//! `net::init_connection` (0x418794): "NET: Init connection index" and the number logged;
//! with DirectPlay's object there, connection eax's address built (`net::build_address`)
//! and DirectPlay set to it (`InitializeConnection`, `dword_52AC40`, flags 0); "NET: Init"
//! and its result logged.
//!
//! In:  `connection` (eax: the connection's number)
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Caller: `net::connecting`.

use crate::errors::Refused;
use crate::net::build_address::build_address;
use crate::net::lobby::{log, log_hex, log_int_nl};
use crate::net::owner::net;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version.
pub fn init_connection(connection: u32) -> Result<bool, Refused> {
    let vars = &net().vars;
    log(
        VIEWER_PANEL.text_ptr(texts::NET_INIT_CONNECTION_INDEX),
        false,
        2,
    );
    log_int_nl(connection, 2);
    if vars.dplay.get() == 0 || !build_address(connection)? {
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
