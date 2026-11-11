//! The texture places, 0x5D2AE8 .. 0x5E9EC8: which texture each slot shows, the pictures wanted
//! and loaded, the counters, the car icon sheets and the shapes' marks. Follows the kind
//! texture tables (`scene::kind_texture_store`); the matrices follow (`race::matrices_store`).
//! Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TexturePlaces {
    /// +0x0 (0x5D2AE8): `dword_5D2AE8`: the texture of each slot (`scene::try_allocate_textures`).
    pub texture_of: [Dword; 1024],
    /// +0x1000 (0x5D3AE8): `dword_5D3AE8`: each place's slot (`scene::look_up_texture`, `scene::map_texture_places`, `scene::draw_polygon`, `race::frame_draw`). [1999] is `dword_5D5A24`, the windscreen drops' texture (`effects::windscreen_drops_setup`).
    pub place_slot: [Dword; 4000],
    /// +0x4e80 (0x5D7968): `g_kindPictures`: the picture number of each kind (IDA types it int[4]; the code reaches [0x87], [0x258] ..) (`race::set_all_plates`, `scene::reload_system_textures`, `scene::set_up_front_end_scene`).
    pub kind_pictures: [Dword; 1024],
    /// +0x5e80 (0x5D8968): `g_textureWantedPictures`: the picture each texture should hold (`scene::load_changed_texture_picture`, `scene::reset_texture_places`, `scene::textures`).
    pub wanted_pictures: [Dword; 1024],
    /// +0x6e80 (0x5D9968): `g_textureLoadedPictures`: the picture each holds (same users).
    pub loaded_pictures: [Dword; 1024],
    /// +0x7e80 (0x5DA968): `g_slotWantedTextures`: the texture each slot should show (`scene::clear_slot_textures`, `scene::textures`, `race::draw_race_scene`).
    pub slot_wanted: [Dword; 1024],
    /// +0x8e80 (0x5DB968): `g_slotLoadedTextures`: the texture each slot shows (`scene::forget_slot_texture`, `scene::textures`).
    pub slot_loaded: [Dword; 1024],
    /// +0x9e80 (0x5DC968): `dword_5DC968`, `dword_5DC96C`, `g_textureSlotCount`, `g_systemTextureCount`: video k, system k, video textures, system textures (`scene::count_texture_needs`); [2] and [3] are read alone (`scene::look_up_texture`, `scene::draw_polygon`, `scene::reload_system_textures`, `scene::step_loading_bar`).
    pub totals: [Dword; 4],
    /// +0x9e90 (0x5DC978): `g_kindSlotBytes`: what a kind's slots take (`scene::texture_levels`, `scene::reset_texture_tables`).
    pub kind_slot_bytes: Dword,
    /// +0x9e94 (0x5DC97C): `dword_5DC97C`: 0x100 counters (`scene::build_texture_catalogue`).
    pub counters: [Dword; 256],
    /// +0xa294 (0x5DCD7C): `dword_5DCD7C` .. `dword_5DCD98`: the texture steps' flags (`scene::texture_steps`, `scene::build_texture_catalogue`).
    pub step_flags: [Dword; 8],
    /// +0xa2b4 (0x5DCD9C): `g_carIconSheetX`: the car icons' sheet x (`scene::load_hud_car_icons`).
    pub car_icon_sheet_x: [Dword; 36],
    /// +0xa344 (0x5DCE2C): `g_carIconSheetY`: the sheet y (`scene::load_hud_car_icons`).
    pub car_icon_sheet_y: [Dword; 36],
    /// +0xa3d4 (0x5DCEBC): `unk_5DCEBC`: the objects' texture lists, a count then the textures (`scene::list_object_textures`).
    pub object_texture_records: [Dword; 1024],
    /// +0xb3d4 (0x5DDEBC): `dword_5DDEBC`: the shape's faces taken (`scene::add_object_shape`).
    pub face_marks: [Dword; 4096],
    /// +0xf3d4 (0x5E1EBC): `dword_5E1EBC`: the points' marks (`scene::add_object_shape`).
    pub point_marks: [Dword; 4096],
    /// +0x133d4 (0x5E5EBC): `dword_5E5EBC`: the normals' marks (`scene::add_object_shape`); three dwords more follow.
    pub normal_marks: [Dword; 4096],
    /// +0x173d4 (0x5E9EBC): three dwords past the normals' marks; nothing reads or writes them (an overrun space for the marks table, probably padding).
    pub unknown_173d4: [Dword; 3],
}

