//! The draw switches, second part (IDA 0x621900 .. 0x621A84): the far clip switch, the group table word, the loading group mask, the SSE and 3DNow switches, and a marks source word. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFlag};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DrawSwitches {
    /// +0x0 (0x621900): `dword_621900`: the far clip / own car switch (`scene::clip_polygon_by_depth`, `race::draw_view_overlays`).
    pub far_on: PlainDword,
    /// +0x4 (0x621904): `dword_621904` (`scene::draw_polygon`).
    pub clears_switch: PlainDword,
    /// +0x8 (0x621908): 11 dwords after the polygon switch: the original's developer front end keeps its mode (0x621920), mipmapping (0x621924), cars (0x621928) and screens (0x62192C) here; the port keeps them in `debug::dev_front_end`, so nothing reads or writes these cells.
    pub unknown_8: [PlainDword; 11],
    /// +0x34 (0x621934): `dword_621934`: the group table word (`scene::list_object_face`).
    pub group_table: PlainDword,
    /// +0x38 (0x621938): 71 dwords after the group table word; the original's developer front end keeps its rally (0x6219B8) and stage (0x6219BC) in them, and a few more cells (0x621A38, 0x621A50) belong to its drawing code; the port keeps those in `debug::dev_front_end`, so nothing here is read or written. (`group_table` is the first of a table indexed by group: `scene::list_object_face` reads it on past the one dword.)
    pub unknown_38: [PlainDword; 71],
    /// +0x154 (0x621A54): `g_loadingGroupMask` (`scene::add_object_shape`).
    pub loading_group_mask: PlainDword,
    /// +0x158 (0x621A58): `g_useSse` (`scene::draw_object`, `scene::draw_polygon`, `math::normalise_vector`, `physics::ground`, `race::frame_draw`).
    pub use_sse: PlainFlag,
    /// +0x15c (0x621A5C): `g_use3dNow` (`scene::draw_object`).
    pub use_3dnow: PlainDword,
    /// +0x160 (0x621A60): `dword_621A60`, `g_useMmx` (shown by the debug Switches tab, `debug::switches`).
    pub use_mmx: PlainDword,
    /// +0x164 (0x621A64): seven dwords after `use_mmx`, unreferenced.
    pub unknown_164: [PlainDword; 7],
    /// +0x180 (0x621A80): `dword_621A80` (`effects::draw_frame`).
    pub marks_source: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl DrawSwitches {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            far_on: PlainDword::new(1),
            clears_switch: PlainDword::new(1),
            unknown_8: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            unknown_38: {
                let mut a: [PlainDword; 71] = ZeroCells::ZERO;
                a[64] = PlainDword::new(0x5C726176);
                a[65] = PlainDword::new(0x74706F73);
                a[66] = PlainDword::new(0x736E6F69);
                a[68] = PlainDword::new(0x64);
                a[69] = PlainDword::new(0x22);
                a
            },
            loading_group_mask: PlainDword::new(0xFFFFFFFF),
            ..ZeroCells::ZERO
        }
    }
}
