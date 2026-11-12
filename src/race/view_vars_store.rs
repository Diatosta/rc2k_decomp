//! The view/camera variables of the race block at 0x60626C .. 0x606500: the drawn car, face group mask,
//! fade alpha, view place and angles (int and float), the 52 view-record names at 0x606320 and the
//! per-view hide table; then the 52 view records (`ViewRecord`, 0x606500 .. 0x6077B0). Gaps are
//! unknown bytes. Names are readings unless IDA's; the records' are the developers' own (the
//! kstar camera page's labels: `xoff`, `xfoc`, `xang`, `mspx`, `fish`).

use crate::race::groups_store::ViewAndRecords;
#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewVars {
    /// +0x0 (0x60626C): the car block's +0x69C copied at the frame start (`race::frame_draw`); `race::car_draw` tests it against the drawn car, so it marks which car the view follows.
    pub view_car_flag_a: Dword,
    /// +0x4 (0x606270): the car block's +0x684 copied at the frame start; the car's index into the stage records (`race::car_draw`, `race::frame_draw`).
    pub view_car_stage: Dword,
    /// +0x8 (0x606274): the car block's +0x6F0 copied at the frame start (`race::frame_draw`); IDA: read by `camera_view` (0x43832A) and written by `race::car_light_modes` (0x455C44).
    pub view_car_flag_b: Dword,
    /// +0xc (0x606278): `dword_606278`: copied to `view_scale` by `race::frame_draw` (the view's size scale).
    pub view_scale_kept: Float,
    /// +0x10 (0x60627C): `g_drawnCar`: the car being drawn (`race::car_draw`, `scene::set_object_light`, `scene::list_object_face`).
    pub drawn_car: Dword,
    /// +0x14 (0x606280): `g_groupMask`: face group mask for the drawn car; the drawing's is the
    /// view's (`race::car_draw_state`), this one not used.
    pub group_mask: Dword,
    /// +0x18 (0x606284): `dword_606284`: the car's fade alpha, now the view's (`race::face_work`):
    /// this place is no longer reached.
    pub fade_alpha_moved: Dword,
    /// +0x1c (0x606288): `dword_606288`: camera height bias (`physics::camera_height_bias`).
    pub camera_height_bias: Dword,
    /// +0x20 (0x60628C): `flt_60628C` (`physics::camera_height_bias`).
    pub camera_bias_scale: Float,
    /// +0x24 (0x606290): `flt_606290`: held to 0 ..= 1 (`physics::camera_height_bias`).
    pub camera_bias_limit: Float,
    /// +0x28 (0x606294): `dword_606294`: a float, the far/scale (`scene::set_view_window`, `race::draw_mirror_view`, `race::race_end`).
    pub view_scale: Float,
    /// +0x2c (0x606298): `g_viewPlace`, `dword_60629C`, `dword_6062A0`: the view place x, y, z (`scene::view_matrix`, `race::frame_draw`, `drive::set_listener`). The original writes the run at once.
    pub view_place: [Dword; 3],
    /// +0x38 (0x6062A4): `flt_6062A4` ..: the place as floats (`race::course_faces`, `race::frame_draw`).
    pub view_place_f: [Float; 3],
    /// +0x44 (0x6062B0): `dword_6062B0`: view x (`race::frame_draw`).
    pub view_x: Dword,
    /// +0x48 (0x6062B4): `dword_6062B4`: view z (`race::frame_draw`).
    pub view_z: Dword,
    /// +0x4c (0x6062B8): `g_viewAngles` y, x, z then a second triple at `dword_6062C4` ..; (`scene::view_matrix`, `race::frame_draw`).
    pub view_angles: [Dword; 6],
    /// +0x64 (0x6062D0): `flt_6062D0` (`physics::update_camera`).
    pub camera_zoom_hold: Float,
    /// +0x68 (0x6062D4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_68: Dword,
    /// +0x6c (0x6062D8): `flt_6062D8` ..: the camera vector (three floats) whose length `physics::update_camera` takes.
    pub camera_vector: [Float; 3],
    /// +0x78 (0x6062E4): used only by the unported `sub_427DB4` and `sub_43B177` (0x6062E4: compared with 0 and cleared); the port never reaches it.
    pub unknown_78: [Dword; 14],
    /// +0xb0 (0x60631C): `flt_60631C`: held to 0 ..= 1 (`physics::camera_height_bias`, `physics::update_camera`).
    pub camera_distance: Float,
    /// +0xb4 (0x606320): `off_606320`: 0x34 pointers to view records (`physics::camera_view`, `drive::pick_replay_camera`, `effects::stage_record`, `race::car_draw`).
    pub view_records: [Dword; 52],
    /// +0x184 (0x6063F0): `dword_6063F0`: the view kind, the camera mode number (`drive::set_camera_view`; `race::frame_draw::scene_objects` and `car_draw` branch on it).
    pub view_kind: Dword,
    /// +0x188 (0x6063F4): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_188: [Dword; 33],
    /// +0x20c (0x606478): `dword_606478[]`: by camera view (`race::frame_draw`).
    pub view_hides: [Dword; 34],
}

