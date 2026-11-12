//! `effects::draw_glow` (0x4D2DED): unless the glow switch is 1, render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 0, 0x1B = 1 (`scene::set_render_state`), `scene::set_texture` with the
//! glow texture, and `effects::draw_turned_sprite` at the glow's place plus its offsets (st(0)
//! first, the sums unrounded), its angle, colour -1, size 0x10000. A glow over the scene, by the
//! look of it (a reading). (Its first lines, a developer's key for the switch, are jumped
//! over.)//!
//! Caller: `effects::run_frame`.

use crate::errors::Refused;
use crate::math::x87;
use crate::scene::matrix3::extended;
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (0, 0x1C), (1, 0x1B)];

/// The render states (`scene::set_render_state`), the
/// texture (`scene::set_texture`), `draw_turned_sprite`.
pub fn draw_glow(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let part = crate::workers::game_scratch();
    let clipping = &effects.clipping;
    {
        if clipping.glow.off.get() == 1 {
            return Ok(());
        }
        for (value, state) in STATES {
            crate::scene::set_render_state::set_render_state(part, value, state);
        }
        crate::scene::set_texture::set_texture(
            part,
            scene,
            scene.textures.kinds.get().kind_first_slots[211].get(),
        );
        let place = [
            x87::add_80(&clipping.glow.place[2], &clipping.glow.offset[0]),
            x87::add_80(&clipping.glow.place[1], &clipping.glow.offset[1]),
            extended(clipping.glow.place[0].get()),
        ];
        crate::effects::draw_turned_sprite::draw_turned_sprite(
            engine_tables,
            buffer,
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
