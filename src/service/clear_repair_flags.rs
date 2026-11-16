//! Sets every entry of the flag table (`Service::repair_flags`) to 0, up to its -1 end mark:
//! nothing is chosen any more.
//!
//! The first entry is cleared without being checked for the end mark. Refuses a table whose end
//! mark is not where it belongs.

use crate::errors::BadInput;
use crate::service::owner::Service;
use crate::service::state::{flags, store_flags};

pub fn clear_repair_flags(service: &Service) -> Result<(), BadInput> {
    let mut t = flags(service);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_flags(service, &t);
    Ok(())
}
