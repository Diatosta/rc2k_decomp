//! `hud::draw_icon` (0x4B587E): a square icon of the HUD record's entry at ebp + edi
//! (none when its +0x84 is -1): its width (+0x88) and drop (+0x90) times `dword_75C06C`
//! (`fistp` into `dword_75C064` / `dword_75C068`), centred on 0x320 / 2 plus
//! `dword_75C070`, its top ebx plus the drop, as tall as wide; texture places 0 / 1;
//! colour white under its +0x8C; `hud::add_polygon` on four.
//!
//! Takes: the record, the entry's offset, the top
//! Out: ebx the bottom, edx the colour, esi the left, edi the right (kept when none)
//!
//! Callers: `hud::icons`.

use crate::d3d::vertices_store::VERTICES;
use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;

use crate::scene::owner::scene;
use rc2k_core::hud::icons;

/// The Rust version (see the header): the icon of the record's entry at `entry` (edi),
/// `bottom` (ebx) its row.
// Converted: `rc2k_core::hud::icons`
pub fn draw_icon(
    frame: &Frame,
    hud: &Hud,
    record: GamePtr,
    bottom: u32,
    entry: u32,
) -> Result<(), Refused> {
    let e = record.offset(entry);
    // A field of the record by an offset from a table or the caller, as the original walks it.
    let get = |o: u32| e.offset(o).read::<u32>("a hud record's field");
    if !icons::shown(get(0x84)?) {
        return Ok(());
    }
    let scale = hud.block.icon_scale.get();
    let width = get(0x88)?;
    hud.block.icon_size[0].set(icons::scaled(width, scale));
    let drop = get(0x90)?;
    hud.block.icon_size[1].set(icons::scaled(drop, scale));
    let i = icons::icon(
        width,
        get(0x8C)?,
        drop,
        scale,
        hud.block.icon_shift.get(),
        bottom,
    );
    for (n, u, v, x, y) in icons::corners(&i) {
        let at = &VERTICES.vertices[n];
        at.u.set_bits((u as i32 as f32).to_bits());
        at.v.set_bits((v as i32 as f32).to_bits());
        at.x.set_bits((x as i32 as f32).to_bits());
        at.y.set_bits((y as i32 as f32).to_bits());
        at.colour.set(i.colour);
        at.specular.set(0);
        // `fild` through `dword_75C060`: it keeps the last.
        hud.block.scratch.set(y);
    }
    scene().drawing.scene.polygon_vertex_count.set_bits(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud)
}
