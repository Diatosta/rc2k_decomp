//! `effects::draw_lens_flare` (0x4C4ED3): the lens flare, 0x11 parts along the line from the
//! glow's place on the screen (the settings' +0x2758, +0x275C) through the screen's
//! centre. The way out (`flt_91FFDA` x, `flt_91FFDE` -y) also into `flt_92051E` ..
//! (`flt_920526` 0) and made one long there (`scene::normalise_vector3`); its length
//! `flt_91FFD6`; the alpha `flt_9206D6` less the length times `flt_9206D2` (`fistp`, at
//! least 0) times the settings' +0x2754, >> 8. Then for each part k from 0x10 down to 0:
//! its centre (-x, y times `g_marksWaveLevels`[k] plus the centre, unrounded),
//! `effects::draw_flare_part` with corner `dword_92052A`[k], size `dword_9205B2`[k] times
//! `flt_9206DA` (`fistp`), angle 0, colour 0xFFFFFF, that alpha.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `sub_4C4EA2`. Refuses null settings and a part's corner past `g_marksCorners`,
//! before anything.

use crate::effects::draw_flare_part::draw_flare_part;
use crate::effects::paths::Paths;
use crate::errors::{BadInput, Refused};
use crate::math::x87;

const PARTS: u32 = 0x11;

/// The Rust version (see the header).
pub fn draw_lens_flare(paths: &Paths) -> Result<(), Refused> {
    let settings: u32 = crate::effects::owner::effects().setup.settings.is_set.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let marks = crate::effects::setup::marks()?;
    if crate::effects::owner::effects()
        .frame
        .flare_parts
        .corner
        .iter()
        .any(|c| c.get() >= 0x100)
    {
        return Err(BadInput("a flare corner past g_marksCorners").into());
    }
    let [cx, cy] = [
        crate::scene::owner::scene().drawing.scene.view_centre[0].bits(),
        crate::scene::owner::scene().drawing.scene.view_centre[1].bits(),
    ];
    let dx = x87::sub(marks.listener_figures[1].get(), cx);
    let dy = x87::sub(marks.listener_figures[2].get(), cy) ^ 0x8000_0000;
    crate::effects::owner::effects().frame.draw_frame.flare_way[0].set_bits(dx);
    crate::effects::owner::effects().frame.draw_frame.flare_way[1].set_bits(dy);
    crate::effects::owner::effects().frame.flare_parts.unit[0].set_bits(x87::loaded(dx));
    crate::effects::owner::effects().frame.flare_parts.unit[1].set_bits(x87::loaded(dy));
    crate::effects::owner::effects().frame.flare_parts.unit[2].set_bits(0);
    crate::scene::vector3::normalise(&crate::effects::owner::effects().frame.flare_parts.unit);
    let length = crate::math::x87_load::float_hypot(dx, dy);
    crate::effects::owner::effects()
        .frame
        .draw_frame
        .flare_length
        .set_bits(length);
    let alpha = x87::times_subr_to_int(
        length,
        crate::effects::owner::effects().frame.flare_scales.fade[0].bits(),
        crate::effects::owner::effects().frame.flare_scales.fade[1].bits(),
    )
    .max(0);
    let product = u64::from(marks.listener_figures[0].get()) * u64::from(alpha as u32);
    let alpha = (product as u32) >> 8;
    let size_k: u32 = crate::effects::owner::effects()
        .frame
        .flare_scales
        .size_k
        .bits();
    for k in (0..PARTS as usize).rev() {
        let level: u32 = paths.wave_levels.levels[k].get();
        let y = x87::times_plus_80(dy, level, cy, false);
        let x = x87::times_plus_80(dx, level, cx, true);
        draw_flare_part(
            crate::effects::owner::effects().frame.flare_parts.corner[k].get(),
            0,
            alpha,
            x87::int_times_to_int(
                crate::effects::owner::effects().frame.flare_parts.size[k].get() as i32,
                size_k,
            ) as u32,
            0xFF_FFFF,
            [x, y],
        )?;
    }
    Ok(())
}
