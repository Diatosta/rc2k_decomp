//! `system::read_volume_serial` (0x403131): the current drive's serial number and volume
//! name (`GetVolumeInformationA` with no root), into the first 16 bytes of `g_pngInfo`
//! (0x93E8B0, a scratch place here): +0 the serial, +4 the name in 12 bytes, +0xF made 0.
//!
//! The original ends by restoring a saved register over the result, so the serial meant for
//! the result is lost and the caller's own value comes back (`notes/bugs.md`). Worked: the three dwords at +4, +8, +0xC
//! are returned and the serial may be 0. Failed: they are 0. One caller, `start`, before
//! `math::stir_random`, which stirs in the value that comes back. The Win32 call is made from Rust.
//! `start` calls it directly.

// ---- game data ----

/// The serial and name written (see the header). Its one caller, `start`, uses the
/// result register after it, which holds the caller's own value from before the call, not
/// anything of this function's: so it returns nothing.
pub fn read_volume_serial(system: &crate::system::owner::System) {
    let Some((serial, name)) = crate::os::volume() else {
        return;
    };
    let mut info = [0u8; 16];
    info[..4].copy_from_slice(&serial.to_le_bytes());
    info[4..].copy_from_slice(&name);
    // Converted: `rc2k_core::system::machine::volume_registers` (only its writing of +0xF
    // matters here).
    rc2k_core::system::machine::volume_registers(&mut info);
    system
        .work
        .files
        .buffer
        .write_at(0, &info, "the scratch buffer")
        .unwrap_or_else(|e| crate::fatal::raise::end("system::read_volume_serial", e));
}
