//! `effects::draw_glow` (0x4D2DED): unless `dword_93CC70` is 1, render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`), `scene::set_texture` with
//! `dword_5D1E34`, and `effects::draw_turned_sprite` at (`flt_93CC7C` + `flt_93CCA8`,
//! `flt_93CC78` + `flt_93CCAC`, `flt_93CC74`) (st(0) first, the sums unrounded), angle
//! `dword_93CCA4`, colour -1, size 0x10000. A glow over the scene, by the look of it (a
//! reading). (Its first lines, a developer's key for the switch, are jumped over.)
//!
//! Caller: `effects::run_frame` (0x4D1693).

use crate::errors::Refused;
use crate::math::x87;
use crate::scene::matrix3::extended;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `draw_turned_sprite`.
pub fn draw_glow(
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let clipping = &effects.clipping;
    {
        if clipping.glow.off.get() == 1 {
            return Ok(());
        }
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(value, state);
        }
        crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[211].bits());
        let place = [
            x87::add_80(
                clipping.glow.place[2].bits(),
                clipping.glow.offset[0].bits(),
            ),
            x87::add_80(
                clipping.glow.place[1].bits(),
                clipping.glow.offset[1].bits(),
            ),
            extended(clipping.glow.place[0].bits()),
        ];
        crate::effects::draw_turned_sprite::draw_turned_sprite(
            clipping.glow.angle.get(),
            0xFFFF_FFFF,
            0x1_0000,
            place,
            scene,
            effects,
        )?;
    }
    Ok(())
}
