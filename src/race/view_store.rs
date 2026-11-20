//! The course faces' work and the race's views: the face being drawn (corners, textures, orders,
//! coordinates, colours and lights by corner, `race::course_faces`), the polygon mode, the lamps,
//! the views (which are on, their cars and blocks, the drawn view), the race's cars and players,
//! the frame clock, the depth and sort counts. `#[repr(C)]`, laid over the race block bytes.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct RaceView {
    /// Not 0 while a replay of a network race is viewed: the headlights then light the road
    /// for the drawn view's car (`race::frame_draw::lights`), as a network race's for its own.
    pub lights_follow_view: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_4: [PlainDword; 5],
    /// The face's surface kinds (`race::course_faces`, `physics::wheel_surfaces`,
    /// `race::best_line`).
    pub surface_kinds: [PlainDword; 4],
    /// The depth test (`race::course_faces`); now each view's (`FaceWork::depth_test`);
    /// unused.
    pub depth_test: PlainDword,
    /// The face's halves turned to the view (`race::course_faces`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub face_turned: [PlainDword; 2],
    /// The face's corners ([3] -1 for faces of three points, `race::norms`); now each view's
    /// (`race::face_work::FaceWork`); here only the starting values, which `race::norms` reads
    /// before any face was drawn (`face_work::last_fourth_corner`).
    pub corners: [PlainDword; 4],
    /// The face's slope (`race::course_faces`).
    pub slope: PlainFloat,
    /// The face's near value (`race::course_faces`).
    pub near: PlainFloat,
    /// Unused; it only keeps the layout.
    pub unused_4c: PlainDword,
    /// The frame the face was drawn in (`race::face_materials` wrote it; nothing read it, so
    /// it is no longer written); unused.
    pub face_frame: PlainDword,
    /// The first texture's coordinates by corner, x then y; now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub corner_uv: [PlainDword; 8],
    /// The first texture (`race::course_faces`); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub texture_a: PlainDword,
    /// Its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_a: PlainDword,
    /// Its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_a: PlainDword,
    /// The second texture (`scene::draw_polygon`'s material); now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub texture_b: PlainDword,
    /// Its corner order; now each view's (`race::face_work::FaceWork`); unused.
    pub order_b: PlainDword,
    /// Its back; now each view's (`race::face_work::FaceWork`); unused.
    pub back_b: PlainDword,
    /// The colour by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_colours: [PlainDword; 4],
    /// The light by corner; now each view's (`race::face_work::FaceWork`); unused.
    pub corner_lights: [PlainDword; 4],
    /// The see-through by corner (a byte); now each view's (`race::face_work::FaceWork`);
    /// unused.
    pub corner_alphas: [PlainDword; 4],
    /// Unused; it only keeps the layout.
    pub unused_bc: [PlainDword; 2],
    /// The turned orders, a byte table read in place (`race::course_faces`).
    pub turned_corner_order: [PlainDword; 14],
    /// The polygon mode; now each view's (`race::face_work::FaceWork`); unused.
    pub polygon_mode: PlainDword,
    /// The second texture's coordinates by corner, x then y; now each view's
    /// (`race::face_work::FaceWork`); unused.
    pub corner_uv_b: [PlainDword; 8],
    /// The corner texture offsets, four by order.
    pub corner_uv_offsets: [PlainDword; 32],
    /// The lamps wanted; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamps_wanted: PlainDword,
    /// The level (`drive::share_car_state`).
    pub level_from: PlainFloat,
    /// Unused; it only keeps the layout.
    pub unused_1a8: [PlainDword; 3],
    /// 0 from `race::draw_view_scene`, never read; no longer written.
    pub view_lamps: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_1b8: PlainDword,
    /// Each view on (`race::draw_race_scene`, `race::show_replay_still`).
    pub view_on: [PlainDword; 2],
    /// The two players' option bits (`drive::replay_cameras`, `race::settings_copy`).
    pub player_options: [PlainDword; 2],
    /// The race's ticks run while below it (`drive::race_tick`; `race::credits_race`).
    pub after_credits: PlainDword,
    /// The view switch (`drive::choose_shown_cars`, `drive::replay_cameras`).
    pub view_switch: PlainDword,
    /// Unused; it only keeps the layout.
    pub unused_1d4: PlainDword,
    /// The race's cars.
    pub race_cars: PlainDword,
    /// The players (`settings.race_mode`; `[shaunnumplayers]`).
    pub players: PlainDword,
    /// The views shown (`race::draw_race_scene`'s one view; the second player's).
    pub views: PlainDword,
    /// The view being drawn.
    pub drawn_view: PlainDword,
    /// This machine's car (`net::lets_race`).
    pub local_car: PlainDword,
    /// The drawn view's car, read only by `race::car_light_modes`, which keeps it itself;
    /// no longer written.
    pub shown_car: PlainDword,
    /// Each view's car.
    pub view_cars: [PlainDword; 2],
    /// Each view's car block (by handle:
    /// `blocks_store::block_ref`; read through `view_block`).
    pub view_blocks: [PlainDword; 2],
    /// Cleared by `race::draw_race_scene`.
    pub view_cleared: [PlainDword; 32],
    /// The clock at the frame's start (`race::show_frame`).
    pub clock_base: PlainDword,
    /// The frame's time (`race::show_frame`).
    pub frame_time: PlainDword,
    /// The face's depth key; now each view's (`race::face_work::FaceWork`); unused.
    pub face_depth_key: PlainDword,
    /// The face's side (`race::frame_draw`); written only, never read; unused.
    pub face_side: PlainDword,
    /// The lamp count; now each view's (`race::lamp_work::LampWork`); unused.
    pub lamp_count: PlainDword,
    /// The depth entry count.
    pub depth_entry_count: PlainDword,
    /// The sort entry count.
    pub sort_entry_count: PlainDword,
    /// How far is seen times 0x20, twice (`scene::set_view_window`); now each view's
    /// (`scene::view_window`); unused.
    pub view_depth: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl RaceView {
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
                PlainDword::new(crate::race::blocks_store::block_handle(0)),
                PlainDword::new(crate::race::blocks_store::block_handle(1)),
            ],
            view_depth: [PlainDword::new(0xC0), PlainDword::new(0xC0)],
            ..ZeroCells::ZERO
        }
    }
}

