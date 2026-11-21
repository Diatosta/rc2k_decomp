//! `effects::draw_lens_flare`: the lens flare, 0x11 parts along the line from the glow's place on
//! the screen (the settings' `listener_figures[1]` and `[2]`) through the screen's centre. The way
//! out (x, -y) also goes into the flare's work vector (z 0) and is made one long there
//! (`scene::vector3::normalise`); its length is kept; the alpha = the base alpha less the length
//! times a factor (rounded, at least 0) times the settings' `listener_figures[0]` (the flare's
//! strength), >> 8. Then for each part k from 0x10 down to 0: its centre (-x, y times the wave
//! level k plus the centre, unrounded), `effects::draw_flare_part` with the part's corner and size
//! (times a factor, rounded), angle 0, colour 0xFFFFFF, that alpha.
//!
//! Refuses null settings and a part's corner past the mark corners table, before anything.

use crate::effects::draw_flare_part::draw_flare_part;
use crate::effects::paths_store::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;

const PARTS: u32 = 0x11;

pub fn draw_lens_flare(
    engine_tables: crate::math::engine_tables::EngineTablesRef,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    paths: &Paths,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let settings: u32 = effects.setup.settings.is_set.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let marks = effects.settings()?;
    if effects
        .frame
        .flare_parts
        .borrow()
        .corner
        .iter()
        .any(|&c| c >= 0x100)
    {
        return Err(BadInput("a flare corner past g_marksCorners").into());
    }
    let [cx, cy] = [
        scene
            .scratch
            .front_end_polygon()
            .window
            .borrow()
            .view_centre[0],
        scene
            .scratch
            .front_end_polygon()
            .window
            .borrow()
            .view_centre[1],
    ];
    let dx = x87::sub(marks.listener_figures[1].get(), cx);
    let dy = -x87::sub(marks.listener_figures[2].get(), cy);
    effects.frame.draw_frame.borrow_mut().flare_way = [dx, dy];
    {
        let mut parts = effects.frame.flare_parts.borrow_mut();
        parts.unit = [x87::loaded(dx), x87::loaded(dy), 0.0];
        crate::scene::vector3::normalise_plain(&mut parts.unit);
    }
    let length = crate::math::x87::float_hypot(dx, dy);
    effects.frame.draw_frame.borrow_mut().flare_length = length;
    let scales = effects.frame.flare_scales.borrow();
    let alpha = bh::to_int(bh::ld(scales.fade[1]) - bh::ld(length) * bh::ld(scales.fade[0])).max(0);
    let product = u64::from(marks.listener_figures[0].get()) * u64::from(alpha as u32);
    let alpha = (product as u32) >> 8;
    let size_k = scales.size_k;
    let parts = effects.frame.flare_parts.borrow();
    for k in (0..PARTS as usize).rev() {
        let level: u32 = paths.wave_levels.borrow().levels[k];
        let y = x87::times_plus_80(dy, level, cy, false);
        let x = x87::times_plus_80(dx, level, cx, true);
        draw_flare_part(
            engine_tables,
            buffer,
            parts.corner[k],
            0,
            alpha,
            x87::int_times_to_int(parts.size[k] as i32, size_k) as u32,
            0xFF_FFFF,
            [x, y],
            scene,
            effects,
        )?;
    }
    Ok(())
}
