//! One more element of the compound DirectPlay address being built: a data type, its size and
//! its bytes. At most three are added in one go; there is no limit check.

use crate::net::dp_objects::Element;
use crate::net::owner::Net;
/// The element of type `data_type` with `data`.
pub fn add_address_element(net: &Net, data_type: &[u8; 16], data: &[u8]) {
    net.address()
        .elements
        .set(net.address().elements.get().wrapping_add(1));
    net.link.push_element((*data_type, data.to_vec()));
}

/// No elements (the count 0).
pub fn clear(net: &Net) {
    net.address().elements.set(0);
    net.link.clear_elements();
}

/// The elements added.
pub fn elements(net: &Net) -> Vec<Element> {
    net.link.elements()
}
