//! The course faces' work and the race's views (IDA 0x63D9B8 .. 0x63DC5C; phase 7, typed): the
//! face being drawn (corners, textures, orders, coordinates, colours and lights by corner,
//! `race::course_faces`), the polygon mode, the lamps, the views (which are on, their cars and
//! blocks, the drawn view), the race's cars and players, the frame clock, the depth and sort
//! counts. `#[repr(C)]` with the image's layout, over the race block's bytes (`RACE_VIEW`), as
//! `CAR`. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceView {
    /// +0x0 (0x63D9B8): `g_reflectionsKept` (`scene::cached_reflection_coordinates`); now each view's (`race::face_work::FaceWork`); unused.
    pub reflections_kept: Dword,
    /// +0x4 (0x63D9BC): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_4: [Dword; 5],
    /// +0x18 (0x63D9D0): the face's surface kinds (`race::course_faces`, `physics::wheel_surfaces`, `race::best_line`).
    pub surface_kinds: [Dword; 4],
    /// +0x28 (0x63D9E0): `g_depthTest` (`race::course_faces`); now each view's (`FaceWork::depth_test`); unused.
    pub depth_test: Dword,
    /// +0x2c (0x63D9E4): the face's halves turned to the view (`race::course_faces`); now each view's (`race::face_work::FaceWork`); unused.
    pub face_turned: [Dword; 2],
    /// +0x34 (0x63D9EC): the face's corners ([3] -1 for faces of three points, `race::norms`); now each view's (`race::face_work::FaceWork`); here only the image's first values, which
    /// `race::norms` reads before any face was drawn (`face_work::last_fourth_corner`).
    pub corners: [Dword; 4],
    /// +0x44 (0x63D9FC): (`race::course_faces`).
    pub slope: Float,
    /// +0x48 (0x63DA00): (`race::course_faces`).
    pub near: Float,
    /// +0x4c (0x63DA04): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_4c: Dword,
    /// +0x50 (0x63DA08): the frame the face was drawn in (`race::course_faces`).
    pub face_frame: Dword,
    /// +0x54 (0x63DA0C): the first texture's coordinates by corner, x then y; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_uv: [Dword; 8],
    /// +0x74 (0x63DA2C): the first texture (`race::course_faces`); now each view's (`race::face_work::FaceWork`); unused.
    pub texture_a: Dword,
    /// +0x78 (0x63DA30): its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_a: Dword,
    /// +0x7c (0x63DA34): its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_a: Dword,
    /// +0x80 (0x63DA38): the second texture (`scene::draw_polygon`'s material); now each view's (`race::face_work::FaceWork`); unused.
    pub texture_b: Dword,
    /// +0x84 (0x63DA3C): its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_b: Dword,
    /// +0x88 (0x63DA40): its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_b: Dword,
    /// +0x8c (0x63DA44): the colour by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_colours: [Dword; 4],
    /// +0x9c (0x63DA54): the light by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_lights: [Dword; 4],
    /// +0xac (0x63DA64): the see-through by corner (a byte); now each view's (`race::face_work::FaceWork`); unused.
    pub corner_alphas: [Dword; 4],
    /// +0xbc (0x63DA74): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_bc: [Dword; 2],
    /// +0xc4 (0x63DA7C): `g_turnedCornerOrder`: the turned orders, a byte table read in place (`race::course_faces`).
    pub turned_corner_order: [Dword; 14],
    /// +0xfc (0x63DAB4): `g_polygonMode`; now each view's (`race::face_work::FaceWork`); unused.
    pub polygon_mode: Dword,
    /// +0x100 (0x63DAB8): the second texture's coordinates by corner, x then y; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_uv_b: [Dword; 8],
    /// +0x120 (0x63DAD8): `g_cornerUvOffsets`: by order, four.
    pub corner_uv_offsets: [Dword; 32],
    /// +0x1a0 (0x63DB58): `g_lampsWanted`; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamps_wanted: Dword,
    /// +0x1a4 (0x63DB5C): (`drive::share_car_state`).
    pub level_from: Float,
    /// +0x1a8 (0x63DB60): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1a8: [Dword; 3],
    /// +0x1b4 (0x63DB6C): `g_viewLamps`: 0 from `race::draw_view_scene`, read by nothing; no longer written.
    pub view_lamps: Dword,
    /// +0x1b8 (0x63DB70): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1b8: Dword,
    /// +0x1bc (0x63DB74): each view on (`race::draw_race_scene`, `race::show_replay_still`).
    pub view_on: [Dword; 2],
    /// +0x1c4 (0x63DB7C): the two players' option bits (`drive::replay_cameras`, `race::settings_copy`).
    pub player_options: [Dword; 2],
    /// +0x1cc (0x63DB84): the race's ticks run while below it (`drive::race_tick`; `race::credits_race`).
    pub after_credits: Dword,
    /// +0x1d0 (0x63DB88): (`drive::choose_shown_cars`, `drive::replay_cameras`).
    pub view_switch: Dword,
    /// +0x1d4 (0x63DB8C): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_1d4: Dword,
    /// +0x1d8 (0x63DB90): `g_raceCars`: the race's cars.
    pub race_cars: Dword,
    /// +0x1dc (0x63DB94): the players (`settings.race_mode`; `[shaunnumplayers]`).
    pub players: Dword,
    /// +0x1e0 (0x63DB98): the views shown (`race::draw_race_scene`'s one view; the second player's).
    pub views: Dword,
    /// +0x1e4 (0x63DB9C): `g_drawnView`.
    pub drawn_view: Dword,
    /// +0x1e8 (0x63DBA0): this machine's car (`net::lets_race`).
    pub local_car: Dword,
    /// +0x1ec (0x63DBA4): `g_shownCar`: the drawn view's car, read only by `race::car_light_modes`, which keeps it itself; no longer written.
    pub shown_car: Dword,
    /// +0x1f0 (0x63DBA8): each view's car.
    pub view_cars: [Dword; 2],
    /// +0x1f8 (0x63DBB0): `g_viewBlocks`: each view's car block (by handle:
    /// `blocks_store::block_ref`; read through `view_block`).
    pub view_blocks: [Dword; 2],
    /// +0x200 (0x63DBB8): cleared by `race::draw_race_scene`.
    pub view_cleared: [Dword; 32],
    /// +0x280 (0x63DC38): (`race::show_frame`).
    pub clock_base: Dword,
    /// +0x284 (0x63DC3C): (`race::show_frame`).
    pub frame_time: Dword,
    /// +0x288 (0x63DC40): `g_faceDepthKey`; now each view's (`race::face_work::FaceWork`); unused.
    pub face_depth_key: Dword,
    /// +0x28c (0x63DC44): (`race::frame_draw`); written only, by nothing now (a dead store); unused.
    pub face_side: Dword,
    /// +0x290 (0x63DC48): `g_lampCount`; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamp_count: Dword,
    /// +0x294 (0x63DC4C): `g_depthEntryCount`.
    pub depth_entry_count: Dword,
    /// +0x298 (0x63DC50): `g_sortEntryCount`.
    pub sort_entry_count: Dword,
    /// +0x29c (0x63DC54): how far is seen times 0x20, twice (`scene::set_view_window`); now each view's (`scene::view_window`); unused.
    pub view_depth: [Dword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceView {
    /// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
    /// `race::store::LEFT`), with the image's first values.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::blocks_store::with_block_refs(
            crate::race::store::first_values::<{ END - START }>(START),
            crate::race::blocks_store::VIEW_BLOCKS_IDA - START,
            2,
        ))
    }
}

