//! `net::add_address_element` (0x41896E): one more element of the compound address being
//! built (`unk_52AC48`, 0x18 bytes each, `dword_52AC3C` of them: a DPCOMPOUNDADDRESSELEMENT,
//! the data type's GUID, the data's size, the data).
//!
//! In:  `data_type` (eax: the data type's GUID), `size` (ecx: the data's size), `data` (ebx)
//! Out: nothing (the original leaves edx the element's offset, which its one caller,
//!      `net::build_address`, does not read in Rust; the rest kept)
//! No limit on the count. Caller: `net::build_address` (five times; three at most in one
//! go).

use crate::net::store::NET;
use rc2k_core::net::address;

/// The Rust version.
pub fn add_address_element(data_type: u32, size: u32, data: u32) {
    // Converted: `rc2k_core::net::address` (`element_at`, `element`).
    let offset = address::element_at(NET.elements.get());
    NET.elements.set(NET.elements.get().wrapping_add(1));
    let at = (NET.element_table.addr() as u32).wrapping_add(offset);
    let data_type: [u8; 16] = crate::pointers::read_or_end(data_type, "an address element's type");
    let bytes = address::element(&data_type, size, data);
    if let Err(e) =
        crate::pointers::GamePtr::new(at).write_bytes(&bytes, "the address's element table")
    {
        crate::fatal::raise::end("net::add_address_element", e);
    }
}
