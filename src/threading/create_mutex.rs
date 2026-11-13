//! A mutex without a name and without an owner. Gives the handle, or none; the caller
//! (`net::set_up`) raises fatal error 0x80168502 on none.

/// The handle, 0 for none.
pub fn create_mutex() -> u32 {
    crate::sync::create_mutex()
}
