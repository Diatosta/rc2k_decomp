//! The menus' lists and tables, out of RAL.EXE's image (IDA 0x7594AA .. 0x75A658; phase 7,
//! typed): the last draw list, the carousel kinds' own places, the paths and sounds the menus
//! name, the car lists the menus walk, the best scores' titles. `#[repr(C)]` struct of atomic
//! cells with the image's layout, one static; `moved::MOVED` points the range at it. Its
//! last field is a 22-byte string: the struct is padded to a multiple of 4.

use crate::menu::cars_store::car_cell;
use crate::session::{Bytes, Dword};

/// A gap's first bytes: `runs` are (offset, bytes) of the non-zero ones.
const fn sparse<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// One kind's own place in a menu scene (`scene::draw_carousel_entry`: the
/// kind's size, its y and its angle).
#[repr(C)]
pub struct OwnPlace {
    /// Its size.
    pub size: Dword,
    /// Its y.
    pub y: Dword,
    /// Its angle about x.
    pub angle: Dword,
}

/// The menus' lists and tables.
#[repr(C)]
pub struct MenuLists {
    /// `dword_7594AA`: the menus' last draw list (25 entries, -1 at start;
    /// `menu::player4_buttons_loop`'s).
    pub draw_list: [Dword; 25],
    /// `unk_75950E`: the carousel kinds' own places, 12 bytes each, indexed by a kind's
    /// number (`scene::draw_carousel_entry` checks it against 0x88 records, a slip: the table
    /// holds 73 and the lists that follow answer the rest).
    pub own_places: [OwnPlace; 73],
    /// Unreferenced bytes (IDA 0X75987A .. 0X75987E).
    pub _gap_75987a: Bytes<4>,
    /// `off_75987E`: the pictures' paths the stage view's welcome shows, a championship's.
    pub welcome_championship: [Dword; 6],
    /// `off_759896`: the same for the other modes (the A8 game).
    pub welcome_other: [Dword; 6],
    /// `off_7598AE`: the map's overlays, by the stage's +0x3C (`menu::stage_choice`).
    pub map_overlays: [Dword; 36],
    /// Unreferenced bytes (IDA 0X75993E .. 0X759942).
    pub _gap_75993e: Bytes<4>,
    /// `off_759942`: the first of the four lists of paths a car page cycles through
    /// (`menu::load_car_background`), ending in a gap dword.
    pub page_pictures_a: [Dword; 25],
    /// `off_7599A6`: the second of the four lists.
    pub page_pictures_b: [Dword; 25],
    /// `off_759A0A`: the third of the four lists.
    pub page_pictures_c: [Dword; 25],
    /// `off_759A6E`: the fourth of the four lists.
    pub page_pictures_d: [Dword; 25],
    /// `off_759AD2`: the addresses of the four lists above; read by nothing now (`page_pictures`).
    pub page_picture_lists: [Dword; 4],
    /// Unreferenced bytes (IDA 0X759AE2 .. 0X759AE6).
    pub _gap_759ae2: Bytes<4>,
    /// `funcs_47CED1`: four code addresses (0x469C40 ..) of the car pages. Not used by the
    /// standalone.
    pub page_functions: [Dword; 4],
    /// Unreferenced bytes (IDA 0X759AF6 .. 0X759AFA).
    pub _gap_759af6: Bytes<4>,
    /// `dword_759AFA`: the car pages' title strings, ended by -1 (`screens::car_page_title`).
    pub car_page_titles: [Dword; 5],
    /// `off_759B0E`: the slideshow pictures' path templates (`menu::load_slideshow_picture`).
    pub slideshow_templates: [Dword; 24],
    /// `dword_759B6E`: the service boxes' flags, the first set, by item value.
    pub inspection_flags_a: [Dword; 21],
    /// `dword_759BC2`: the service boxes' flags, the second set, by item value.
    pub inspection_flags_b: [Dword; 21],
    /// `unk_759C16`: pointers to each repair item's time, by item (`service::state`).
    pub repair_time_ptrs: [Dword; 24],
    /// `dword_759C76`: the inspection's fault flags, 13 items.
    pub inspection_faults: [Dword; 14],
    /// `g_championshipCars`: the championship's car list (`menu::event_flow`).
    pub cars_championship: [Dword; 26],
    /// `g_a8Cars`: the A8 game's car list.
    pub cars_a8: [Dword; 45],
    /// `off_759DCA`: the car list without the Clio, the first.
    pub cars_first: [Dword; 34],
    /// `off_759E52`: the car list with the Clio and the extra cars.
    pub cars_with_clio: [Dword; 62],
    /// `g_secretCars`: the cars won.
    pub secret_cars: [Dword; 12],
    /// `unk_759F7A`: the scratch list `menu::remove_secret_cars` builds.
    pub cars_scratch: [Dword; 50],
    /// `off_75A042`: the second car list (arcade events).
    pub cars_second: [Dword; 59],
    /// `off_75A12E`: the list of the cars shown (`menu::arcade_steps`).
    pub shown_cars: [Dword; 48],
    /// Unreferenced bytes (IDA 0X75A1EE .. 0X75A1FE).
    pub _gap_75a1ee: Bytes<16>,
    /// `dword_75A1FE`: a kind by car number (`scene::set_up_carousel`).
    pub car_kinds: [Dword; 35],
    /// `dword_75A28A`: the best scores' section title string ids, by section.
    pub section_titles: [Dword; 7],
    /// `dword_75A2A6`: the best scores' page title string ids, by page.
    pub page_titles_a: [Dword; 12],
    /// `dword_75A2D6`: the best scores' second page title string ids, by page.
    pub page_titles_b: [Dword; 11],
    /// `off_75A302`: the rally speeches' sounds, by section (`menu::stage_view_steps`).
    pub rally_speeches: [Dword; 6],
    /// Unreferenced bytes (IDA 0X75A31A .. 0X75A31E).
    pub _gap_75a31a: Bytes<4>,
    /// `off_75A31E`: the car streams' file names (`sound::play_car_stream`).
    pub car_streams: [Dword; 24],
    /// `off_75A37E`: the stage speeches' sounds (`menu::stage_view_steps`).
    pub stage_speeches: [Dword; 36],
    /// Unreferenced bytes (IDA 0X75A40E .. 0X75A412).
    pub _gap_75a40e: Bytes<4>,
    /// `off_75A412`: the cheat car list (`menu::cheat_car_list`).
    pub cheat_cars: [Dword; 22],
    /// Unreferenced bytes (IDA 0X75A46A .. 0X75A49A).
    pub _gap_75a46a: Bytes<48>,
    /// `dword_75A49A`: the stage panel's durations in minutes, by entry +8
    /// (`screens::stage_panel`; the table runs on over unreferenced bytes).
    pub durations: [Dword; 37],
    /// `g_servicePictures`: the service pictures' paths (`service::load_background`).
    pub service_pictures: [Dword; 60],
    /// Unreferenced bytes (IDA 0X75A61E .. 0X75A622).
    pub _gap_75a61e: Bytes<4>,
    /// `dword_75A622`: the eight texture numbers of the carousel's menu (-1 none).
    pub carousel_textures: [Dword; 8],
    /// `aFilesGFrntBack`: a path string, ends at 0x75A658.
    pub front_back_path: Bytes<22>,
}

