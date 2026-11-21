//! Gives the network's memory back and zeroes the connection and session counts and tables.
//! Refuses what `memory::free_list` refuses.

use crate::errors::Refused;
use crate::net::owner::Net;
pub(super) fn free_messages(net: &Net) -> Result<(), Refused> {
    { crate::memory::free_list::free_list(net.link.memory()) }?;
    net.link.give_back_tables();
    Ok(())
}
