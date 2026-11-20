//! The scene's draw variables. `SceneDraw` holds what the drawing parts read (the polygons' alpha,
//! the fade and clip factors, the mirror picture's texture, the `v3driverfix` setting), a lent
//! store (`session::DrawShared`) written on the game's thread between frames and read through
//! `scene::workers::DrawReads`. `DrawWork` holds the work words only the game's thread reaches
//! (the x87 code's shared dword, the physics' work floats, the turns' sines): plain fields
//! (`scene::owner::Scene::draw_work`).

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SceneDraw {
    /// The polygon's alpha (set by `hud::draw_view_strip` only, read by every view).
    pub polygon_alpha: PlainDword,
    /// The fade's second figure (`scene::set_view_window`).
    pub fade_b: PlainFloat,
    /// The fade's first figure (`scene::set_view_window`).
    pub fade_a: PlainFloat,
    /// The farthest clip depth is the farthest depth times it.
    pub clip_factor: PlainFloat,
    /// The mirror picture's texture.
    pub mirror_picture: PlainDword,
    /// The `v3driverfix` setting.
    pub v3_driver_fix: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl SceneDraw {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            polygon_alpha: PlainDword::new(0xFF),
            fade_b: PlainFloat::new(1.5),
            fade_a: PlainFloat::new(0.9),
            clip_factor: PlainFloat::new(0.9),
            ..ZeroCells::ZERO
        }
    }
}

/// The work words of the game's thread: plain fields, borrowed for one statement at a time
/// (`Scene::draw_work`, `Scene::draw_work_mut`).
#[derive(Clone, Debug)]
pub struct DrawWork {
    /// A work dword (`physics::engine_torque`, `physics::wheel_springs`);
    /// `race::find_present_courses`' four-byte probe.
    pub work_dword: u32,
    /// The camera's heading, saved round a step (`physics::camera_avoid_ground`).
    pub camera_heading_saved: u32,
    /// The camera's heading kept from step to step (`physics::camera_avoid_ground`).
    pub camera_heading_kept: u32,
    /// Five words `physics::camera_avoid_ground` keeps.
    pub camera_saved: [u32; 5],
    /// The x87 code's shared work dword: every converted helper that stored to it still does.
    pub scratch: u32,
    /// A work float (`physics::drive_torque_share`, `physics::drive_grip_forces`,
    /// `physics::drive_wheels_step`).
    pub work_float_a: f32,
    /// A work float (the same).
    pub work_float_b: f32,
    /// A wheel's surface point: x, then y and z less x (`physics::wheel_surfaces`).
    pub surface_point: [u32; 3],
    /// Sine and cosine of the x turn (`scene::matrix`).
    pub turn_x_work: [f32; 2],
    /// The same, y.
    pub turn_y_work: [f32; 2],
    /// The same, z (`scene::view_matrix`).
    pub turn_z_work: [f32; 2],
    /// The view window's mode (`scene::choose_view_window`, `scene::full_view_window`).
    pub view_mode: u16,
}

impl DrawWork {
    /// The work words with their first values (the ones not named here start at zero).
    pub const fn new() -> Self {
        Self {
            work_dword: 0,
            camera_heading_saved: 0,
            camera_heading_kept: 0,
            camera_saved: [0; 5],
            scratch: 0x10000,
            work_float_a: 0.0,
            work_float_b: f32::from_bits(0x00010000),
            surface_point: [0x10000; 3],
            turn_x_work: [0.0; 2],
            turn_y_work: [0.0; 2],
            turn_z_work: [0.0; 2],
            view_mode: 0,
        }
    }
}

/// The shared scratch dword as a cell, for the x87 helpers that take their work word as one: it
/// starts at the scene's value and puts its value back when it is dropped.
pub struct ScratchCell<'a> {
    scene: &'a crate::scene::owner::Scene,
    cell: PlainDword,
}

impl<'a> ScratchCell<'a> {
    pub fn new(scene: &'a crate::scene::owner::Scene) -> Self {
        Self {
            scene,
            cell: PlainDword::new(scene.draw_work().scratch),
        }
    }
}

impl std::ops::Deref for ScratchCell<'_> {
    type Target = PlainDword;

    fn deref(&self) -> &PlainDword {
        &self.cell
    }
}

impl Drop for ScratchCell<'_> {
    fn drop(&mut self) {
        self.scene.draw_work_mut().scratch = self.cell.get();
    }
}
