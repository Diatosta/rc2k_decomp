//! The view and camera variables of the race: the drawn car, the camera's figures, the view
//! place and angles (int and float) and the view kind, the per-view hide table (`VIEW_HIDES`); then
//! the 52 view records (`ViewRecord`). The record fields' short names are the ones on the kstar
//! camera page: `xoff`, `xfoc`, `xang`, `mspx`, `fish`.

use crate::memory::session::ConstFloat;
use crate::race::groups_store::ViewAndRecords;

/// The view variables: the shared ones (`ViewAndRecords::view_vars`, which only the game's
/// thread writes) and a view's copy (`ViewVarsCopy`), which only the view's drawing part
/// reaches.
#[derive(Clone, Copy, rc2k_derive::PlainRecord)]
pub struct ViewVars {
    /// The car block's `camera_switch_share` copied at the frame start (`race::frame_draw`);
    /// `race::car_draw` tests it against the drawn car, so it marks which car the view follows.
    pub view_car_flag_a: f32,
    /// The car block's `camera_view` copied at the frame start; the car's index into the
    /// stage records (`race::car_draw`, `race::frame_draw`).
    pub view_car_stage: u32,
    /// The car block's `camera_mode_is_2_or_3` copied at the frame start
    /// (`race::frame_draw`); read by `camera_view` and written by `race::car_light_modes`.
    pub view_car_flag_b: u32,
    /// The car being drawn (`race::car_draw`, `scene::set_object_light`,
    /// `scene::list_object_face`).
    pub drawn_car: u32,
    /// Camera height bias (`physics::camera_height_bias`).
    pub camera_height_bias: u32,
    /// The camera height bias's scale (`physics::camera_height_bias`).
    pub camera_bias_scale: f32,
    /// Held to 0 ..= 1 (`physics::camera_height_bias`).
    pub camera_bias_limit: f32,
    /// A float, the far/scale (`scene::set_view_window`, `race::draw_mirror_view`,
    /// `race::race_end`).
    pub view_scale: f32,
    /// `view_place`: the view place x, y, z (`scene::view_matrix`, `race::frame_draw`,
    /// `drive::set_listener`). The three are written as a run.
    pub view_place: [u32; 3],
    /// The place as floats (`race::course_faces`, `race::frame_draw`).
    pub view_place_f: [f32; 3],
    /// View x (`race::frame_draw`).
    pub view_x: u32,
    /// View z (`race::frame_draw`).
    pub view_z: u32,
    /// `view_angles`: y, x, z then a second triple (`scene::view_matrix`,
    /// `race::frame_draw`).
    pub view_angles: [u32; 6],
    /// The camera's zoom hold (`physics::update_camera`).
    pub camera_zoom_hold: f32,
    /// The camera vector (three floats) whose length `physics::update_camera` takes.
    pub camera_vector: [f32; 3],
    /// Held to 0 ..= 1 (`physics::camera_height_bias`, `physics::update_camera`).
    pub camera_distance: f32,
    /// The view kind, the camera mode number (`drive::set_camera_view`;
    /// `race::frame_draw::scene_objects` and `car_draw` branch on it).
    pub view_kind: u32,
}

