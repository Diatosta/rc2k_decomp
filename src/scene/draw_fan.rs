use crate::scene::owner::scene;
const DRAWTRI_RVA: &std::ffi::CStr = c"drawtri"; // aDrawtri_0

/// What it leaves in the registers (eax the device's
/// result, the zero and carry flags) is read by none of its callers (register liveness,
/// 2026-10-23: the 24 sites through 0x44EA65 read only SF, which the original's `cmp`
/// leaves and the Rust version never set).
pub fn draw_fan(poly: &crate::scene::polygon_work::PolygonWork) {
    if scene().objects.tables.has_3d.get() == 0 {
        return;
    }
    let result = crate::d3d::draw_fan::draw_fan(poly);
    if result != 0 {
        crate::logfile::log_write::log_nul(&DRAWTRI_RVA, false, 1);
        crate::logfile::log_hex8::log_hex8(result, false, 1);
    }
}
