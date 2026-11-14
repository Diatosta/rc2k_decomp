//! The view and camera variables of the race block: the drawn car, face group mask, fade alpha,
//! view place and angles (int and float), the 52 view-record names (as handles) and the per-view
//! hide table; then the 52 view records (`ViewRecord`). Gaps are unknown bytes. The record fields'
//! short names are the ones on the kstar camera page: `xoff`, `xfoc`, `xang`, `mspx`, `fish`.

use crate::cells::ZeroCells;
use crate::race::groups_store::ViewAndRecords;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::{CellKind, PlainCells, PlainDword, PlainFloat};

/// The view variables, on the cells of kind `K`: plain for the shared ones
/// (`ViewAndRecords::view_vars`, which only the main thread writes) and for a view's copy
/// (`ViewVarsCopy`), which only the view's drawing part reaches.
#[derive(rc2k_derive::Cells)]
pub struct ViewVars<K: CellKind = PlainCells> {
    /// +0x0: the car block's `camera_switch_share` copied at the frame start (`race::frame_draw`);
    /// `race::car_draw` tests it against the drawn car, so it marks which car the view follows.
    pub view_car_flag_a: K::Float,
    /// +0x4: the car block's `camera_view` copied at the frame start; the car's index into the
    /// stage records (`race::car_draw`, `race::frame_draw`).
    pub view_car_stage: K::Dword,
    /// +0x8: the car block's `camera_mode_is_2_or_3` copied at the frame start
    /// (`race::frame_draw`); read by `camera_view` and written by `race::car_light_modes`.
    pub view_car_flag_b: K::Dword,
    /// +0xc: copied to `view_scale` by `race::frame_draw` (the view's size scale).
    pub view_scale_kept: K::Float,
    /// +0x10: the car being drawn (`race::car_draw`, `scene::set_object_light`,
    /// `scene::list_object_face`).
    pub drawn_car: K::Dword,
    /// +0x14: face group mask for the drawn car; the drawing's is the view's
    /// (`race::car_draw_state`), this one not used.
    pub group_mask: K::Dword,
    /// +0x18: the car's fade alpha, now the view's (`race::face_work`); no longer reached.
    pub fade_alpha_moved: K::Dword,
    /// +0x1c: camera height bias (`physics::camera_height_bias`).
    pub camera_height_bias: K::Dword,
    /// +0x20: the camera height bias's scale (`physics::camera_height_bias`).
    pub camera_bias_scale: K::Float,
    /// +0x24: held to 0 ..= 1 (`physics::camera_height_bias`).
    pub camera_bias_limit: K::Float,
    /// +0x28: a float, the far/scale (`scene::set_view_window`, `race::draw_mirror_view`,
    /// `race::race_end`).
    pub view_scale: K::Float,
    /// +0x2c: `view_place`: the view place x, y, z (`scene::view_matrix`, `race::frame_draw`,
    /// `drive::set_listener`). The three are written as a run.
    pub view_place: [K::Dword; 3],
    /// +0x38: the place as floats (`race::course_faces`, `race::frame_draw`).
    pub view_place_f: [K::Float; 3],
    /// +0x44: view x (`race::frame_draw`).
    pub view_x: K::Dword,
    /// +0x48: view z (`race::frame_draw`).
    pub view_z: K::Dword,
    /// +0x4c: `view_angles`: y, x, z then a second triple (`scene::view_matrix`,
    /// `race::frame_draw`).
    pub view_angles: [K::Dword; 6],
    /// +0x64: the camera's zoom hold (`physics::update_camera`).
    pub camera_zoom_hold: K::Float,
    /// +0x68: unused; it only keeps the layout.
    pub unused_68: K::Dword,
    /// +0x6c: the camera vector (three floats) whose length `physics::update_camera` takes.
    pub camera_vector: [K::Float; 3],
    /// +0x78: unused.
    pub unknown_78: [K::Dword; 14],
    /// +0xb0: held to 0 ..= 1 (`physics::camera_height_bias`, `physics::update_camera`).
    pub camera_distance: K::Float,
    /// +0xb4: 0x34 pointers to view records (`physics::camera_view`, `drive::pick_replay_camera`,
    /// `effects::stage_record`, `race::car_draw`).
    pub view_records: [K::Dword; 52],
    /// +0x184: the view kind, the camera mode number (`drive::set_camera_view`;
    /// `race::frame_draw::scene_objects` and `car_draw` branch on it).
    pub view_kind: K::Dword,
    /// +0x188: unused; it only keeps the layout.
    pub unused_188: [K::Dword; 33],
    /// +0x20c: the table of which views hide what, by camera view (`race::frame_draw`).
    pub view_hides: [K::Dword; 34],
}