/// A view record (0x5C bytes; 52 of them, `ViewAndRecords::records`): one
/// camera mode's figures. The three-letter names are the developers' (the kstar camera page).
/// Records 1 .. 3 are refilled from the car's light record (`physics::update_camera`), 4 .. 6
/// placed by `physics::place_camera`, 6 by the finish camera too.
#[derive(Clone, Copy, rc2k_derive::PlainRecord)]
pub struct ViewRecord {
    /// The mode's kind (what a replay camera wish stops at: `drive::pick_replay_camera`).
    pub kind: u32,
    /// `xoff`, `yoff`, `zoff`: the camera's offset (floats; `take_first_view` /
    /// `take_second_view` add the camera frame to it).
    pub offset: [u32; 3],
    /// `xfoc`, `yfoc`, `zfoc`: the focus (floats). The next eleven figures are the ones the
    /// views take whole (`take_first_view` / `take_second_view`).
    pub focus: [u32; 3],
    /// `xang`, `yang`, `zang`: the angles.
    pub angles: [u32; 3],
    /// `mspx`, `mspy`, `mspz` (floats; the views' shares).
    pub speeds: [u32; 3],
    /// The record's dword before `fish`; copied with the rest of the record and never
    /// used otherwise.
    pub reserved_34: u32,
    /// `fish` (a whole number).
    pub fish: u32,
    /// How long a change into this mode takes (ticks; `physics::camera_view::share_done`).
    pub change_time: u32,
    /// The three turn figures eased in over a change (floats; `ease_view_angles`).
    pub turn_rates: [u32; 3],
    /// Nonzero: the car's pitch and roll halved and the camera kept off the ground
    /// (`take_first_view`, `camera_avoid_ground`).
    pub halve_tilt: u32,
    /// Nonzero: the zoom held at 0x180 (`physics::update_camera`).
    pub fixed_zoom: u32,
    /// The height bias's figure (`camera_height_bias`; sign read).
    pub height_bias: u32,
    /// The blend's figure (`ease_view_blend`; sign read).
    pub blend: u32,
}

impl ViewRecord {
    /// The offset's three dwords.
    pub fn offset_bits(&self) -> [u32; 3] {
        self.offset
    }

    /// The eleven dwords the views take whole (`focus`, angles, speeds, one more figure, fish).
    pub fn taken(&self) -> [u32; 11] {
        let [f, a, s] = [self.focus, self.angles, self.speeds];
        [
            f[0],
            f[1],
            f[2],
            a[0],
            a[1],
            a[2],
            s[0],
            s[1],
            s[2],
            self.reserved_34,
            self.fish,
        ]
    }
}

impl ViewVars {
    /// View record `n` by its place among the 52 (records 1 .. 3 are refilled from the car, 4 .. 6
    /// placed).
    pub fn record(view_records: &ViewAndRecords, n: usize) -> &ViewRecord {
        &view_records.records[n]
    }

    /// View record `n`, to write.
    pub fn record_mut(view_records: &mut ViewAndRecords, n: usize) -> &mut ViewRecord {
        &mut view_records.records[n]
    }

    /// View record `n`. A view past the 52 is refused.
    pub fn record_of(
        view_records: &ViewAndRecords,
        n: usize,
    ) -> Result<&ViewRecord, crate::errors::BadInput> {
        view_records
            .records
            .get(n)
            .ok_or(crate::errors::BadInput("a view past the 52 records"))
    }
}

/// Which camera views hide the faces the ground's marks hide, by camera view
/// (`race::frame_draw`): 0 for the views near the car, 1 for the others.
pub const VIEW_HIDES: [u32; 52] = [
    0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
];

/// The size scale `view_scale` takes at a view's start (`race::frame_draw`).
pub const VIEW_SCALE_FIRST: ConstFloat = ConstFloat::new(256.0);

impl crate::race::plain_slot::Merge for ViewVars {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        crate::memory::plain_record::PlainRecord::merge_changes(self, now, start);
    }
}

/// A view's copy of the view variables (`race::view_scratch::ViewScratch::vars`): the same
/// fields, which only the view's drawing part reaches.
pub type ViewVarsCopy = ViewVars;

impl ViewVars {
    /// The view variables' starting values: the camera's bias figures and the first view kind.
    pub const fn new() -> Self {
        Self {
            view_car_flag_a: 0.0,
            view_car_stage: 0,
            view_car_flag_b: 0,
            drawn_car: 0,
            camera_height_bias: 0,
            camera_bias_scale: 8.0,
            camera_bias_limit: 64.0,
            view_scale: 384.0,
            view_place: [0; 3],
            view_place_f: [0.0; 3],
            view_x: 0,
            view_z: 0,
            view_angles: [0; 6],
            camera_zoom_hold: 128.0,
            camera_vector: [0.0; 3],
            camera_distance: 0.0,
            view_kind: 1,
        }
    }
}

