//! `net::add_address_element` (0x41896E): one more element of the compound address being
//! built (`unk_52AC48`, 0x18 bytes each, `dword_52AC3C` of them: a DPCOMPOUNDADDRESSELEMENT,
//! the data type's GUID, the data's size, the data).
//!
//! In:  `data_type` (eax: the data type's GUID), `size` (ecx: the data's size), `data` (ebx)
//! Out: nothing (the original leaves edx the element's offset, which its one caller,
//!      `net::build_address`, does not read in Rust; the rest kept)
//! No limit on the count. Caller: `net::build_address` (five times; three at most in one
//! go).
//!
//! Since 2026-10-29 the elements are Rust's (`elements`, each its type and its data's
//! bytes, as `net::dp_objects` takes them); the count is kept, the game's table
//! (`unk_52AC48`, which held pointers to the data) is no longer written.

use crate::net::dp_objects::Element;
use crate::net::store::NET;
use crate::session::Bytes;
use std::sync::Mutex;

/// The elements added since the last `clear`.
static ELEMENTS: Mutex<Vec<Element>> = Mutex::new(Vec::new());

/// The Rust version: the element of type `data_type` with `data`.
pub fn add_address_element(data_type: &Bytes<16>, data: &[u8]) {
    NET.elements.set(NET.elements.get().wrapping_add(1));
    ELEMENTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push((data_type.read(), data.to_vec()));
}

/// No elements (the count 0).
pub fn clear() {
    NET.elements.set(0);
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
