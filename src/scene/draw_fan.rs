use crate::scene::object_tables_store::OBJECT_TABLES;

const DRAWTRI_RVA: &std::ffi::CStr = c"drawtri"; // aDrawtri_0

/// The Rust version (see the header). What it leaves in the registers (eax the device's
/// result, the zero and carry flags) is read by none of its callers (register liveness,
/// 2026-10-23: the 24 sites through 0x44EA65 read only SF, which the original's `cmp`
/// leaves and the Rust version never set).
pub fn draw_fan() {
    if OBJECT_TABLES.has_3d.get() == 0 {
        return;
    }
    let result = crate::d3d::draw_fan::draw_fan();
    if result != 0 {
        crate::logfile::log_write::log_nul(&DRAWTRI_RVA, false, 1);
        crate::logfile::log_hex8::log_hex8(result, false, 1);
    }
}