/// The 52 view records at the start, each as its 23 dwords (floats as their bits).
const FIRST_RECORDS: [[u32; 23]; 52] = [
    [
        0x1, 0x0, 0x0, 0x0, 0x0, 0x3D4CCCCD, 0x3FE00000, 0x0, 0x0, 0x0, 0x3D23D70A, 0x3CA3D70A,
        0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x0, 0x3F35C28F, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A, 0x3CA3D70A,
        0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x3EC00000, 0x3ED9999A, 0x3F400000, 0x0, 0x0, 0x0, 0x3E23D70A,
        0x3E23D70A, 0x3E23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x3EC00000, 0x3ED9999A, 0x3F400000, 0x0, 0x0, 0x0, 0x3E23D70A,
        0x3E23D70A, 0x3E23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x400FAE14, 0xC0EC0000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x404F3B64, 0xC1100000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0,
    ],
    [
        0x2, 0x0, 0x0, 0x80000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x2, 0x0, 0x0, 0x80000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0,
    ],
    [
        0x2, 0x0, 0x0, 0x80000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x2, 0x0, 0x0, 0x80000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x0, 0xC0C00000, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x0, 0x3D23D70A, 0x3D23D70A,
        0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x0, 0xC1400000, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x180,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x0, 0xC1900000, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x180,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x0, 0xC1F00000, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x180,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x0, 0xC2480000, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x180,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0xBE4CCCCD, 0xC0500000, 0x0, 0x0, 0x0, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0xBF0F5C29, 0xC0500000, 0x0, 0x0, 0x0, 0x3D872B02, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0xBE4CCCCD, 0xC0733333, 0x0, 0x0, 0x0, 0x0, 0x3E800000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0xC0733333, 0x0, 0x0, 0x0, 0x3D872B02, 0x3E800000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0xBE4CCCCD, 0xC0733333, 0x0, 0x0, 0x0, 0x0, 0x3F400000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0xC0733333, 0x0, 0x0, 0x0, 0x3D872B02, 0x3F400000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3EC00000, 0xC0900000, 0x0, 0x0, 0x0, 0x3D23D70A, 0x3EC00000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3EC00000, 0xC0900000, 0x0, 0x0, 0x0, 0x3D23D70A, 0x3F200000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3EC00000, 0xC0900000, 0x0, 0x0, 0x0, 0x3D23D70A, 0x3E000000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3EC00000, 0xC0900000, 0x0, 0x0, 0x0, 0x3D23D70A, 0x3F600000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0xC0000000, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD, 0x3E4CCCCD,
        0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x0, 0x3F666666, 0xBF000000, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0xBFA00000, 0xBEB33333, 0x3F19999A, 0x0, 0x0, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x3FA00000, 0xBEB33333, 0x3F19999A, 0x0, 0x0, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0xBFA00000, 0xBEB33333, 0xBF000000, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x3FA00000, 0xBEB33333, 0xBF000000, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x0, 0x3ECCCCCD, 0x3FE66666, 0x0, 0x3F000000, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0xBFCCCCCD, 0x3F000000, 0x3ECCCCCD, 0x0, 0x3E947AE1, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x0, 0x0, 0x3FCCCCCD, 0x3F000000, 0x3ECCCCCD, 0x0, 0x3F35C28F, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3E4CCCCD,
        0x3E4CCCCD, 0x3E4CCCCD, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x0, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0xC8, 0x3F000000, 0x0, 0x3F000000, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x3F000000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0xC8, 0x3F000000, 0x0, 0x3F000000, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0xC350, 0x0, 0x42C80000, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x1, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x3F000000, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0xC350, 0x0, 0x42C80000, 0x0, 0x0, 0x0, 0x0, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x1, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFFFFFFFF, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0xFFFFFFFF, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x1, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFFFFFFFF, 0x0,
    ],
    [
        0x2, 0x0, 0x3FA0624E, 0xC0B80000, 0x0, 0x0, 0x0, 0x3C800000, 0x0, 0x0, 0x3D23D70A,
        0x3CA3D70A, 0x3D23D70A, 0x0, 0x180, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0xFFFFFFFF, 0x0,
    ],
    [
        0x3, 0x0, 0x0, 0xC0600000, 0x0, 0x0, 0x0, 0x0, 0x3F000000, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0xC8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1,
    ],
    [
        0x3, 0x0, 0x0, 0xC0600000, 0x0, 0x0, 0x0, 0x0, 0x3F000000, 0x0, 0x3F800000, 0x3F800000,
        0x3F800000, 0x0, 0x180, 0xC8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFFFFFFFF,
    ],
];