/// A view record (0x5C bytes; 52 of them, named by `ViewAndRecords.view_vars.view_records[0]`): one
/// camera mode's figures. The three-letter names are the developers' (the kstar camera page).
/// Records 1 .. 3 are refilled from the car's light record (`physics::update_camera`), 4 .. 6
/// placed by `physics::place_camera`, 6 by the finish camera too.
#[derive(rc2k_derive::Cells)]
pub struct ViewRecord {
    /// +0x0: the mode's kind (what a replay camera wish stops at: `drive::pick_replay_camera`).
    pub kind: PlainDword,
    /// +0x4: `xoff`, `yoff`, `zoff`: the camera's offset (floats; `take_first_view` /
    /// `take_second_view` add the camera frame to it).
    pub offset: [PlainDword; 3],
    /// +0x10: `xfoc`, `yfoc`, `zfoc`: the focus (floats). +0x10 .. +0x38 are the eleven figures the
    /// views take whole (`take_first_view` / `take_second_view`).
    pub focus: [PlainDword; 3],
    /// +0x1C: `xang`, `yang`, `zang`: the angles.
    pub angles: [PlainDword; 3],
    /// +0x28: `mspx`, `mspy`, `mspz` (floats; the views' shares).
    pub speeds: [PlainDword; 3],
    /// +0x34: the record's dword before `fish`; copied with the rest of the record and never
    /// used otherwise.
    pub unknown_34: PlainDword,
    /// +0x38: `fish` (a whole number).
    pub fish: PlainDword,
    /// +0x3C: how long a change into this mode takes (ticks; `physics::camera_view::share_done`).
    pub change_time: PlainDword,
    /// +0x40: the three turn figures eased in over a change (floats; `ease_view_angles`).
    pub turn_rates: [PlainDword; 3],
    /// +0x4C: nonzero: the car's pitch and roll halved and the camera kept off the ground
    /// (`take_first_view`, `camera_avoid_ground`).
    pub halve_tilt: PlainDword,
    /// +0x50: nonzero: the zoom held at 0x180 (`physics::update_camera`).
    pub fixed_zoom: PlainDword,
    /// +0x54: the height bias's figure (`camera_height_bias`; sign read).
    pub height_bias: PlainDword,
    /// +0x58: the blend's figure (`ease_view_blend`; sign read).
    pub blend: PlainDword,
}

impl ViewRecord {
    /// The record's dword `n` (0 .. 23) in its order: what a read running on into the record by
    /// dword sees (`race::frame_draw::course_view::face_hidden` for camera views past the view
    /// hides table).
    pub fn dword(&self, n: usize) -> Option<u32> {
        let [o, f, a, s, r] = [
            &self.offset,
            &self.focus,
            &self.angles,
            &self.speeds,
            &self.turn_rates,
        ]
        .map(|t| t.each_ref().map(PlainDword::get));
        let words: [u32; 23] = [
            self.kind.get(),
            o[0],
            o[1],
            o[2],
            f[0],
            f[1],
            f[2],
            a[0],
            a[1],
            a[2],
            s[0],
            s[1],
            s[2],
            self.unknown_34.get(),
            self.fish.get(),
            self.change_time.get(),
            r[0],
            r[1],
            r[2],
            self.halve_tilt.get(),
            self.fixed_zoom.get(),
            self.height_bias.get(),
            self.blend.get(),
        ];
        words.get(n).copied()
    }

    /// The offset's three dwords (+0x4 .. +0xC).
    pub fn offset_bits(&self) -> [u32; 3] {
        self.offset.each_ref().map(PlainDword::get)
    }

    /// The eleven dwords the views take whole (+0x10 .. +0x38): focus, angles, speeds, +0x34, fish.
    pub fn taken(&self) -> [u32; 11] {
        let [f, a, s] =
            [&self.focus, &self.angles, &self.speeds].map(|t| t.each_ref().map(PlainDword::get));
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
            self.unknown_34.get(),
            self.fish.get(),
        ]
    }
}

impl ViewVars {
    /// View record `n` by its place among the 52 (records 1 .. 3 are refilled from the car, 4 .. 6
    /// placed).
    pub fn record(view_records: &ViewAndRecords, n: usize) -> &ViewRecord {
        &view_records.records[n]
    }

    /// View record `n`. A view past the 52, or a value that is no handle, is refused.
    pub fn record_of(
        view_records: &ViewAndRecords,
        n: usize,
    ) -> Result<&ViewRecord, crate::errors::BadInput> {
        let cell = view_records
            .view_vars
            .view_records
            .get(n)
            .ok_or(crate::errors::BadInput("a view past the 52 records"))?;
        let k = cell
            .get()
            .checked_sub(VIEW_TAG)
            .ok_or(crate::errors::BadInput(
                "a view record's name that is no handle",
            ))?;
        view_records
            .records
            .get(k as usize)
            .ok_or(crate::errors::BadInput(
                "a view record's name off the records",
            ))
    }
}

/// A view's copy of the view variables (`race::view_scratch::ViewScratch::vars`): the same
/// fields, which only the view's drawing part reaches.
pub type ViewVarsCopy = ViewVars;

