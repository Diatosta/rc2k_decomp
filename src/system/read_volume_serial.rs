//! `system::read_volume_serial` (0x403131): the current drive's serial number and volume
//! name (`GetVolumeInformationA` with no root), into the first 16 bytes of `g_pngInfo`
//! (0x93E8B0, a scratch place here): +0 the serial, +4 the name in 12 bytes, +0xF made 0.
//!
//! The original pushes esi and ends with `pop eax`, so the serial meant for the result is lost and
//! the caller's own value comes back (`notes/bugs.md`). Worked: the three dwords at +4, +8, +0xC
//! are returned and the serial may be 0. Failed: they are 0. One caller, `start` (0x40100B), before
//! `math::stir_random`, which stirs in eax. The Win32 call is made from Rust.
//! `start` calls it directly (2026-10-23).

// ---- game data ----

// ---- Rust version ----

/// The serial and name written (see the header). Its one caller, `start`, reads eax
/// after it (register liveness, 2026-10-23: eax only), which is `start`'s own esi at
/// entry, not anything of this function's: so it returns nothing.
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
