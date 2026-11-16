use crate::scene::owner::Scene;
/// Draws the polygon's fan while the draw switch is on.
pub fn draw_fan(
    part: &crate::scene::workers::PartScratch,
    scene: &Scene,
    poly: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
) {
    if scene.objects.tables.get().d3d_drawing_on.get() == 0 {
        return;
    }
    crate::d3d::draw_fan::draw_fan(part, poly, buffer);
}
