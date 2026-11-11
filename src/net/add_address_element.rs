//! `net::add_address_element` (0x41896E): one more element of the compound address being
//! built (`element_table`, 0x18 bytes each, `elements` of them: a DPCOMPOUNDADDRESSELEMENT,
//! the data type's GUID, the data's size, the data).
//!
//! Takes `data_type`, `size`, `data`.
//! No limit on the count. Caller: `net::build_address` (five times; three at most in one
//! go).
//!
//! The elements are Rust's (`elements`, each its type and its data's
//! bytes, as `net::dp_objects` takes them); the count is kept, the game's table
//! (`element_table`, which held pointers to the data) is not written.

use crate::net::dp_objects::Element;
use crate::net::owner::Net;
use crate::session::Bytes;
use std::sync::Mutex;

/// The elements added since the last `clear`.
static ELEMENTS: Mutex<Vec<Element>> = Mutex::new(Vec::new());

/// The element of type `data_type` with `data`.
pub fn add_address_element(net: &Net, data_type: &Bytes<16>, data: &[u8]) {
    net.address()
        .elements
        .set(net.address().elements.get().wrapping_add(1));
    ELEMENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((data_type.read(), data.to_vec()));
}

/// No elements (the count 0).
pub fn clear(net: &Net) {
    net.address().elements.set(0);
    ELEMENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

/// The elements added.
pub fn elements() -> Vec<Element> {
    ELEMENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}