impl RaceView {
    /// View `n`'s car block (`g_viewBlocks[n]`, kept by handle: `blocks_store::block_of`).
    pub fn view_block(&self, n: usize) -> u32 {
        crate::race::blocks_store::block_of(self.view_blocks[n].get())
    }

    /// View `n`'s car block (None for none).
    pub fn view_car(&self, n: usize) -> Option<&'static crate::physics::car_store::Car> {
        crate::race::blocks_store::car_of(self.view_blocks[n].get())
    }
}

/// IDA 0x63D9B8, the first byte of the piece.
pub const START: usize = 0x63D9B8;
/// IDA 0x63DC5C, the end of the piece.
pub const END: usize = 0x63DC5C;

const _: () = assert!(END - START == core::mem::size_of::<RaceView>());

/// Where IDA 0x63D9B8 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().shared.view))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        let at = crate::race::blocks_store::VIEW_BLOCKS_IDA - START;
        crate::gfx::first_values_are_the_games_but(&race().shared.view, START, END, |image| {
            for k in 0..2 {
                let o = at + 4 * k;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4]
                    .copy_from_slice(&crate::race::blocks_store::block_ref(v).to_le_bytes());
            }
        });
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(RaceView, reflections_kept), 0x63D9B8 - START);
        assert_eq!(offset_of!(RaceView, near), 0x63DA00 - START);
        assert_eq!(offset_of!(RaceView, texture_b), 0x63DA38 - START);
        assert_eq!(offset_of!(RaceView, turned_corner_order), 0x63DA7C - START);
        assert_eq!(offset_of!(RaceView, view_lamps), 0x63DB6C - START);
        assert_eq!(offset_of!(RaceView, players), 0x63DB94 - START);
        assert_eq!(offset_of!(RaceView, view_blocks), 0x63DBB0 - START);
        assert_eq!(offset_of!(RaceView, lamp_count), 0x63DC48 - START);
    }
}