impl ViewRecord {
    /// A record from its 23 dwords in field order.
    const fn from_words(w: [u32; 23]) -> Self {
        ViewRecord {
            kind: w[0],
            offset: [w[1], w[2], w[3]],
            focus: [w[4], w[5], w[6]],
            angles: [w[7], w[8], w[9]],
            speeds: [w[10], w[11], w[12]],
            reserved_34: w[13],
            fish: w[14],
            change_time: w[15],
            turn_rates: [w[16], w[17], w[18]],
            halve_tilt: w[19],
            fixed_zoom: w[20],
            height_bias: w[21],
            blend: w[22],
        }
    }

    /// The 52 view records at the start.
    pub const fn first_records() -> [ViewRecord; 52] {
        let mut records = [const { ViewRecord::from_words([0; 23]) }; 52];
        let mut k = 0;
        while k < 52 {
            records[k] = ViewRecord::from_words(FIRST_RECORDS[k]);
            k += 1;
        }
        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::session::{PlainDword, PlainFloat};

    #[test]
    fn a_view_record_is_found_by_its_place_and_one_past_the_52_is_refused() {
        let places = &*crate::test_support::own::race()
            .frame
            .view_and_records
            .borrow();
        for n in 0..52usize {
            assert!(core::ptr::eq(
                ViewVars::record_of(places, n).unwrap(),
                ViewVars::record(places, n)
            ));
        }
        assert!(ViewVars::record_of(places, 52).is_err());
    }

    /// Every call a walk makes, as text.
    #[derive(Default)]
    struct Trace(Vec<String>);

    impl crate::memory::cells::CellVisitor for Trace {
        fn enter(&mut self, name: &'static str) {
            self.0.push(format!("enter {name}"));
        }
        fn index(&mut self, i: usize) {
            self.0.push(format!("index {i}"));
        }
        fn leave(&mut self) {
            self.0.push("leave".into());
        }
        fn leaf(&mut self, bytes: &[u8]) {
            self.0.push(format!("leaf {bytes:?}"));
        }
    }

    /// The view variables as they were held in cells.
    #[derive(rc2k_derive::Cells)]
    struct VarsInCells {
        view_car_flag_a: PlainFloat,
        view_car_stage: PlainDword,
        view_car_flag_b: PlainDword,
        drawn_car: PlainDword,
        camera_height_bias: PlainDword,
        camera_bias_scale: PlainFloat,
        camera_bias_limit: PlainFloat,
        view_scale: PlainFloat,
        view_place: [PlainDword; 3],
        view_place_f: [PlainFloat; 3],
        view_x: PlainDword,
        view_z: PlainDword,
        view_angles: [PlainDword; 6],
        camera_zoom_hold: PlainFloat,
        camera_vector: [PlainFloat; 3],
        camera_distance: PlainFloat,
        view_kind: PlainDword,
    }

    /// A view record as it was held in cells.
    #[derive(rc2k_derive::Cells)]
    struct RecordInCells {
        kind: PlainDword,
        offset: [PlainDword; 3],
        focus: [PlainDword; 3],
        angles: [PlainDword; 3],
        speeds: [PlainDword; 3],
        reserved_34: PlainDword,
        fish: PlainDword,
        change_time: PlainDword,
        turn_rates: [PlainDword; 3],
        halve_tilt: PlainDword,
        fixed_zoom: PlainDword,
        height_bias: PlainDword,
        blend: PlainDword,
    }

    /// The group as it was held in cells.
    #[derive(rc2k_derive::Cells)]
    struct GroupInCells {
        view_vars: VarsInCells,
        records: [RecordInCells; 52],
    }

    /// The plain group walks as the cells of its shape did: the same paths and bytes, which is
    /// what keeps the walks the tests and the simulation hash take.
    #[test]
    fn the_plain_group_walks_as_the_cells_of_its_shape() {
        use crate::memory::cells::Cells;
        let plain = ViewAndRecords::new();
        let cells = GroupInCells {
            view_vars: VarsInCells {
                view_car_flag_a: PlainFloat::new(plain.view_vars.view_car_flag_a),
                view_car_stage: PlainDword::new(plain.view_vars.view_car_stage),
                view_car_flag_b: PlainDword::new(plain.view_vars.view_car_flag_b),
                drawn_car: PlainDword::new(plain.view_vars.drawn_car),
                camera_height_bias: PlainDword::new(plain.view_vars.camera_height_bias),
                camera_bias_scale: PlainFloat::new(plain.view_vars.camera_bias_scale),
                camera_bias_limit: PlainFloat::new(plain.view_vars.camera_bias_limit),
                view_scale: PlainFloat::new(plain.view_vars.view_scale),
                view_place: PlainDword::array(plain.view_vars.view_place),
                view_place_f: PlainFloat::array(plain.view_vars.view_place_f),
                view_x: PlainDword::new(plain.view_vars.view_x),
                view_z: PlainDword::new(plain.view_vars.view_z),
                view_angles: PlainDword::array(plain.view_vars.view_angles),
                camera_zoom_hold: PlainFloat::new(plain.view_vars.camera_zoom_hold),
                camera_vector: PlainFloat::array(plain.view_vars.camera_vector),
                camera_distance: PlainFloat::new(plain.view_vars.camera_distance),
                view_kind: PlainDword::new(plain.view_vars.view_kind),
            },
            records: std::array::from_fn(|n| {
                let r = &plain.records[n];
                RecordInCells {
                    kind: PlainDword::new(r.kind),
                    offset: PlainDword::array(r.offset),
                    focus: PlainDword::array(r.focus),
                    angles: PlainDword::array(r.angles),
                    speeds: PlainDword::array(r.speeds),
                    reserved_34: PlainDword::new(r.reserved_34),
                    fish: PlainDword::new(r.fish),
                    change_time: PlainDword::new(r.change_time),
                    turn_rates: PlainDword::array(r.turn_rates),
                    halve_tilt: PlainDword::new(r.halve_tilt),
                    fixed_zoom: PlainDword::new(r.fixed_zoom),
                    height_bias: PlainDword::new(r.height_bias),
                    blend: PlainDword::new(r.blend),
                }
            }),
        };
        let (mut from_cells, mut from_plain) = (Trace::default(), Trace::default());
        cells.visit(&mut from_cells);
        crate::memory::plain_record::PlainRecord::visit(&plain, &mut from_plain);
        assert_eq!(from_plain.0, from_cells.0);
    }

    #[test]
    fn every_camera_view_has_its_own_hide_entry() {
        let hides = VIEW_HIDES;
        // Near the car (views 0 .. 6) the ground's marks hide faces; the other views show them.
        assert!(hides[..7].iter().all(|&shows| shows == 0));
        assert!(hides[7..0x23].iter().all(|&shows| shows == 1));
        // The replay's views: the cameras behind the car like views 4 .. 6, the fixed
        // cameras like views 7 .. 10.
        assert!(hides[0x23..0x2A].iter().all(|&shows| shows == 0));
        assert!(hides[0x2A..0x32].iter().all(|&shows| shows == 1));
        assert!(hides[0x32..].iter().all(|&shows| shows == 0));
    }
}