/// First values the image's.
/// A text of `texts_store` by its place there (phase 13, handles; the original kept its
/// address). 0 stays none, -1 stays the end mark.
const fn text_ref(ida: u32) -> Dword {
    Dword::new(crate::menu::texts_store::text_key(ida))
}

/// A sample's name as `audio::store::name_key` has it.
const fn name_ref(ida: u32) -> Dword {
    Dword::new(crate::audio::store::name_key(ida))
}

/// A repair time's number + 1 in `MENU_STATE`'s repair times (the original kept its address;
/// 0: none; -1 the end mark).
const fn repair_key(ida: u32) -> u32 {
    match ida {
        0 | 0xFFFF_FFFF => ida,
        _ => {
            assert!(REPAIR_TIMES_IDA <= ida && ida < REPAIR_TIMES_IDA + 16 * 4);
            (ida - REPAIR_TIMES_IDA) / 4 + 1
        }
    }
}

/// IDA's address of `MENU_STATE`'s repair times.
const REPAIR_TIMES_IDA: u32 = 0x75AB60;

const fn repair_ref(ida: u32) -> Dword {
    Dword::new(repair_key(ida))
}

/// The car pages' pictures by page (the original's `page_picture_lists`, in its order; a car
/// past a list reads on into the next, as there).
pub fn page_pictures(page: u32) -> Result<&'static [Dword], crate::errors::BadInput> {
    Ok(match page {
        0 => &MENU_LISTS.page_pictures_a,
        1 => &MENU_LISTS.page_pictures_c,
        2 => &MENU_LISTS.page_pictures_d,
        3 => &MENU_LISTS.page_pictures_b,
        _ => return Err(crate::errors::BadInput("a car page past the four")),
    })
}

