//! The course faces' work and the race's views (IDA 0x63D9B8 .. 0x63DC5C; phase 7, typed): the
//! face being drawn (corners, textures, orders, coordinates, colours and lights by corner,
//! `race::course_faces`), the polygon mode, the lamps, the views (which are on, their cars and
//! blocks, the drawn view), the race's cars and players, the frame clock, the depth and sort
//! counts. `#[repr(C)]` with the image's layout, over the race block's bytes (`RACE_VIEW`), as
//! `CAR`. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceView {
    /// +0x0 (0x63D9B8): `g_reflectionsKept` (`scene::cached_reflection_coordinates`); now each view's (`race::face_work::FaceWork`); unused.
    pub reflections_kept: PlainDword,
    /// +0x4 (0x63D9BC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_4: [PlainDword; 5],
    /// +0x18 (0x63D9D0): the face's surface kinds (`race::course_faces`, `physics::wheel_surfaces`, `race::best_line`).
    pub surface_kinds: [PlainDword; 4],
    /// +0x28 (0x63D9E0): `g_depthTest` (`race::course_faces`); now each view's (`FaceWork::depth_test`); unused.
    pub depth_test: PlainDword,
    /// +0x2c (0x63D9E4): the face's halves turned to the view (`race::course_faces`); now each view's (`race::face_work::FaceWork`); unused.
    pub face_turned: [PlainDword; 2],
    /// +0x34 (0x63D9EC): the face's corners ([3] -1 for faces of three points, `race::norms`); now each view's (`race::face_work::FaceWork`); here only the image's first values, which
    /// `race::norms` reads before any face was drawn (`face_work::last_fourth_corner`).
    pub corners: [PlainDword; 4],
    /// +0x44 (0x63D9FC): (`race::course_faces`).
    pub slope: PlainFloat,
    /// +0x48 (0x63DA00): (`race::course_faces`).
    pub near: PlainFloat,
    /// +0x4c (0x63DA04): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_4c: PlainDword,
    /// +0x50 (0x63DA08): the frame the face was drawn in (`race::face_materials` wrote it; nothing read it, so it is no longer written); unused.
    pub face_frame: PlainDword,
    /// +0x54 (0x63DA0C): the first texture's coordinates by corner, x then y; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_uv: [PlainDword; 8],
    /// +0x74 (0x63DA2C): the first texture (`race::course_faces`); now each view's (`race::face_work::FaceWork`); unused.
    pub texture_a: PlainDword,
    /// +0x78 (0x63DA30): its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_a: PlainDword,
    /// +0x7c (0x63DA34): its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_a: PlainDword,
    /// +0x80 (0x63DA38): the second texture (`scene::draw_polygon`'s material); now each view's (`race::face_work::FaceWork`); unused.
    pub texture_b: PlainDword,
    /// +0x84 (0x63DA3C): its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_b: PlainDword,
    /// +0x88 (0x63DA40): its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_b: PlainDword,
    /// +0x8c (0x63DA44): the colour by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_colours: [PlainDword; 4],
    /// +0x9c (0x63DA54): the light by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_lights: [PlainDword; 4],
    /// +0xac (0x63DA64): the see-through by corner (a byte); now each view's (`race::face_work::FaceWork`); unused.
    pub corner_alphas: [PlainDword; 4],
    /// +0xbc (0x63DA74): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_bc: [PlainDword; 2],
    /// +0xc4 (0x63DA7C): `g_turnedCornerOrder`: the turned orders, a byte table read in place (`race::course_faces`).
    pub turned_corner_order: [PlainDword; 14],
    /// +0xfc (0x63DAB4): `g_polygonMode`; now each view's (`race::face_work::FaceWork`); unused.
    pub polygon_mode: PlainDword,
    /// +0x100 (0x63DAB8): the second texture's coordinates by corner, x then y; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_uv_b: [PlainDword; 8],
    /// +0x120 (0x63DAD8): `g_cornerUvOffsets`: by order, four.
    pub corner_uv_offsets: [PlainDword; 32],
    /// +0x1a0 (0x63DB58): `g_lampsWanted`; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamps_wanted: PlainDword,
    /// +0x1a4 (0x63DB5C): (`drive::share_car_state`).
    pub level_from: PlainFloat,
    /// +0x1a8 (0x63DB60): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1a8: [PlainDword; 3],
    /// +0x1b4 (0x63DB6C): `g_viewLamps`: 0 from `race::draw_view_scene`, read by nothing; no longer written.
    pub view_lamps: PlainDword,
    /// +0x1b8 (0x63DB70): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1b8: PlainDword,
    /// +0x1bc (0x63DB74): each view on (`race::draw_race_scene`, `race::show_replay_still`).
    pub view_on: [PlainDword; 2],
    /// +0x1c4 (0x63DB7C): the two players' option bits (`drive::replay_cameras`, `race::settings_copy`).
    pub player_options: [PlainDword; 2],
    /// +0x1cc (0x63DB84): the race's ticks run while below it (`drive::race_tick`; `race::credits_race`).
    pub after_credits: PlainDword,
    /// +0x1d0 (0x63DB88): (`drive::choose_shown_cars`, `drive::replay_cameras`).
    pub view_switch: PlainDword,
    /// +0x1d4 (0x63DB8C): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1d4: PlainDword,
    /// +0x1d8 (0x63DB90): `g_raceCars`: the race's cars.
    pub race_cars: PlainDword,
    /// +0x1dc (0x63DB94): the players (`settings.race_mode`; `[shaunnumplayers]`).
    pub players: PlainDword,
    /// +0x1e0 (0x63DB98): the views shown (`race::draw_race_scene`'s one view; the second player's).
    pub views: PlainDword,
    /// +0x1e4 (0x63DB9C): `g_drawnView`.
    pub drawn_view: PlainDword,
    /// +0x1e8 (0x63DBA0): this machine's car (`net::lets_race`).
    pub local_car: PlainDword,
    /// +0x1ec (0x63DBA4): `g_shownCar`: the drawn view's car, read only by `race::car_light_modes`, which keeps it itself; no longer written.
    pub shown_car: PlainDword,
    /// +0x1f0 (0x63DBA8): each view's car.
    pub view_cars: [PlainDword; 2],
    /// +0x1f8 (0x63DBB0): `g_viewBlocks`: each view's car block (by handle:
    /// `blocks_store::block_ref`; read through `view_block`).
    pub view_blocks: [PlainDword; 2],
    /// +0x200 (0x63DBB8): cleared by `race::draw_race_scene`.
    pub view_cleared: [PlainDword; 32],
    /// +0x280 (0x63DC38): (`race::show_frame`).
    pub clock_base: PlainDword,
    /// +0x284 (0x63DC3C): (`race::show_frame`).
    pub frame_time: PlainDword,
    /// +0x288 (0x63DC40): `g_faceDepthKey`; now each view's (`race::face_work::FaceWork`); unused.
    pub face_depth_key: PlainDword,
    /// +0x28c (0x63DC44): (`race::frame_draw`); written only, by nothing now (a dead store); unused.
    pub face_side: PlainDword,
    /// +0x290 (0x63DC48): `g_lampCount`; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamp_count: PlainDword,
    /// +0x294 (0x63DC4C): `g_depthEntryCount`.
    pub depth_entry_count: PlainDword,
    /// +0x298 (0x63DC50): `g_sortEntryCount`.
    pub sort_entry_count: PlainDword,
    /// +0x29c (0x63DC54): how far is seen times 0x20, twice (`scene::set_view_window`); now each view's (`scene::view_window`); unused.
    pub view_depth: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceView {
    /// The piece, with the image's first values.
    pub const fn new() -> Self {
        Self {
            surface_kinds: [
                PlainDword::new(3),
                PlainDword::new(5),
                PlainDword::new(4),
                PlainDword::new(0),
            ],
            slope: PlainFloat::new(1.2),
            near: PlainFloat::new(1048576.0),
            unused_bc: [PlainDword::new(0x3020100), PlainDword::new(0x7060504)],
            turned_corner_order: [
                PlainDword::new(0x2030001),
                PlainDword::new(0x5040706),
                PlainDword::new(0x1000302),
                PlainDword::new(0x6070405),
                PlainDword::new(0x3020100),
                PlainDword::new(0x7060504),
                PlainDword::new(0x7060504),
                PlainDword::new(0x10203),
                PlainDword::new(0x3020100),
                PlainDword::new(0x7060504),
                PlainDword::new(0x3020100),
                PlainDword::new(0x7060504),
                PlainDword::new(0x3020100),
                PlainDword::new(0x7060504),
            ],
            corner_uv_offsets: [
                PlainDword::new(0),
                PlainDword::new(4),
                PlainDword::new(8),
                PlainDword::new(0xC),
                PlainDword::new(4),
                PlainDword::new(0),
                PlainDword::new(0xC),
                PlainDword::new(8),
                PlainDword::new(8),
                PlainDword::new(0xC),
                PlainDword::new(0),
                PlainDword::new(4),
                PlainDword::new(0xC),
                PlainDword::new(8),
                PlainDword::new(4),
                PlainDword::new(0),
                PlainDword::new(8),
                PlainDword::new(0),
                PlainDword::new(0xC),
                PlainDword::new(4),
                PlainDword::new(0xC),
                PlainDword::new(4),
                PlainDword::new(8),
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(8),
                PlainDword::new(4),
                PlainDword::new(0xC),
                PlainDword::new(4),
                PlainDword::new(0xC),
                PlainDword::new(0),
                PlainDword::new(8),
            ],
            level_from: PlainFloat::new(75.0),
            unused_1a8: [PlainDword::new(1), PlainDword::new(0), PlainDword::new(0)],
            players: PlainDword::new(1),
            view_cars: [PlainDword::new(0), PlainDword::new(1)],
            view_blocks: [
                PlainDword::new(crate::race::blocks_store::block_ref(0x70F3EC)),
                PlainDword::new(crate::race::blocks_store::block_ref(0x7103CC)),
            ],
            view_depth: [PlainDword::new(0xC0), PlainDword::new(0xC0)],
            ..ZeroCells::ZERO
        }
    }
}

impl RaceView {
    /// The number of view `n`'s car block, refused for none.
    pub fn view_number(&self, n: usize) -> Result<usize, crate::errors::BadInput> {
        crate::race::blocks_store::number_of(self.view_blocks[n].get())
    }

    /// The car of drawn view `drawn` (`view_cars`[drawn]`). The original does not bound the
    /// index, but every drawn view is 0 or 1: a part's own view number modulo 2, or for the
    /// players' scenes `drawn_view`, which only `race::draw_race_scene` writes (0, or the
    /// player 0 .. 1). One past the two is refused.
    pub fn drawn_view_car(&self, drawn: u32) -> Result<u32, crate::errors::BadInput> {
        self.view_cars
            .get(drawn as usize)
            .map(PlainDword::get)
            .ok_or(crate::errors::BadInput("a drawn view past the two"))
    }
}
