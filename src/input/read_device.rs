//! `input::read_device` (0x408E38): a device's four readings from its bindings (see
//! `input/devices.rs`). The device polled (`sub_408869`: its state copied into +0x2C ..
//! +0x38); while it is read (+0x414 1):
//! - the bits: the hat's angle (`input::hat_reading`) in the top byte, its direction
//!   (`input::hat_direction`) in the second;
//! - steering left (binding +0x41C) and right (+0x42C), throttle (+0x43C), brake (+0x44C):
//!   each its key down (`g_keysDown`), or its axis (the record's dword at the binding's
//!   place times its sign, below 0 made 0) as it is, or, with a button bound too, all of it
//!   when that is above 0x40 and the button down, or the button alone: all of it is
//!   0xFFFFFF00 for left (whose axis is negated), 0x100 for the others, and sets bit 0x1000
//!   (steering), 0x4000 (throttle), 0x2000 (brake); the steering is left plus right, and
//!   with bit 0x1000 set held to 0xFFFFFF00 or 0x100 by its sign; on a digital device (+8
//!   1) bits 0x7000 and every reading not 0 made all of it;
//! - eight buttons (+0x45C .. +0x4CC) as bits 1 .. 0x80: the key down, or the axis's
//!   value above 0x40 (with a button bound, that button down as well), or the button;
//!
//! Into +0x4DC (steering), +0x4E0 (brake), +0x4E4 (throttle), +0x4E8 (the bits).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_408A56` (both devices, each tick of the input thread). Refuses (before
//! anything) a bound key past 0xFF and an axis's place past the record.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::devices::put;
use crate::input::devices::{joy_state, key_table, record_bytes};

/// The device `record` read (see the header). Converted: `rc2k_core::input::devices::read_device`; the bindings are checked before the poll,
/// the record read again after it.
pub fn read_device(record: u32) -> Result<(), Refused> {
    use rc2k_core::input::devices;
    devices::check_bindings(&record_bytes(record)?)?;
    { crate::input::poll_device::poll_device(record) }?;
    let Some(r) = devices::read_device(&record_bytes(record)?, &key_table(), &joy_state())? else {
        return Ok(());
    };
    let readings = devices::READINGS as u32;
    put(record, readings, r.steering)?;
    put(record, readings + 4, r.brake)?;
    put(record, readings + 8, r.throttle)?;
    put(record, devices::BITS as u32, r.bits)?;
    Ok(())
}
