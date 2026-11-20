//! The texture places: which texture each slot shows, the pictures wanted and loaded, the counters,
//! the car icon sheets and the shapes' marks. `TexturePlaces` is plain fields only the game's thread
//! reaches (`scene::owner::Scene::texture_places`); the part the drawing parts read, `PlaceSlots`,
//! stays a lent store.

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

/// What the drawing parts read of the places: each place's slot, and how many textures the
/// video memory holds. Written between frames (`scene::map_texture_places`,
/// `scene::count_texture_needs`), read by the parts through their reads
/// (`scene::workers::DrawReads`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct PlaceSlots {
    /// Each place's slot (`scene::look_up_texture`, `scene::map_texture_places`,
    /// `scene::draw_polygon`, `race::frame_draw`). [1999] is the windscreen drops' texture
    /// (`effects::windscreen_drops_setup`).
    pub place_slot: [PlainDword; 4000],
    /// The textures in video memory (`scene::count_texture_needs`; `scene::look_up_texture`,
    /// `scene::draw_polygon`, `scene::load_slot_textures`).
    pub video_textures: PlainDword,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl PlaceSlots {
    /// The store with its first values (everything starts at zero).
    pub const fn new() -> Self {
        <Self as ZeroCells>::ZERO
    }
}

/// The places' other tables: plain fields, reached by the game's thread alone.
#[derive(Clone)]
pub struct TexturePlaces {
    /// The texture of each slot (`scene::try_allocate_textures`).
    pub texture_of: [u32; 1024],
    /// The picture number of each kind (the code reaches [0x87], [0x258] ..)
    /// (`race::set_all_plates`, `scene::reload_system_textures`,
    /// `scene::set_up_front_end_scene`).
    pub kind_pictures: [u32; 1024],
    /// The picture each texture should hold (`scene::load_changed_texture_picture`,
    /// `scene::reset_texture_places`, `scene::textures`).
    pub wanted_pictures: [u32; 1024],
    /// The picture each holds (same users).
    pub loaded_pictures: [u32; 1024],
    /// The texture each slot should show (`scene::clear_slot_textures`, `scene::textures`,
    /// `race::draw_race_scene`).
    pub slot_wanted: [u32; 1024],
    /// The texture each slot shows (`scene::forget_slot_texture`, `scene::textures`).
    pub slot_loaded: [u32; 1024],
    /// What the video memory and the system memory hold, in kilobytes, and the system
    /// memory's textures (`scene::count_texture_needs`; the video memory's textures are
    /// `PlaceSlots::video_textures`).
    pub video_kilobytes: u32,
    pub system_kilobytes: u32,
    pub system_textures: u32,
    /// What a kind's slots take (`scene::texture_levels`, `scene::reset_texture_tables`).
    pub kind_slot_bytes: u32,
    /// 0x100 counters (`scene::build_texture_catalogue`).
    pub counters: [u32; 256],
    /// The texture steps' flags (`scene::texture_steps`, `scene::build_texture_catalogue`).
    pub step_flags: [u32; 8],
    /// The car icons' sheet x (`scene::load_hud_car_icons`).
    pub car_icon_sheet_x: [u32; 36],
    /// The sheet y (`scene::load_hud_car_icons`).
    pub car_icon_sheet_y: [u32; 36],
    /// The objects' texture lists, a count then the textures (`scene::list_object_textures`).
    pub object_texture_records: [u32; 1024],
    /// The shape's faces taken (`scene::add_object_shape`).
    pub face_marks: [u32; 4096],
    /// The points' marks (`scene::add_object_shape`).
    pub point_marks: [u32; 4096],
    /// The normals' marks (`scene::add_object_shape`).
    pub normal_marks: [u32; 4096],
}

/// The sheet x of each car icon.
const CAR_ICON_SHEET_X: [u32; 36] = {
    let mut a = [0; 36];
    let mut n = 0;
    while n < 36 {
        a[n] = match n % 3 {
            1 => 0x52,
            2 => 0xA4,
            _ => 0,
        };
        n += 1;
    }
    a
};

/// The sheet y of each car icon: rows of three icons, 0x2A apart, twice (two sheets of six rows).
const CAR_ICON_SHEET_Y: [u32; 36] = {
    let mut a = [0; 36];
    let mut n = 0;
    while n < 36 {
        a[n] = ((n % 18) / 3) as u32 * 0x2A;
        n += 1;
    }
    a
};

impl TexturePlaces {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        Self {
            texture_of: [0; 1024],
            kind_pictures: [0; 1024],
            wanted_pictures: [0; 1024],
            loaded_pictures: [0; 1024],
            slot_wanted: [0; 1024],
            slot_loaded: [0; 1024],
            video_kilobytes: 0,
            system_kilobytes: 0,
            system_textures: 0,
            kind_slot_bytes: 0,
            counters: [0; 256],
            step_flags: [0; 8],
            car_icon_sheet_x: CAR_ICON_SHEET_X,
            car_icon_sheet_y: CAR_ICON_SHEET_Y,
            object_texture_records: [0; 1024],
            face_marks: [0; 4096],
            point_marks: [0; 4096],
            normal_marks: [0; 4096],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The icon sheet's first values: the columns 0, 0x52, 0xA4 and the rows 0x2A apart.
    #[test]
    fn the_icon_sheet_starts_as_the_game_has_it() {
        let places = TexturePlaces::new();
        assert_eq!(
            &places.car_icon_sheet_x[..6],
            &[0, 0x52, 0xA4, 0, 0x52, 0xA4]
        );
        assert_eq!(places.car_icon_sheet_x[35], 0xA4);
        assert_eq!(places.car_icon_sheet_y[2], 0);
        assert_eq!(places.car_icon_sheet_y[3], 0x2A);
        assert_eq!(places.car_icon_sheet_y[17], 0xD2);
        assert_eq!(places.car_icon_sheet_y[18], 0);
        assert_eq!(places.car_icon_sheet_y[21], 0x2A);
        assert_eq!(places.car_icon_sheet_y[35], 0xD2);
    }
}
