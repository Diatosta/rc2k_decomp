//! The texture places, 0x5D2AE8 .. 0x5E9EC8: which texture each slot shows, the pictures wanted
//! and loaded, the counters, the car icon sheets and the shapes' marks. Follows the kind
//! texture tables (`scene::kind_texture_store`); the matrices follow (`race::matrices_store`).
//! Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct TexturePlaces {
    /// +0x0 (0x5D2AE8): `dword_5D2AE8`: the texture of each slot (`scene::try_allocate_textures`).
    pub texture_of: [PlainDword; 1024],
    /// +0x1000 (0x5D3AE8): `dword_5D3AE8`: each place's slot (`scene::look_up_texture`, `scene::map_texture_places`, `scene::draw_polygon`, `race::frame_draw`). [1999] is `dword_5D5A24`, the windscreen drops' texture (`effects::windscreen_drops_setup`).
    pub place_slot: [PlainDword; 4000],
    /// +0x4e80 (0x5D7968): `g_kindPictures`: the picture number of each kind (IDA types it int[4]; the code reaches [0x87], [0x258] ..) (`race::set_all_plates`, `scene::reload_system_textures`, `scene::set_up_front_end_scene`).
    pub kind_pictures: [PlainDword; 1024],
    /// +0x5e80 (0x5D8968): `g_textureWantedPictures`: the picture each texture should hold (`scene::load_changed_texture_picture`, `scene::reset_texture_places`, `scene::textures`).
    pub wanted_pictures: [PlainDword; 1024],
    /// +0x6e80 (0x5D9968): `g_textureLoadedPictures`: the picture each holds (same users).
    pub loaded_pictures: [PlainDword; 1024],
    /// +0x7e80 (0x5DA968): `g_slotWantedTextures`: the texture each slot should show (`scene::clear_slot_textures`, `scene::textures`, `race::draw_race_scene`).
    pub slot_wanted: [PlainDword; 1024],
    /// +0x8e80 (0x5DB968): `g_slotLoadedTextures`: the texture each slot shows (`scene::forget_slot_texture`, `scene::textures`).
    pub slot_loaded: [PlainDword; 1024],
    /// +0x9e80 (0x5DC968): `dword_5DC968`, `dword_5DC96C`, `g_textureSlotCount`, `g_systemTextureCount`: video k, system k, video textures, system textures (`scene::count_texture_needs`); [2] and [3] are read alone (`scene::look_up_texture`, `scene::draw_polygon`, `scene::reload_system_textures`, `scene::step_loading_bar`).
    pub totals: [PlainDword; 4],
    /// +0x9e90 (0x5DC978): `g_kindSlotBytes`: what a kind's slots take (`scene::texture_levels`, `scene::reset_texture_tables`).
    pub kind_slot_bytes: PlainDword,
    /// +0x9e94 (0x5DC97C): `dword_5DC97C`: 0x100 counters (`scene::build_texture_catalogue`).
    pub counters: [PlainDword; 256],
    /// +0xa294 (0x5DCD7C): `dword_5DCD7C` .. `dword_5DCD98`: the texture steps' flags (`scene::texture_steps`, `scene::build_texture_catalogue`).
    pub step_flags: [PlainDword; 8],
    /// +0xa2b4 (0x5DCD9C): `g_carIconSheetX`: the car icons' sheet x (`scene::load_hud_car_icons`).
    pub car_icon_sheet_x: [PlainDword; 36],
    /// +0xa344 (0x5DCE2C): `g_carIconSheetY`: the sheet y (`scene::load_hud_car_icons`).
    pub car_icon_sheet_y: [PlainDword; 36],
    /// +0xa3d4 (0x5DCEBC): `unk_5DCEBC`: the objects' texture lists, a count then the textures (`scene::list_object_textures`).
    pub object_texture_records: [PlainDword; 1024],
    /// +0xb3d4 (0x5DDEBC): `dword_5DDEBC`: the shape's faces taken (`scene::add_object_shape`).
    pub face_marks: [PlainDword; 4096],
    /// +0xf3d4 (0x5E1EBC): `dword_5E1EBC`: the points' marks (`scene::add_object_shape`).
    pub point_marks: [PlainDword; 4096],
    /// +0x133d4 (0x5E5EBC): `dword_5E5EBC`: the normals' marks (`scene::add_object_shape`); three dwords more follow.
    pub normal_marks: [PlainDword; 4096],
    /// +0x173d4 (0x5E9EBC): three dwords past the normals' marks; nothing reads or writes them (an overrun space for the marks table, probably padding).
    pub unknown_173d4: [PlainDword; 3],
}

