//! The scene's light and object-drawing work (IDA 0x62DA44 .. 0x62DB5C; phase 7, typed): the object
//! being drawn, the lamp tint and the point's place, the light-dot inputs, the light's colour and
//! kind, the lamp and ground colours the car lighting keeps, the plain light colours, the
//! material phases' lengths and clock, and the count of listed faces. `#[repr(C)]` with the image's
//! layout, over the race block's bytes. Names are readings unless IDA's.

use crate::session::{Dword, Float};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct LightWork {
    /// +0x0 (0x62DA44): `g_drawingObject`: the object being drawn (`scene::draw_shown_object`, `scene::log_bad_polygon`).
    pub drawing_object: Dword,
    /// +0x4 (0x62DA48): not named.
    pub unknown_4: [Dword; 13],
    /// +0x38 (0x62DA7C): `dword_62DA7C`: the lamp's tint taken from its entry (`scene::add_point_lights`, `race::frame_draw`).
    pub lamp_tint: Dword,
    /// +0x3c (0x62DA80): not named.
    pub unknown_3c: [Dword; 3],
    /// +0x48 (0x62DA8C): `dword_62DA8C` ..: the point's place (`scene::draw_object_face`; read by `fsub`).
    pub point_place: [Float; 3],
    /// +0x54 (0x62DA98): `flt_62DA98` .. `flt_62DAA4`: float work of the lighting (a reading; written and read in the object drawing code only).
    pub light_work_floats: [Float; 4],
    /// +0x64 (0x62DAA8): `dword_62DAA8` ..: the point of `scene::light_dot`'s inputs (a reading: floats, loaded by `fld`).
    pub light_dot_point: [Float; 3],
    /// +0x70 (0x62DAB4): `dword_62DAB4` ..: the light's vector in `scene::light_dot`'s inputs (a reading).
    pub light_dot_light: [Float; 3],
    /// +0x7c (0x62DAC0): `flt_62DAC0`: the factor in `scene::light_dot`'s inputs.
    pub light_dot_factor: Float,
    /// +0x80 (0x62DAC4): `g_lightColour`: the light's colour (`scene::vertex_light`, `scene::add_point_lights`, `race::course_faces`, `race::frame_draw`).
    pub light_colour: Dword,
    /// +0x84 (0x62DAC8): not named.
    pub unknown_84: Dword,
    /// +0x88 (0x62DACC): `dword_62DACC`: the lamp parts' mask (`race::car_parts`, `race::draw_race_scene`, `race::loadlevel`).
    pub lamps_mask: Dword,
    /// +0x8c (0x62DAD0): `dword_62DAD0`: the lone-group switch / marks mask (`race::car_parts`, `physics::drop_broken_parts`, `scene::list_object_face`, `race::draw_race_scene`, `race::loadlevel`).
    pub marks_mask: Dword,
    /// +0x90 (0x62DAD4): `dword_62DAD4`: the base light colour of the view or car (`scene::vertex_light`, `race::car_draw`, `race::frame_draw`, `race::frame_views`).
    pub view_light_base: Dword,
    /// +0x94 (0x62DAD8): not named.
    pub unknown_94: [Dword; 2],
    /// +0x9c (0x62DAE0): `xmmword_62DAE0`: 16 bytes the point records are cleared with by SSE (`race::frame_draw`); 8 bytes of padding before it are unused.
    pub sse_fill: [Dword; 4],
    /// +0xac (0x62DAF0): `dword_62DAF0`: the fog's step (`race::frame_draw`) (a reading).
    pub fog_step: Dword,
    /// +0xb0 (0x62DAF4): `g_lightKind`: the kind of light being set (`scene::set_object_light`, `scene::vertex_light`, `race::car_draw`, `scene::draw_described_object`, `scene::draw_menu_object`, `race::frame_draw`).
    pub light_kind: Dword,
    /// +0xb4 (0x62DAF8): `dword_62DAF8`: set with the light kind, 0 for a car (`race::car_draw`) (a reading).
    pub light_kind_extra: Dword,
    /// +0xb8 (0x62DAFC): `dword_62DAFC`: the car's light colour (`physics::car_lighting`).
    pub car_light_colour: Dword,
    /// +0xbc (0x62DB00): `dword_62DB00`: first lamp table's colour (`physics::car_lighting`).
    pub lamp_a_colour: Dword,
    /// +0xc0 (0x62DB04): `dword_62DB04`: second lamp table's colour (`physics::car_lighting`).
    pub lamp_b_colour: Dword,
    /// +0xc4 (0x62DB08): `dword_62DB08`: the ground's texture s kept (`physics::car_lighting`).
    pub ground_texture_s: Dword,
    /// +0xc8 (0x62DB0C): `dword_62DB0C`: the ground's texture t kept (`physics::car_lighting`).
    pub ground_texture_t: Dword,
    /// +0xcc (0x62DB10): `dword_62DB10` (int[8]): the plain light tables' colours, two colours (`scene::fill_light_tables`).
    pub plain_light_colours: [Dword; 8],
    /// +0xec (0x62DB30): `dword_62DB30` (nine dwords to 0x62DB54): four bases, four random parts and a 0, initial 200, 400, 200, 400, 400, 400, 400, 400, 0 (`scene::set_object_material`).
    pub phase_lengths: [Dword; 9],
    /// +0x110 (0x62DB54): `dword_62DB54`: the clock when `scene::set_object_material` was last here.
    pub phase_clock: Dword,
    /// +0x114 (0x62DB58): `g_listedFaceCount`: faces in the list (`scene::list_object_face`, `scene::draw_object_faces`, `scene::sort_listed_faces`).
    pub listed_face_count: Dword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl LightWork {
    /// The piece with the image's first values (a static of its own since 2026-10-29: it left
    /// the race block, phase 13, `race::store::LEFT`; the owner `scene::owner::Scene` holds it).
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x62DA44, the first byte of the piece.
pub const START: usize = 0x62DA44;
/// IDA 0x62DB5C, the end of the piece.
pub const END: usize = 0x62DB5C;

const _: () = assert!(END - START == core::mem::size_of::<LightWork>());

/// Where IDA 0x62DA44 is now (the piece's static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(std::ptr::from_ref(
        &crate::scene::owner::scene().lights.work,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::owner::scene;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(scene().lights.work.first(), START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(offset_of!(LightWork, drawing_object), 0x62DA44 - START);
        assert_eq!(offset_of!(LightWork, light_dot_factor), 0x62DAC0 - START);
        assert_eq!(offset_of!(LightWork, fog_step), 0x62DAF0 - START);
        assert_eq!(offset_of!(LightWork, ground_texture_s), 0x62DB08 - START);
    }
}