/// An item's repair time cell (`repair_time_ptrs`; None: none).
pub fn repair_time_at(key: u32) -> Result<Option<&'static Dword>, crate::errors::BadInput> {
    match key {
        0 => Ok(None),
        _ => crate::menu::state_store::MENU_STATE
            .driver_stats
            .repair_times
            .get(key.wrapping_sub(1) as usize)
            .map(Some)
            .ok_or(crate::errors::BadInput(
                "a repair time's handle off the times",
            )),
    }
}

pub static MENU_LISTS: MenuLists = MenuLists {
    draw_list: [const { Dword::new(0xffffffff) }; 25],
    own_places: [
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xd),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xd),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x14),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x11),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x11),
            y: Dword::new(0),
            angle: Dword::new(0x5ff),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x13),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x13),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x11),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0x3ff),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0x3ff),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0x3ff),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0x3ff),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x10),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(0xc),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0x12),
            y: Dword::new(0),
            angle: Dword::new(0),
        },
        OwnPlace {
            size: Dword::new(0xe),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
        OwnPlace {
            size: Dword::new(5),
            y: Dword::new(0),
            angle: Dword::new(0xcf0),
        },
    ],
    _gap_75987a: Bytes::new([0xff; 4]),
    welcome_championship: [
        text_ref(0x7326fd),
        text_ref(0x732720),
        text_ref(0x732744),
        text_ref(0x732768),
        text_ref(0x73278b),
        text_ref(0x7327ae),
    ],
    welcome_other: [
        text_ref(0x7327d2),
        text_ref(0x7327f4),
        text_ref(0x732816),
        text_ref(0x732838),
        text_ref(0x73285a),
        text_ref(0x73287c),
    ],
    map_overlays: [
        text_ref(0x733018),
        text_ref(0x733042),
        text_ref(0x73306c),
        text_ref(0x733096),
        text_ref(0x7330c0),
        text_ref(0x7330ea),
        text_ref(0x733114),
        text_ref(0x73313e),
        text_ref(0x733168),
        text_ref(0x733192),
        text_ref(0x7331bc),
        text_ref(0x7331e6),
        text_ref(0x733210),
        text_ref(0x733239),
        text_ref(0x733262),
        text_ref(0x73328b),
        text_ref(0x7332b4),
        text_ref(0x7332dd),
        text_ref(0x733306),
        text_ref(0x73332f),
        text_ref(0x733358),
        text_ref(0x733381),
        text_ref(0x7333aa),
        text_ref(0x7333d3),
        text_ref(0x7333fc),
        text_ref(0x733425),
        text_ref(0x73344e),
        text_ref(0x733477),
        text_ref(0x7334a0),
        text_ref(0x7334c9),
        text_ref(0x7334f2),
        text_ref(0x73351c),
        text_ref(0x733546),
        text_ref(0x733570),
        text_ref(0x73359a),
        text_ref(0x7335c4),
    ],
    _gap_75993e: Bytes::new([0xff; 4]),
    page_pictures_a: [
        text_ref(0x7335ee),
        text_ref(0x733618),
        text_ref(0x733642),
        text_ref(0x73366c),
        text_ref(0x733696),
        text_ref(0x7336c0),
        text_ref(0x7336ea),
        text_ref(0x733714),
        text_ref(0x73373e),
        text_ref(0x733768),
        text_ref(0x733792),
        text_ref(0x7337bc),
        text_ref(0x7337e6),
        text_ref(0x733810),
        text_ref(0x73383a),
        text_ref(0x733864),
        text_ref(0x73388e),
        text_ref(0x7338b8),
        text_ref(0x7338e2),
        text_ref(0x73390c),
        text_ref(0x733936),
        text_ref(0x733960),
        text_ref(0x73398a),
        text_ref(0x7339b4),
        text_ref(0xffffffff),
    ],
    page_pictures_b: [
        text_ref(0x7339de),
        text_ref(0x733a08),
        text_ref(0x733a32),
        text_ref(0x733a5c),
        text_ref(0x733a86),
        text_ref(0x733ab0),
        text_ref(0x733ada),
        text_ref(0x733b04),
        text_ref(0x733b2e),
        text_ref(0x733b58),
        text_ref(0x733b82),
        text_ref(0x733bac),
        text_ref(0x733bd6),
        text_ref(0x733c00),
        text_ref(0x733c2a),
        text_ref(0x733c54),
        text_ref(0x733c7e),
        text_ref(0x733ca8),
        text_ref(0x733cd2),
        text_ref(0x733cfc),
        text_ref(0x733d26),
        text_ref(0x733d50),
        text_ref(0x733d7a),
        text_ref(0x733da4),
        text_ref(0xffffffff),
    ],
    page_pictures_c: [
        text_ref(0x733dce),
        text_ref(0x733df8),
        text_ref(0x733e22),
        text_ref(0x733e4c),
        text_ref(0x733e76),
        text_ref(0x733ea0),
        text_ref(0x733eca),
        text_ref(0x733ef4),
        text_ref(0x733f1e),
        text_ref(0x733f48),
        text_ref(0x733f72),
        text_ref(0x733f9c),
        text_ref(0x733fc6),
        text_ref(0x733ff0),
        text_ref(0x73401a),
        text_ref(0x734044),
        text_ref(0x73406e),
        text_ref(0x734098),
        text_ref(0x7340c2),
        text_ref(0x7340ec),
        text_ref(0x734116),
        text_ref(0x734140),
        text_ref(0x73416a),
        text_ref(0x734194),
        text_ref(0xffffffff),
    ],
    page_pictures_d: [
        text_ref(0x7341be),
        text_ref(0x7341e8),
        text_ref(0x734212),
        text_ref(0x73423c),
        text_ref(0x734266),
        text_ref(0x734290),
        text_ref(0x7342ba),
        text_ref(0x7342e4),
        text_ref(0x73430e),
        text_ref(0x734338),
        text_ref(0x734362),
        text_ref(0x73438c),
        text_ref(0x7343b6),
        text_ref(0x7343e0),
        text_ref(0x73440a),
        text_ref(0x734434),
        text_ref(0x73445e),
        text_ref(0x734488),
        text_ref(0x7344b2),
        text_ref(0x7344dc),
        text_ref(0x734506),
        text_ref(0x734530),
        text_ref(0x73455a),
        text_ref(0x734584),
        text_ref(0xffffffff),
    ],
    page_picture_lists: [
        Dword::new(0x759942),
        Dword::new(0x759a0a),
        Dword::new(0x759a6e),
        Dword::new(0x7599a6),
    ],
    _gap_759ae2: Bytes::new([0xff; 4]),
    page_functions: [
        Dword::new(0x469c40),
        Dword::new(0x469f12),
        Dword::new(0x46a193),
        Dword::new(0x469ca0),
    ],
    _gap_759af6: Bytes::new([0xff; 4]),
    car_page_titles: [
        Dword::new(0x24c),
        Dword::new(0x249),
        Dword::new(0x24b),
        Dword::new(0x24a),
        Dword::new(0xffffffff),
    ],
    slideshow_templates: [
        text_ref(0x7345ae),
        text_ref(0x7345cb),
        text_ref(0x7345e8),
        text_ref(0x734605),
        text_ref(0x734622),
        text_ref(0x73463f),
        text_ref(0x73465c),
        text_ref(0x734679),
        text_ref(0x734696),
        text_ref(0x7346b3),
        text_ref(0x7346d0),
        text_ref(0x7346ed),
        text_ref(0x73470a),
        text_ref(0x734727),
        text_ref(0x734744),
        text_ref(0x734761),
        text_ref(0x73477e),
        text_ref(0x73479b),
        text_ref(0x7347b8),
        text_ref(0x7347d5),
        text_ref(0x7347f2),
        text_ref(0x73480f),
        text_ref(0x73482c),
        text_ref(0x734849),
    ],
    inspection_flags_a: [
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0xffffffff),
    ],
    inspection_flags_b: [
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0xffffffff),
    ],
    repair_time_ptrs: [
        repair_ref(0),
        repair_ref(0),
        repair_ref(0x75ab60),
        repair_ref(0x75ab64),
        repair_ref(0x75ab68),
        repair_ref(0x75ab6c),
        repair_ref(0x75ab70),
        repair_ref(0x75ab74),
        repair_ref(0x75ab78),
        repair_ref(0x75ab7c),
        repair_ref(0x75ab80),
        repair_ref(0x75ab84),
        repair_ref(0x75ab88),
        repair_ref(0x75ab8c),
        repair_ref(0x75ab90),
        repair_ref(0x75ab94),
        repair_ref(0),
        repair_ref(0),
        repair_ref(0x75ab98),
        repair_ref(0x75ab9c),
        repair_ref(0),
        repair_ref(0),
        repair_ref(0),
        repair_ref(0xffffffff),
    ],
    inspection_faults: [
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0),
        Dword::new(0xffffffff),
    ],
    cars_championship: [
        car_cell(0x756d60),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756b08),
        car_cell(0x756a90),
        car_cell(0x7567c0),
        car_cell(0x7569a0),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756ec8),
        car_cell(0x7565e0),
        car_cell(0x7566d0),
        car_cell(0x756928),
        car_cell(0x756568),
        car_cell(0x756b80),
        car_cell(0x756f40),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    cars_a8: [
        car_cell(0x756bf8),
        car_cell(0x756748),
        car_cell(0x7568b0),
        car_cell(0x756658),
        car_cell(0x756a18),
        car_cell(0x756fb8),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0x756d60),
        car_cell(0x7568b0),
        car_cell(0xffffffff),
        car_cell(0x756568),
        car_cell(0x7565e0),
        car_cell(0x756658),
        car_cell(0x7566d0),
        car_cell(0x756748),
        car_cell(0x7567c0),
        car_cell(0x756838),
        car_cell(0x7568b0),
        car_cell(0x756928),
        car_cell(0x7569a0),
        car_cell(0x756a18),
        car_cell(0x756a90),
        car_cell(0x756b08),
        car_cell(0x756b80),
        car_cell(0x756bf8),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756d60),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756ec8),
        car_cell(0x756f40),
        car_cell(0x756fb8),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    cars_first: [
        car_cell(0x756b80),
        car_cell(0x756f40),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756b08),
        car_cell(0x756a90),
        car_cell(0x7567c0),
        car_cell(0x7569a0),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756ec8),
        car_cell(0x7565e0),
        car_cell(0x7566d0),
        car_cell(0x756928),
        car_cell(0x756568),
        car_cell(0x756d60),
        car_cell(0x756fb8),
        car_cell(0x756a18),
        car_cell(0x756658),
        car_cell(0x7568b0),
        car_cell(0x756748),
        car_cell(0x756bf8),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    cars_with_clio: [
        car_cell(0x756b80),
        car_cell(0x756f40),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756b08),
        car_cell(0x756a90),
        car_cell(0x7567c0),
        car_cell(0x7569a0),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756ec8),
        car_cell(0x7565e0),
        car_cell(0x7566d0),
        car_cell(0x756928),
        car_cell(0x756568),
        car_cell(0x756d60),
        car_cell(0x756fb8),
        car_cell(0x756a18),
        car_cell(0x756658),
        car_cell(0x7568b0),
        car_cell(0x756748),
        car_cell(0x756bf8),
        car_cell(0x757030),
        car_cell(0x7570a8),
        car_cell(0x757120),
        car_cell(0x757198),
        car_cell(0x757210),
        car_cell(0x757288),
        car_cell(0x757300),
        car_cell(0x7573f0),
        car_cell(0x757468),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    secret_cars: [
        car_cell(0x757030),
        car_cell(0x7570a8),
        car_cell(0x757120),
        car_cell(0x757198),
        car_cell(0x757210),
        car_cell(0x757288),
        car_cell(0x757300),
        car_cell(0x757378),
        car_cell(0x7573f0),
        car_cell(0x757468),
        car_cell(0x756838),
        car_cell(0xffffffff),
    ],
    cars_scratch: [const { car_cell(0xffffffff) }; 50],
    cars_second: [
        car_cell(0x756b80),
        car_cell(0x756f40),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756b08),
        car_cell(0x756a90),
        car_cell(0x7567c0),
        car_cell(0x7569a0),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756ec8),
        car_cell(0x7565e0),
        car_cell(0x7566d0),
        car_cell(0x756928),
        car_cell(0x756568),
        car_cell(0x756d60),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    shown_cars: [
        car_cell(0x756658),
        car_cell(0x756748),
        car_cell(0x7568b0),
        car_cell(0x756a18),
        car_cell(0x756bf8),
        car_cell(0x756fb8),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
        car_cell(0xffffffff),
    ],
    _gap_75a1ee: Bytes::new([0xff; 16]),
    car_kinds: [
        Dword::new(0x191),
        Dword::new(0x193),
        Dword::new(0x195),
        Dword::new(0x197),
        Dword::new(0x199),
        Dword::new(0x19b),
        Dword::new(0x1c3),
        Dword::new(0x19f),
        Dword::new(0x1a1),
        Dword::new(0x1a3),
        Dword::new(0x1a5),
        Dword::new(0x1a7),
        Dword::new(0x1a9),
        Dword::new(0x1ab),
        Dword::new(0x1ad),
        Dword::new(0x1af),
        Dword::new(0x1b1),
        Dword::new(0x1b3),
        Dword::new(0x1b5),
        Dword::new(0x1b7),
        Dword::new(0x1b9),
        Dword::new(0x1bb),
        Dword::new(0x1bd),
        Dword::new(0x1bf),
        Dword::new(0x19d),
        Dword::new(0x1c1),
        Dword::new(0x1c5),
        Dword::new(0x1c7),
        Dword::new(0x1c9),
        Dword::new(0x1cb),
        Dword::new(0x1cd),
        Dword::new(0x1cf),
        Dword::new(0x1d1),
        Dword::new(0x23a),
        Dword::new(0xffffffff),
    ],
    section_titles: [
        Dword::new(0x3d6),
        Dword::new(0x3d7),
        Dword::new(0x3d8),
        Dword::new(0x3d9),
        Dword::new(0x3da),
        Dword::new(0x3db),
        Dword::new(0xffffffff),
    ],
    page_titles_a: [
        Dword::new(0x361),
        Dword::new(0x360),
        Dword::new(0x35f),
        Dword::new(0x362),
        Dword::new(0x35e),
        Dword::new(0x363),
        Dword::new(0x364),
        Dword::new(0x365),
        Dword::new(0x366),
        Dword::new(0x367),
        Dword::new(0x368),
        Dword::new(0xffffffff),
    ],
    page_titles_b: [
        Dword::new(3),
        Dword::new(2),
        Dword::new(1),
        Dword::new(4),
        Dword::new(0),
        Dword::new(0),
        Dword::new(1),
        Dword::new(2),
        Dword::new(3),
        Dword::new(4),
        Dword::new(5),
    ],
    rally_speeches: [
        name_ref(0x5096e3),
        name_ref(0x509704),
        name_ref(0x509725),
        name_ref(0x509746),
        name_ref(0x509767),
        name_ref(0x509788),
    ],
    _gap_75a31a: Bytes::new([0xff; 4]),
    car_streams: [
        text_ref(0x734866),
        text_ref(0x734872),
        text_ref(0x73487e),
        text_ref(0x73488a),
        text_ref(0x734896),
        text_ref(0x7348a2),
        text_ref(0x7348ae),
        text_ref(0x7348ba),
        text_ref(0x7348c6),
        text_ref(0x7348d2),
        text_ref(0x7348de),
        text_ref(0x7348ea),
        text_ref(0x7348f6),
        text_ref(0x734902),
        text_ref(0x73490e),
        text_ref(0x73491a),
        text_ref(0x734926),
        text_ref(0x734932),
        text_ref(0x73493e),
        text_ref(0x73494a),
        text_ref(0x734956),
        text_ref(0x734962),
        text_ref(0x73496e),
        text_ref(0x73497a),
    ],
    stage_speeches: [
        text_ref(0x734986),
        text_ref(0x734996),
        text_ref(0x7349a6),
        text_ref(0x7349b6),
        text_ref(0x7349c6),
        text_ref(0x7349d6),
        text_ref(0x7349e6),
        text_ref(0x7349f6),
        text_ref(0x734a06),
        text_ref(0x734a16),
        text_ref(0x734a26),
        text_ref(0x734a36),
        text_ref(0x734a46),
        text_ref(0x734a56),
        text_ref(0x734a66),
        text_ref(0x734a76),
        text_ref(0x734a86),
        text_ref(0x734a96),
        text_ref(0x734aa6),
        text_ref(0x734ab6),
        text_ref(0x734ac6),
        text_ref(0x734ad6),
        text_ref(0x734ae6),
        text_ref(0x734af6),
        text_ref(0x734b06),
        text_ref(0x734b16),
        text_ref(0x734b26),
        text_ref(0x734b36),
        text_ref(0x734b46),
        text_ref(0x734b56),
        text_ref(0x734b66),
        text_ref(0x734b76),
        text_ref(0x734b86),
        text_ref(0x734b96),
        text_ref(0x734ba6),
        text_ref(0x734bb6),
    ],
    _gap_75a40e: Bytes::new([0xff; 4]),
    cheat_cars: [
        car_cell(0x756568),
        car_cell(0x7565e0),
        car_cell(0x7566d0),
        car_cell(0x756928),
        car_cell(0x7569a0),
        car_cell(0x756c70),
        car_cell(0x756ce8),
        car_cell(0x756d60),
        car_cell(0x7567c0),
        car_cell(0x756a90),
        car_cell(0x756b08),
        car_cell(0x756b80),
        car_cell(0x756dd8),
        car_cell(0x756e50),
        car_cell(0x756658),
        car_cell(0x756748),
        car_cell(0x7568b0),
        car_cell(0x756a18),
        car_cell(0x756bf8),
        car_cell(0x756ec8),
        car_cell(0x756f40),
        car_cell(0x756fb8),
    ],
    _gap_75a46a: Bytes::new([0xff; 48]),
    durations: [
        Dword::new(0x11),
        Dword::new(0xe),
        Dword::new(0x14),
        Dword::new(0x15),
        Dword::new(0x17),
        Dword::new(0x14),
        Dword::new(0x1f),
        Dword::new(0x1b),
        Dword::new(0x17),
        Dword::new(0x1e),
        Dword::new(0xd),
        Dword::new(0x19),
        Dword::new(0xd),
        Dword::new(8),
        Dword::new(8),
        Dword::new(0xa),
        Dword::new(0x1a),
        Dword::new(0x1d),
        Dword::new(5),
        Dword::new(9),
        Dword::new(8),
        Dword::new(9),
        Dword::new(3),
        Dword::new(9),
        Dword::new(0xe),
        Dword::new(0x11),
        Dword::new(0xf),
        Dword::new(0x11),
        Dword::new(0xd),
        Dword::new(0xd),
        Dword::new(7),
        Dword::new(0xb),
        Dword::new(0xf),
        Dword::new(6),
        Dword::new(0xe),
        Dword::new(0x10),
        Dword::new(0xffffffff),
    ],
    service_pictures: [
        text_ref(0x732cb8),
        text_ref(0x732cdc),
        text_ref(0x732d00),
        text_ref(0x732d24),
        text_ref(0x732d48),
        text_ref(0x732d6c),
        text_ref(0x732d90),
        text_ref(0x732db4),
        text_ref(0x732dd8),
        text_ref(0x732dfc),
        text_ref(0x732e20),
        text_ref(0x732e44),
        text_ref(0x732e68),
        text_ref(0x732e8c),
        text_ref(0x732eb0),
        text_ref(0x732ed4),
        text_ref(0x732ef8),
        text_ref(0x732f1c),
        text_ref(0x732f40),
        text_ref(0x732f64),
        text_ref(0x732f88),
        text_ref(0x732fac),
        text_ref(0x732fd0),
        text_ref(0x732ff4),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
        text_ref(0x732fd0),
    ],
    _gap_75a61e: Bytes::new([0xff; 4]),
    carousel_textures: [const { Dword::new(0) }; 8],
    front_back_path: Bytes::new(sparse::<22>(&[(
        0x0,
        &[
            0x66, 0x69, 0x6c, 0x65, 0x73, 0x5c, 0x67, 0x5c, 0x66, 0x72, 0x6e, 0x74, 0x5c, 0x62,
            0x61, 0x63, 0x6b, 0x2e, 0x70, 0x63, 0x78,
        ],
    )])),
};

