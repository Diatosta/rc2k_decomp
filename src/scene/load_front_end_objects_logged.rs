//! `scene::load_front_end_objects_logged` (0x4433DA): `scene::load_front_end_objects`,
//! then its log: a rule, "Front end objects loaded", "total points" `g_objectPointCount`,
//! "total pointmem" `dword_634FA4`, "total faces" `dword_5CD67C`, "total facemem"
//! `dword_634FA8` (each label without a line's end, its value in hex8 with one), a rule.
//!
//! In:  nothing
//! Out: nothing its caller reads (`load_front_end_objects`' leftovers are read by none)
//!
//! Caller: `scene::set_up_front_end_scene`.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::logfile::{log_hex8::log_hex8, log_write::log_write};

const RULE: &std::ffi::CStr = c"===========================";
const LOADED: &std::ffi::CStr = c"Front end objects loaded";
const RULE_2: &std::ffi::CStr = c"===========================";
/// (label, value)
const TOTALS: [(&std::ffi::CStr, usize); 4] = [
    (c"total points", 0x5CD680),   // "total points", g_objectPointCount
    (c"total pointmem", 0x634FA4), // "total pointmem"
    (c"total faces", 0x5CD67C),    // "total faces"
    (c"total facemem", 0x634FA8),  // "total facemem"
];

/// The Rust version.
pub unsafe fn load_front_end_objects_logged() -> Result<(), Refused> {
    unsafe {
        crate::scene::load_front_end_objects::load_front_end_objects()?;
        let text = |s: &std::ffi::CStr| s.as_ptr().cast::<u8>();
        log_write(text(RULE), true, 1);
        log_write(text(LOADED), true, 1);
        for (label, value) in TOTALS {
            log_write(text(label), false, 1);
            log_hex8(game_read(rva(value)), true, 1);
        }
        log_write(text(RULE_2), true, 1);
    }
    Ok(())
}