/// A view record (0x5C bytes; 52 of them at 0x606500, named by `ViewAndRecords.view_vars.view_records[0]`): one camera
/// mode's figures. The three-letter names are the developers' (the kstar camera page).
/// Records 1 .. 3 are refilled from the car's light record (`physics::update_camera`),
/// 4 .. 6 placed by `physics::place_camera`, 6 by the finish camera too.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewRecord {
    /// +0x0: the mode's kind (what a replay camera wish stops at: `drive::pick_replay_camera`).
    pub kind: Dword,
    /// +0x4: `xoff`, `yoff`, `zoff`: the camera's offset (floats; `take_first_view` / `take_second_view` add the
    /// camera frame to it).
    pub offset: [Dword; 3],
    /// +0x10: `xfoc`, `yfoc`, `zfoc`: the focus (floats). +0x10 .. +0x38 are the eleven
    /// figures the views take whole (`take_first_view` / `take_second_view`).
    pub focus: [Dword; 3],
    /// +0x1C: `xang`, `yang`, `zang`: the angles.
    pub angles: [Dword; 3],
    /// +0x28: `mspx`, `mspy`, `mspz` (floats; the views' shares).
    pub speeds: [Dword; 3],
    /// +0x34: the camera mode record's dword before `fish`; copied with the rest of the record and named nowhere else (the code shows no use).
    pub unknown_34: Dword,
    /// +0x38: `fish` (a whole number).
    pub fish: Dword,
    /// +0x3C: how long a change into this mode takes (ticks; `physics::camera_view::share_done`).
    pub change_time: Dword,
    /// +0x40: the three turn figures eased in over a change (floats; `ease_view_angles`).
    pub turn_rates: [Dword; 3],
    /// +0x4C: nonzero: the car's pitch and roll halved and the camera kept off the ground
    /// (`take_first_view`, `camera_avoid_ground`).
    pub halve_tilt: Dword,
    /// +0x50: nonzero: the zoom held at 0x180 (`physics::update_camera`).
    pub fixed_zoom: Dword,
    /// +0x54: the height bias's figure (`camera_height_bias`; sign read).
    pub height_bias: Dword,
    /// +0x58: the blend's figure (`ease_view_blend`; sign read).
    pub blend: Dword,
}

const _: () = assert!(core::mem::size_of::<ViewRecord>() == ViewVars::RECORD_SIZE);

impl ViewRecord {
    /// The offset's three dwords (+0x4 .. +0xC).
    pub fn offset_bits(&self) -> [u32; 3] {
        self.offset.each_ref().map(Dword::get)
    }

