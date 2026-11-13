//! Sets every entry of the second flag table (`inspection_flags_b`) to 0, up to its -1 end
//! mark. What the marks mean has not been worked out: the name is a reading.
//!
//! The first entry is cleared without being checked for the end mark. Refuses a table whose end
//! mark is not where it belongs.

use crate::errors::BadInput;
use crate::menu::owner::Tables;
use crate::service::state::{marks, store_marks};

pub fn clear_repair_marks(tables: &Tables) -> Result<(), BadInput> {
    let mut t = marks(tables);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_marks(tables, &t);
    Ok(())
}