/// IDA 0X7594AA, the first byte of the piece.
pub const START: usize = 0x7594AA;
/// IDA 0X75A658, the end of the piece.
pub const END: usize = 0x75A658;

/// Where IDA 0X7594AA is now (`moved::MOVED`'s home).
pub fn menu_lists_addr() -> u32 {
    crate::pointers::game_address(&raw const MENU_LISTS)
}

/// The tables of text handles (IDA start, entries).
#[cfg(test)]
const TEXT_TABLES: [(usize, usize); 6] = [
    (0x75987E, 12 + 36), // the welcome pictures, the map's overlays
    (0x759942, 4 * 25),  // the car pages' pictures
    (0x759B0E, 24),      // the slideshow's templates
    (0x75A31E, 24),      // the car streams
    (0x75A37E, 36),      // the stage speeches
    (0x75A52E, 60),      // the service pictures
];
/// The rally speeches (IDA start, entries).
#[cfg(test)]
const NAME_TABLE: (usize, usize) = (0x75A302, 6);
/// The repair times' handles (IDA start, entries).
#[cfg(test)]
const REPAIR_TABLE: (usize, usize) = (0x759C16, 24);

/// The car lists (IDA's address, entries): their entries are car handles
/// (`cars_store::car_ref`) where the original kept descriptions' addresses.
#[cfg(test)]
const CAR_LISTS: [(usize, usize); 9] = {
    use core::mem::offset_of;
    [
        (START + offset_of!(MenuLists, cars_championship), 26),
        (START + offset_of!(MenuLists, cars_a8), 45),
        (START + offset_of!(MenuLists, cars_first), 34),
        (START + offset_of!(MenuLists, cars_with_clio), 62),
        (START + offset_of!(MenuLists, secret_cars), 12),
        (START + offset_of!(MenuLists, cars_scratch), 50),
        (START + offset_of!(MenuLists, cars_second), 59),
        (START + offset_of!(MenuLists, shown_cars), 48),
        (START + offset_of!(MenuLists, cheat_cars), 22),
    ]
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn menu_lists_starts_as_the_games() {
        let size = size_of::<MenuLists>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts((&raw const MENU_LISTS).cast::<u8>(), END - START)
        };
        // The car lists' entries are handles: the image's addresses made handles to compare.
        let mut image = image_bytes(START, END - START);
        for (at, len) in CAR_LISTS {
            for n in 0..len {
                let o = at - START + 4 * n;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4].copy_from_slice(&crate::menu::cars_store::car_ref(v).to_le_bytes());
            }
        }
        // So are the texts, names and repair times.
        let mut made = |(at, len): (usize, usize), key: &dyn Fn(u32) -> u32| {
            for n in 0..len {
                let o = at - START + 4 * n;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4].copy_from_slice(&key(v).to_le_bytes());
            }
        };
        for t in TEXT_TABLES {
            made(t, &crate::menu::texts_store::text_key);
        }
        made(NAME_TABLE, &crate::audio::store::name_key);
        made(REPAIR_TABLE, &repair_key);
        assert_eq!(mine, &image[..]);
        assert_eq!(offset_of!(MenuLists, secret_cars), 0x759F4A - START);
        assert_eq!(offset_of!(MenuLists, carousel_textures), 0x75A622 - START);
    }
}