    /// The eleven dwords the views take whole (+0x10 .. +0x38): focus, angles, speeds,
    /// +0x34, fish.
    pub fn taken(&self) -> [u32; 11] {
        let [f, a, s] =
            [&self.focus, &self.angles, &self.speeds].map(|t| t.each_ref().map(Dword::get));
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
    /// IDA's addresses of the first view record and the end of the last (52 of 0x5C).
    const RECORDS: usize = 0x606500;
    const RECORDS_END: usize = 0x6077B0;

    /// View record `n` by its place among the 52 (the records the code names by IDA
    /// address: 1 .. 3 refilled from the car, 4 .. 6 placed).
    pub fn record(view_records: &ViewAndRecords, n: usize) -> &ViewRecord {
        &view_records.records[n]
    }

    /// The bytes of a view record.
    pub const RECORD_SIZE: usize = 0x5C;

    /// View record `n` (phase 13: the table holds handles, `VIEW_TAG | k` for the k-th
    /// record). A view past the 52, or a value that is no handle, is refused.
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

/// A view record by handle (phase 13): `VIEW_TAG | k` for the record `k` of the 52, in
/// place of the record's address in `ViewAndRecords.view_vars.view_records[0]` (0xC3 of the tags 0xC3 .. 0xC6 given).
pub const VIEW_TAG: u32 = 0xC300_0000;

/// The handle for IDA's `ida` (a view record's address; anything else fails to compile).
pub const fn view_ref(ida: u32) -> u32 {
    let ida = ida as usize;
    assert!(ida >= ViewVars::RECORDS && ida < ViewVars::RECORDS_END);
    let off = ida - ViewVars::RECORDS;
    assert!(off.is_multiple_of(ViewVars::RECORD_SIZE));
    VIEW_TAG | (off / ViewVars::RECORD_SIZE) as u32
}

/// `bytes` (the group's first values from IDA's `start`) with the 52 dwords of `ViewAndRecords.view_vars.view_records[0]`
/// made `view_ref`s.
pub const fn with_view_refs<const N: usize>(mut bytes: [u8; N], start: usize) -> [u8; N] {
    let mut k = 0;
    while k < 52 {
        let o = 0x606320 + 4 * k - start;
        let v = u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        let h = view_ref(v).to_le_bytes();
        bytes[o] = h[0];
        bytes[o + 1] = h[1];
        bytes[o + 2] = h[2];
        bytes[o + 3] = h[3];
        k += 1;
    }
    bytes
}

/// IDA 0x60626C, the first byte of the piece.
pub const START: usize = 0x60626C;
/// IDA 0x606500, the end of the piece (the view records follow).
pub const END: usize = 0x606500;

const _: () = assert!(END - START == core::mem::size_of::<ViewVars>());

/// Where IDA 0x60626C is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &race().frame.view_and_records.view_vars,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn the_view_table_names_its_records_as_the_image_does() {
        for n in 0..52usize {
            let image = crate::test_support::first_values::dword(
                "race.owner.RACE.frame.view_and_records.view_vars.view_records[0]/208",
                n,
            );
            // A handle in place of the image's address, naming the same record.
            assert_eq!(
                race().frame.view_and_records.view_vars.view_records[n].get(),
                view_ref(image)
            );
            assert_eq!(image as usize, ViewVars::RECORDS + 0x5C * n);
            assert!(core::ptr::eq(
                ViewVars::record_of(&race().frame.view_and_records, n).unwrap(),
                ViewVars::record(&race().frame.view_and_records, n)
            ));
        }
        assert!(ViewVars::record_of(&race().frame.view_and_records, 52).is_err());
    }

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(offset_of!(ViewVars, view_car_flag_a), 0x0);
        assert_eq!(offset_of!(ViewVars, fade_alpha_moved), 0x18);
        assert_eq!(offset_of!(ViewVars, view_place_f), 0x38);
        assert_eq!(offset_of!(ViewVars, camera_distance), 0xb0);
    }
}
