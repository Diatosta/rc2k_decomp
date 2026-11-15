//! The current drive's serial number and volume name, into the first 16 bytes of a scratch
//! place: +0 the serial, +4 the name in 12 bytes, +0xF made 0. Gives the three dwords at +4, +8,
//! +0xC (0 when the query failed). The serial itself is not given back (a slip kept as is).
//! Used by `start`, before `math::stir_random`, which stirs in the value that comes back.

/// The serial and name written (see the header). Its one caller, `start`, uses the
/// result register after it, which holds the caller's own value from before the call, not
/// anything of this function's: so it returns nothing.
pub fn read_volume_serial(system: &crate::system::owner::System) {
    let Some((serial, name)) = crate::platform::os::volume() else {
        return;
    };
    let mut info = [0u8; 16];
    info[..4].copy_from_slice(&serial.to_le_bytes());
    info[4..].copy_from_slice(&name);
    rc2k_core::system::machine::volume_registers(&mut info);
    system
        .work()
        .files
        .buffer
        .write_at(0, &info, "the scratch buffer")
        .unwrap_or_else(|e| crate::fatal::raise::end("system::read_volume_serial", e));
}
