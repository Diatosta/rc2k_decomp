//! A square icon of a view's start lights (`Panel::lights`; none while `lit` is -1): its width
//! and drop times the icon scale (rounded into `icon_size`), centred on 0x320 / 2 plus the icon
//! shift, its top the row plus the drop, as tall as wide; texture places 0 / 1; colour white
//! under its alpha; `hud::add_polygon` on four.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::race::owner_frame::Frame;
use crate::race::panel_store::Lights;

use rc2k_core::hud::icons;

/// The icon of `lights`, `bottom` its row.
pub fn draw_icon(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    lights: &Lights,
    bottom: u32,
) -> Result<(), Refused> {
    if !icons::shown(lights.lit.get()) {
        return Ok(());
    }
    let scale = hud.block.icon_scale.get();
    let width = lights.width.get();
    hud.block.icon_size[0].set(icons::scaled(width, scale));
    let drop = lights.drop.get();
    hud.block.icon_size[1].set(icons::scaled(drop, scale));
    let i = icons::icon(
        width,
        lights.alpha.get(),
        drop,
        scale,
        hud.block.icon_shift.get(),
        bottom,
    );
    for (n, u, v, x, y) in icons::corners(&i) {
        let at = &pen.vertices.vertices[n];
        at.u.set(u as i32 as f32);
        at.v.set(v as i32 as f32);
        at.x.set(x as i32 as f32);
        at.y.set(y as i32 as f32);
        at.colour.set(i.colour);
        at.specular.set(0);
        // The scratch cell keeps the last value stored.
        hud.block.fistp_scratch.set(y);
    }
    pen.polygons.polygon_vertex_count.set(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud, pen)
}
