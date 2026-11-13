//! One more element of the compound DirectPlay address being built: a data type, its size and
//! its bytes. At most three are added in one go; there is no limit check.

use crate::net::dp_objects::Element;
use crate::net::owner::Net;
use std::sync::Mutex;

/// The elements added since the last `clear`.
static ELEMENTS: Mutex<Vec<Element>> = Mutex::new(Vec::new());

/// The element of type `data_type` with `data`.
pub fn add_address_element(net: &Net, data_type: &[u8; 16], data: &[u8]) {
    net.address()
        .elements
        .set(net.address().elements.get().wrapping_add(1));
    ELEMENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((*data_type, data.to_vec()));
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