/// The view variables at the start, on the cells `$dword` and `$float`: the view size
/// scales, the camera's bias figures, the 52 names of the view records (as handles), the first view
/// kind and the table of which views hide what.
macro_rules! first_view_vars {
    ($dword:ident, $float:ident) => {
        Self {
            view_car_flag_a: ZeroCells::ZERO,
            view_car_stage: ZeroCells::ZERO,
            view_car_flag_b: ZeroCells::ZERO,
            view_scale_kept: $float::new(256.0),
            drawn_car: ZeroCells::ZERO,
            group_mask: ZeroCells::ZERO,
            fade_alpha_moved: ZeroCells::ZERO,
            camera_height_bias: ZeroCells::ZERO,
            camera_bias_scale: $float::new(8.0),
            camera_bias_limit: $float::new(64.0),
            view_scale: $float::new(384.0),
            view_place: ZeroCells::ZERO,
            view_place_f: ZeroCells::ZERO,
            view_x: ZeroCells::ZERO,
            view_z: ZeroCells::ZERO,
            view_angles: ZeroCells::ZERO,
            camera_zoom_hold: $float::new(128.0),
            unused_68: $dword::new(0x40A00000),
            camera_vector: ZeroCells::ZERO,
            unknown_78: ZeroCells::ZERO,
            camera_distance: ZeroCells::ZERO,
            view_records: {
                let mut a: [$dword; 52] = ZeroCells::ZERO;
                let mut k = 0;
                while k < 52 {
                    a[k] = $dword::new(VIEW_TAG | k as u32);
                    k += 1;
                }
                a
            },
            view_kind: $dword::new(1),
            unused_188: [
                $dword::new(1),
                $dword::new(2),
                $dword::new(2),
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(3),
                $dword::new(4),
                $dword::new(5),
                $dword::new(6),
                $dword::new(0xA),
                $dword::new(0xB),
                $dword::new(0xC),
                $dword::new(0xD),
                $dword::new(0xE),
                $dword::new(0x10),
                $dword::new(0x11),
                $dword::new(0x12),
                $dword::new(0x13),
                $dword::new(0x14),
                $dword::new(0x15),
                $dword::new(0x16),
                $dword::new(0x17),
                $dword::new(0x18),
                $dword::new(0x19),
                $dword::new(0x1A),
                $dword::new(0x1B),
                $dword::new(0x1C),
                $dword::new(0x1D),
                $dword::new(0x1E),
                $dword::new(0x1F),
                $dword::new(0x20),
                $dword::new(0x21),
            ],
            view_hides: [
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(0),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
                $dword::new(1),
            ],
        }
    };
}

impl ViewVars<PlainCells> {
    /// The view variables' starting values.
    pub const fn new() -> Self {
        first_view_vars!(PlainDword, PlainFloat)
    }
}

/// A view record's handle in `view_records` is `VIEW_TAG | k` for the record `k` of the 52.
pub const VIEW_TAG: u32 = 0xC300_0000;

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
            kind: PlainDword::new(w[0]),
            offset: [
                PlainDword::new(w[1]),
                PlainDword::new(w[2]),
                PlainDword::new(w[3]),
            ],
            focus: [
                PlainDword::new(w[4]),
                PlainDword::new(w[5]),
                PlainDword::new(w[6]),
            ],
            angles: [
                PlainDword::new(w[7]),
                PlainDword::new(w[8]),
                PlainDword::new(w[9]),
            ],
            speeds: [
                PlainDword::new(w[10]),
                PlainDword::new(w[11]),
                PlainDword::new(w[12]),
            ],
            unknown_34: PlainDword::new(w[13]),
            fish: PlainDword::new(w[14]),
            change_time: PlainDword::new(w[15]),
            turn_rates: [
                PlainDword::new(w[16]),
                PlainDword::new(w[17]),
                PlainDword::new(w[18]),
            ],
            halve_tilt: PlainDword::new(w[19]),
            fixed_zoom: PlainDword::new(w[20]),
            height_bias: PlainDword::new(w[21]),
            blend: PlainDword::new(w[22]),
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

    #[test]
    fn the_view_table_names_its_records() {
        for n in 0..52usize {
            assert_eq!(
                race().frame.view_and_records.get().view_vars.view_records[n].get(),
                VIEW_TAG | n as u32
            );
            assert!(core::ptr::eq(
                ViewVars::record_of(race().frame.view_and_records.get(), n).unwrap(),
                ViewVars::record(race().frame.view_and_records.get(), n)
            ));
        }
        assert!(ViewVars::record_of(race().frame.view_and_records.get(), 52).is_err());
    }

    #[test]
    fn a_record_s_dwords_run_in_its_order() {
        let places = race().frame.view_and_records.get();
        let record = &places.records[0];
        assert_eq!(record.dword(0), Some(record.kind.get()));
        assert_eq!(record.dword(1), Some(record.offset[0].get()));
        assert_eq!(record.dword(14), Some(record.fish.get()));
        assert_eq!(record.dword(15), Some(record.change_time.get()));
        assert_eq!(record.dword(22), Some(record.blend.get()));
        assert_eq!(record.dword(23), None);
        // The camera views past the view hides table (34 .. 51) all land in the first record.
        let past = 52 - places.view_vars.view_hides.len();
        assert!(record.dword(past - 1).is_some());
    }
}