/// The cell `index` entries on from the table `table_offset` bytes into the places. The
/// game does not bound these indices (slot and counter numbers from its data): a number past
/// the table reads on into the next fields of the places in their order, as the original's
/// by-address read did; past the places it is refused.
pub fn cell_at(
    places: &TexturePlaces,
    table_offset: usize,
    index: u32,
) -> Result<&Dword, crate::errors::BadInput> {
    let at = (index as usize)
        .checked_mul(4)
        .and_then(|bytes| table_offset.checked_add(bytes));
    at.and_then(|at| places.cell(at))
        .ok_or(crate::errors::BadInput("a cell past the texture places"))
}

impl TexturePlaces {
    /// The dword `at` bytes into the places (every field is a run of dwords, in order).
    fn cell(&self, at: usize) -> Option<&Dword> {
        use core::mem::offset_of;
        let fields: [(usize, &[Dword]); 18] = [
            (offset_of!(Self, texture_of), &self.texture_of),
            (offset_of!(Self, place_slot), &self.place_slot),
            (offset_of!(Self, kind_pictures), &self.kind_pictures),
            (offset_of!(Self, wanted_pictures), &self.wanted_pictures),
            (offset_of!(Self, loaded_pictures), &self.loaded_pictures),
            (offset_of!(Self, slot_wanted), &self.slot_wanted),
            (offset_of!(Self, slot_loaded), &self.slot_loaded),
            (offset_of!(Self, totals), &self.totals),
            (
                offset_of!(Self, kind_slot_bytes),
                core::slice::from_ref(&self.kind_slot_bytes),
            ),
            (offset_of!(Self, counters), &self.counters),
            (offset_of!(Self, step_flags), &self.step_flags),
            (offset_of!(Self, car_icon_sheet_x), &self.car_icon_sheet_x),
            (offset_of!(Self, car_icon_sheet_y), &self.car_icon_sheet_y),
            (
                offset_of!(Self, object_texture_records),
                &self.object_texture_records,
            ),
            (offset_of!(Self, face_marks), &self.face_marks),
            (offset_of!(Self, point_marks), &self.point_marks),
            (offset_of!(Self, normal_marks), &self.normal_marks),
            (offset_of!(Self, unknown_173d4), &self.unknown_173d4),
        ];
        if !at.is_multiple_of(4) {
            return None;
        }
        fields.into_iter().find_map(|(start, field)| {
            let inside = at.checked_sub(start)?;
            field.get(inside / 4)
        })
    }
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl TexturePlaces {
    /// The piece with the image's first values.
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x5D2AE8, the first byte of the piece.
pub const START: usize = 0x5D2AE8;
/// IDA 0x5E9EC8, the end of the piece.
pub const END: usize = 0x5E9EC8;

const _: () = assert!(END - START == core::mem::size_of::<TexturePlaces>());

/// Where IDA 0x5D2AE8 is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::scene::owner::scene().textures.places,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::owner::scene;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&scene().textures.places, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            piece_addr(),
            crate::framework::game_addr(crate::framework::rva(START))
        );
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(TexturePlaces, texture_of), 0x5D2AE8 - START);
        assert_eq!(offset_of!(TexturePlaces, kind_slot_bytes), 0x5DC978 - START);
        assert_eq!(offset_of!(TexturePlaces, normal_marks), 0x5E5EBC - START);
    }
}
