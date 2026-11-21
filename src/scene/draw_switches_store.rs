//! The draw switches, second part: the far clip switch, the group table and a marks source word.
//! A plain, `Copy` record: held by the scene owner (`scene::owner::Scene::draw_switches`) and
//! copied into the drawing parts' reads when a drawing scope opens.

/// How many words the group table holds: the group word and the 71 after it.
pub const GROUP_TABLE_LEN: usize = 72;

#[derive(Clone, Copy)]
pub struct DrawSwitches {
    /// The far clip / own car switch (`scene::clip_polygon_by_depth`,
    /// `race::draw_view_overlays`).
    pub far_on: u32,
    /// The switch `scene::draw_polygon` clears.
    pub clears_switch: u32,
    /// The table indexed by a face's group (`scene::list_object_face`): word 0 is the group word,
    /// the 71 after it are the rest of the table, which nothing but the developers' page writes
    /// (the developer front end's rally and stage live in `debug::dev_front_end`).
    pub group_table: [u32; GROUP_TABLE_LEN],
    /// The marks source word (`effects::draw_frame`).
    pub marks_source: u32,
}

impl DrawSwitches {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        let mut group_table = [0; GROUP_TABLE_LEN];
        group_table[65] = 0x5C72_6176;
        group_table[66] = 0x7470_6F73;
        group_table[67] = 0x736E_6F69;
        group_table[69] = 0x64;
        group_table[70] = 0x22;
        Self {
            far_on: 1,
            clears_switch: 1,
            group_table,
            marks_source: 0,
        }
    }
}
