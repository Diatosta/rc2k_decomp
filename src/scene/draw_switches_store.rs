//! The draw switches, second part: the far clip switch, the group table word, the loading group
//! mask, the SSE and 3DNow switches, and a marks source word.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFlag};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct DrawSwitches {
    /// The far clip / own car switch (`scene::clip_polygon_by_depth`,
    /// `race::draw_view_overlays`).
    pub far_on: PlainDword,
    /// The switch `scene::draw_polygon` clears (`clear_a_switch`).
    pub clears_switch: PlainDword,
    /// Eleven dwords after the polygon switch; nothing reads or writes them (the developer
    /// front end keeps its mode, mipmapping, cars and screens in `debug::dev_front_end`).
    pub unknown_8: [PlainDword; 11],
    /// The group table word (`scene::list_object_face`).
    pub group_table: PlainDword,
    /// 71 dwords after the group table word: the rest of the table indexed by group, which
    /// `scene::list_object_face` reads on into from `group_table`; nothing else reads or writes
    /// them (the developer front end's rally and stage live in `debug::dev_front_end`).
    pub unknown_38: [PlainDword; 71],
    /// The loading group mask (`scene::add_object_shape`).
    pub loading_group_mask: PlainDword,
    /// The SSE switch (`scene::draw_object`, `scene::draw_polygon`, `math::normalise_vector`,
    /// `physics::ground`, `race::frame_draw`).
    pub use_sse: PlainFlag,
    /// The 3DNow switch (`scene::draw_object`).
    pub use_3dnow: PlainDword,
    /// The MMX switch (shown by the debug Switches tab, `debug::switches`).
    pub use_mmx: PlainDword,
    /// Seven dwords after `use_mmx`, unreferenced.
    pub unknown_164: [PlainDword; 7],
    /// The marks source word (`effects::draw_frame`).
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
