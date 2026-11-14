//! Room for 0x100 connections and 0x100 sessions on the network's memory list; both counts
//! start at 0. An allocation failure would be fatal error 0x80E68021 (connections) or
//! 0x80E68022 (sessions).

use crate::errors::Refused;
use crate::net::link::Link;

/// The fatal errors are not reached: `memory::allocate` refuses rather than fail.
pub fn allocate_lists(link: &Link) -> Result<(), Refused> {
    link.take_tables()
}