/// A table of the places, by name: the field a slot or counter number indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacesTable {
    TextureOf,
    PlaceSlot,
    KindPictures,
    WantedPictures,
    LoadedPictures,
    SlotWanted,
    SlotLoaded,
    Totals,
    KindSlotBytes,
    Counters,
    StepFlags,
    CarIconSheetX,
    CarIconSheetY,
    ObjectTextureRecords,
    FaceMarks,
    PointMarks,
    NormalMarks,
    Unknown173d4,
}

impl PlacesTable {
    /// Every table, in the order of the places.
    pub const ALL: [PlacesTable; 18] = [
        PlacesTable::TextureOf,
        PlacesTable::PlaceSlot,
        PlacesTable::KindPictures,
        PlacesTable::WantedPictures,
        PlacesTable::LoadedPictures,
        PlacesTable::SlotWanted,
        PlacesTable::SlotLoaded,
        PlacesTable::Totals,
        PlacesTable::KindSlotBytes,
        PlacesTable::Counters,
        PlacesTable::StepFlags,
        PlacesTable::CarIconSheetX,
        PlacesTable::CarIconSheetY,
        PlacesTable::ObjectTextureRecords,
        PlacesTable::FaceMarks,
        PlacesTable::PointMarks,
        PlacesTable::NormalMarks,
        PlacesTable::Unknown173d4,
    ];

    /// The table's dwords.
    fn cells(self, places: &TexturePlaces) -> &[PlainDword] {
        match self {
            PlacesTable::TextureOf => &places.texture_of,
            PlacesTable::PlaceSlot => &places.place_slot,
            PlacesTable::KindPictures => &places.kind_pictures,
            PlacesTable::WantedPictures => &places.wanted_pictures,
            PlacesTable::LoadedPictures => &places.loaded_pictures,
            PlacesTable::SlotWanted => &places.slot_wanted,
            PlacesTable::SlotLoaded => &places.slot_loaded,
            PlacesTable::Totals => &places.totals,
            PlacesTable::KindSlotBytes => core::slice::from_ref(&places.kind_slot_bytes),
            PlacesTable::Counters => &places.counters,
            PlacesTable::StepFlags => &places.step_flags,
            PlacesTable::CarIconSheetX => &places.car_icon_sheet_x,
            PlacesTable::CarIconSheetY => &places.car_icon_sheet_y,
            PlacesTable::ObjectTextureRecords => &places.object_texture_records,
            PlacesTable::FaceMarks => &places.face_marks,
            PlacesTable::PointMarks => &places.point_marks,
            PlacesTable::NormalMarks => &places.normal_marks,
            PlacesTable::Unknown173d4 => &places.unknown_173d4,
        }
    }
}

