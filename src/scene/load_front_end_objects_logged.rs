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
use crate::logfile::log_hex8::log_hex8;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;

const RULE: &std::ffi::CStr = c"===========================";
const LOADED: &std::ffi::CStr = c"Front end objects loaded";
const RULE_2: &std::ffi::CStr = c"===========================";
/// (label, value)
type Total = (&'static std::ffi::CStr, fn() -> u32);
const TOTALS: [Total; 4] = [
    // "total points", g_objectPointCount
    (c"total points", || OBJECT_TABLES.point_count.get()),
    (c"total pointmem", || OBJECT_WORK.point_memory_total.bits()),
    (c"total faces", || OBJECT_TABLES.face_count.get()),
    (c"total facemem", || OBJECT_WORK.face_memory_total.bits()),
];

/// The Rust version.
pub fn load_front_end_objects_logged() -> Result<(), Refused> {
    crate::scene::load_front_end_objects::load_front_end_objects()?;
    let text = |s: &'static std::ffi::CStr| s;
    crate::logfile::log_write::log_nul(text(RULE), true, 1);
    crate::logfile::log_write::log_nul(text(LOADED), true, 1);
    for (label, value) in TOTALS {
        crate::logfile::log_write::log_nul(text(label), false, 1);
        log_hex8(value(), true, 1);
    }
    crate::logfile::log_write::log_nul(text(RULE_2), true, 1);
    Ok(())
}
