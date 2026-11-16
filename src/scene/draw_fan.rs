use crate::scene::owner::Scene;
const DRAWTRI: &std::ffi::CStr = c"drawtri";

/// The device's result is read by none of its callers.
pub fn draw_fan(
    part: &crate::scene::workers::PartScratch,
    scene: &Scene,
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
) {
    if scene.objects.tables.get().d3d_drawing_on.get() == 0 {
        return;
    }
    let result = crate::d3d::draw_fan::draw_fan(part, poly, buffer);
    if result != 0 {
        crate::logfile::log_write::log_nul(&DRAWTRI, false, 1);
        crate::logfile::log_hex8::log_hex8(result, false, 1);
    }
}