/// The cell `index` entries on from the table `table`. The
/// game does not bound these indices (slot and counter numbers from its data): a number past
/// the table reads on into the next fields of the places in their order, as the original's
/// by-address read did; past the places it is refused.
pub fn cell_at(
    places: &TexturePlaces,
    table: PlacesTable,
    index: u32,
) -> Result<&PlainDword, crate::errors::BadInput> {
    // Every field is a run of dwords, in the order of `ALL`: a number past the table's end
    // counts on into the next tables.
    let mut left = index as usize;
    for next in PlacesTable::ALL
        .into_iter()
        .skip_while(|&candidate| candidate != table)
    {
        let cells = next.cells(places);
        if let Some(cell) = cells.get(left) {
            return Ok(cell);
        }
        left -= cells.len();
    }
    Err(crate::errors::BadInput("a cell past the texture places"))
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl TexturePlaces {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            car_icon_sheet_x: {
                let mut a: [PlainDword; 36] = ZeroCells::ZERO;
                a[1] = PlainDword::new(0x52);
                a[2] = PlainDword::new(0xA4);
                a[4] = PlainDword::new(0x52);
                a[5] = PlainDword::new(0xA4);
                a[7] = PlainDword::new(0x52);
                a[8] = PlainDword::new(0xA4);
                a[10] = PlainDword::new(0x52);
                a[11] = PlainDword::new(0xA4);
                a[13] = PlainDword::new(0x52);
                a[14] = PlainDword::new(0xA4);
                a[16] = PlainDword::new(0x52);
                a[17] = PlainDword::new(0xA4);
                a[19] = PlainDword::new(0x52);
                a[20] = PlainDword::new(0xA4);
                a[22] = PlainDword::new(0x52);
                a[23] = PlainDword::new(0xA4);
                a[25] = PlainDword::new(0x52);
                a[26] = PlainDword::new(0xA4);
                a[28] = PlainDword::new(0x52);
                a[29] = PlainDword::new(0xA4);
                a[31] = PlainDword::new(0x52);
                a[32] = PlainDword::new(0xA4);
                a[34] = PlainDword::new(0x52);
                a[35] = PlainDword::new(0xA4);
                a
            },
            car_icon_sheet_y: {
                let mut a: [PlainDword; 36] = ZeroCells::ZERO;
                a[3] = PlainDword::new(0x2A);
                a[4] = PlainDword::new(0x2A);
                a[5] = PlainDword::new(0x2A);
                a[6] = PlainDword::new(0x54);
                a[7] = PlainDword::new(0x54);
                a[8] = PlainDword::new(0x54);
                a[9] = PlainDword::new(0x7E);
                a[10] = PlainDword::new(0x7E);
                a[11] = PlainDword::new(0x7E);
                a[12] = PlainDword::new(0xA8);
                a[13] = PlainDword::new(0xA8);
                a[14] = PlainDword::new(0xA8);
                a[15] = PlainDword::new(0xD2);
                a[16] = PlainDword::new(0xD2);
                a[17] = PlainDword::new(0xD2);
                a[21] = PlainDword::new(0x2A);
                a[22] = PlainDword::new(0x2A);
                a[23] = PlainDword::new(0x2A);
                a[24] = PlainDword::new(0x54);
                a[25] = PlainDword::new(0x54);
                a[26] = PlainDword::new(0x54);
                a[27] = PlainDword::new(0x7E);
                a[28] = PlainDword::new(0x7E);
                a[29] = PlainDword::new(0x7E);
                a[30] = PlainDword::new(0xA8);
                a[31] = PlainDword::new(0xA8);
                a[32] = PlainDword::new(0xA8);
                a[33] = PlainDword::new(0xD2);
                a[34] = PlainDword::new(0xD2);
                a[35] = PlainDword::new(0xD2);
                a
            },
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every table resolves, entry by entry, to the dwords it names.
    #[test]
    fn tables_resolve_to_their_fields() {
        let places = TexturePlaces::new();
        for table in PlacesTable::ALL {
            let cells = table.cells(&places);
            for n in [0, cells.len() - 1] {
                let found = cell_at(&places, table, n as u32).unwrap();
                assert!(core::ptr::eq(found, &raw const cells[n]), "{table:?} {n}");
            }
        }
    }

    /// A number past a table's end reads on into the following tables, to the places' end.
    #[test]
    fn numbers_run_on_through_the_tables() {
        let places = TexturePlaces::new();
        let all: Vec<&PlainDword> = PlacesTable::ALL
            .into_iter()
            .flat_map(|table| table.cells(&places))
            .collect();
        for (start, table) in PlacesTable::ALL.into_iter().enumerate() {
            let before: usize = PlacesTable::ALL[..start]
                .iter()
                .map(|t| t.cells(&places).len())
                .sum();
            for n in [0, 5000, 30_000, 40_000] {
                let found = cell_at(&places, table, n);
                match all.get(before + n as usize) {
                    Some(cell) => assert!(core::ptr::eq(found.unwrap(), *cell)),
                    None => assert!(found.is_err()),
                }
            }
        }
    }
}
