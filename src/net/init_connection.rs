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
use crate::net::dplay::{INITIALIZE_CONNECTION, call};
use crate::net::lobby::{ADDRESS, DPLAY, a, g, log, log_hex, log_int_nl};

const TEXT: usize = 0x5C2E80; // "NET: Init connection index"
const INIT: usize = 0x5C2E9B; // "NET: Init"

/// The Rust version.
pub fn init_connection(connection: u32) -> Result<bool, Refused> {
    log(a(TEXT), false, 2);
    log_int_nl(connection, 2);
    if g(DPLAY) == 0 || !build_address(connection)? {
        return Ok(false);
    }
    let result = unsafe { call(DPLAY, INITIALIZE_CONNECTION, &[g(ADDRESS), 0]) };
    log(a(INIT), false, 2);
    log_hex(result, true);
    Ok(result == 0)
}
