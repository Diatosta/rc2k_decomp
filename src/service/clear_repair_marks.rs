//! Sets every entry of the second flag table (`Service::repair_marks`) to 0, up to its -1 end
//! mark. What the marks mean has not been worked out: the name is a reading.
//!
//! The first entry is cleared without being checked for the end mark. Refuses a table whose end
//! mark is not where it belongs.

use crate::errors::BadInput;
use crate::service::owner::Service;
use crate::service::state::{marks, store_marks};

pub(super) fn clear_repair_marks(service: &Service) -> Result<(), BadInput> {
    let mut t = marks(service);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_marks(service, &t);
    Ok(())
}
