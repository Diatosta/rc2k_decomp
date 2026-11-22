//! Clears the 7 contact records of 0x20 bytes (`contacts`, the step's contact forces) with
//! `physics::clear_records`.

use crate::errors::Refused;
use crate::physics::clear_records::clear_records;
use crate::physics::tables_store::PhysicsTables;
pub(super) fn clear_contacts(tables: &mut PhysicsTables) -> Result<(), Refused> {
    clear_records(&mut tables.contacts);
    Ok(())
}