impl RaceView {
    /// The face's surface kind `n`. The index is a byte of the stage's texture record, not held to
    /// the four kinds: past them the fields after the table are read, in order.
    pub fn surface_kind(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(kind) = self.surface_kinds.get(n) {
            return Ok(kind.get());
        }
        crate::memory::session::read_runs(
            &[
                &self.surface_kinds,
                &self.depth_test,
                &self.face_turned,
                &self.corners,
                &self.slope,
                &self.near,
                &self.unused_4c,
                &self.face_frame,
                &self.corner_uv,
                &self.texture_a,
                &self.order_a,
                &self.back_a,
                &self.texture_b,
                &self.order_b,
                &self.back_b,
                &self.corner_colours,
                &self.corner_lights,
                &self.corner_alphas,
                &self.unused_bc,
                &self.turned_corner_order,
                &self.polygon_mode,
                &self.corner_uv_b,
                &self.corner_uv_offsets,
                &self.lamps_wanted,
                &self.level_from,
                &self.unused_1a8,
                &self.view_lamps,
                &self.unused_1b8,
                &self.view_on,
                &self.player_options,
                &self.after_credits,
                &self.view_switch,
                &self.unused_1d4,
                &self.race_cars,
                &self.players,
                &self.views,
                &self.drawn_view,
                &self.local_car,
                &self.shown_car,
                &self.view_cars,
                &self.view_blocks,
                &self.view_cleared,
                &self.clock_base,
                &self.frame_time,
                &self.face_depth_key,
                &self.face_side,
                &self.lamp_count,
                &self.depth_entry_count,
                &self.sort_entry_count,
                &self.view_depth,
            ],
            n,
            "the surface kinds (a byte index past the table)",
        )
    }

    /// The number of view `n`'s car block, refused for none.
    pub(super) fn view_number(&self, n: usize) -> Result<usize, crate::errors::BadInput> {
        match self.view_blocks.get(n) {
            Some(block) => crate::race::blocks_store::number_of(block.get()),
            // The further views of a split screen show their own player's car.
            None if n < 4 => Ok(n),
            None => Err(crate::errors::BadInput("a view past the four")),
        }
    }

    /// The car view `n` shows: the first two views' (`view_cars`), and with a split screen of
    /// more than two players each further view's own player's. One past the four is refused.
    pub(super) fn view_car_number(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        match self.view_cars.get(n) {
            Some(car) => Ok(car.get()),
            None if n < 4 => Ok(n as u32),
            None => Err(crate::errors::BadInput("a view past the four")),
        }
    }

    /// The car of drawn view `drawn` (`view_cars[drawn]`). The drawn view is a part's own view
    /// number (modulo 2 with a mirror's slot), or for the players' scenes `drawn_view`, which
    /// only `race::draw_race_scene` writes (the player 0 .. 3). One past the four is refused.
    pub fn drawn_view_car(&self, drawn: u32) -> Result<u32, crate::errors::BadInput> {
        self.view_car_number(drawn as usize)
    }
}

#[cfg(test)]
mod surface_kind_tests {
    use super::*;

    /// The four kinds are read by index, and an index far past the record is refused.
    #[test]
    fn surface_kind_reads_the_kinds_and_refuses_far_past_them() {
        let view = RaceView::new();
        for (n, cell) in view.surface_kinds.iter().enumerate() {
            cell.set(0x2000 + n as u32);
            assert_eq!(
                view.surface_kind(n).ok(),
                Some(0x2000 + n as u32),
                "index {n}"
            );
        }
        assert!(view.surface_kind(1 << 20).is_err());
    }
}
