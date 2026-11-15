//! The polygon pipeline's working values, a view's own: the polygon being set up, clipped,
//! projected and drawn (its colour, textures, sides, vertex count, entry and record numbers, clip
//! planes and flags, the clip's work runs), the texture page, the object face's word and shine, and
//! the sort key. One per view slot (`race::view_scratch::ViewScratch::polygon`); the HUD, the
//! loading bar and the menus use the front end's. Not here: `LevelGrid`'s `smooth`, `plain_first`
//! and `plain_second` (the HUD sets the plain switches and leaves them set into the next frame's
//! views until their first face resets them) and `polygon_alpha` (set by the HUD's strips only,
//! read by every view's drawing).

use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells)]
pub struct PolygonWork {
    /// The next free record of the pool.
    pub polygon_next_record: PlainDword,
    /// The polygon's two texture words (`scene::draw_polygon`).
    pub polygon_textures: [PlainDword; 2],
    /// The texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch).
    pub texture_page: PlainDword,
    /// The polygon's colour.
    pub polygon_colour: PlainDword,
    /// The polygon's texture.
    pub polygon_texture: PlainDword,
    /// The face's shine (`scene::draw_object_face`).
    pub face_shine: PlainDword,
    /// A face's word (`scene::draw_object_face`).
    pub face_word: PlainDword,
    /// The polygon's two side switches (`scene::draw_polygon`, the quads).
    pub polygon_sides: [PlainDword; 2],
    /// Kept by `scene::draw_polygon`.
    pub polygon_kept: [PlainDword; 3],
    /// The polygon's count of points outside.
    pub polygon_outside_count: PlainDword,
    /// The polygon's vertex count.
    pub polygon_vertex_count: PlainDword,
    /// The next free entry of the work table.
    pub polygon_next_entry: PlainDword,
    /// The polygon's clip mask.
    pub polygon_clip_mask: PlainDword,
    /// The clip's plane (`scene::clip`).
    pub clip_plane: PlainDword,
    /// Clipped by the screen ("donesuth").
    pub screen_clipped: PlainDword,
    /// Clipped by depth ("donezcut").
    pub depth_clipped: PlainDword,
    /// A list entry's third word (`scene::list_object_face`).
    pub entry_third: PlainDword,
    /// Whether the polygon was drawn.
    pub polygon_drawn: PlainDword,
    /// The face's sort key (`scene::face_sort_key`).
    pub face_sort_key: PlainDword,
    /// The clip's kept mask (`scene::clip`).
    pub clip_kept_mask: PlainDword,
    /// `scene::clip`'s work; `race::car_shadow` keeps a vertex's pieces in [2], [4], [10],
    /// [11].
    pub clip_work: [PlainFloat; 12],
    /// `scene::clip`'s second output run (five words written back from its work);
    /// `race::car_shadow` keeps the new crossing vertex's fog byte (+0x20) in [0].
    pub clip_out_words: [PlainDword; 5],
    /// The four quad vertices (8 dwords each; `scene::point_records::of_quad`): the screen
    /// quads', the halfway vertices' and the skidmarks' own. Every writer sets dwords 3, 6, 7
    /// of the vertices it names just before the polygon is clipped and drawn.
    pub quad_vertices: [PlainDword; 32],
    /// The texture filter's switches (`LevelGrid.smooth` .. `LevelGrid.plain_second`): `smooth`
    /// the pass being drawn's (render state 3: 3 smooth, 1 not), set from the pass's set's
    /// `plain`, 1 when its texture's mapping is 0. A view starts from
    /// `race::view_scratch::filter_input`; the HUD sets the front end's slot's `plain` 1, 1
    /// (`hud::overlay_pass`).
    pub smooth: PlainDword,
    pub plain_first: PlainDword,
    pub plain_second: PlainDword,
    /// The clear's rectangle (left, top, right, bottom), each view's own.
    pub cleared_rectangle: [PlainDword; 4],
    /// The amount (alpha) byte of the vertex a clip crossing makes, blended between its two
    /// ends, as a dword (`race::car_shadow`), each view's own.
    pub crossing_amount: PlainDword,
    /// The colour the faded polygons fade from (`Gfx::d3d.cards.fade_from`, the fog colour
    /// `scene::set_fog_colour` was given), each view's own.
    pub fade_from: PlainDword,
    /// `GroundWork.view_mirrored`: 1 while the rear-view mirror is drawn (set by
    /// `race::mirror_view_window`; a main view's stays 0): its polygons' vertices turned
    /// round, its car lights and turn the mirror's.
    pub view_mirrored: PlainDword,
    /// The view's sun (`race::place_sun`; the hit test covers it).
    pub sun: ViewSun,
    /// The view's window, scale and depths (`scene::view_window`): set before the view's
    /// scene begins, so `ViewScratch::begin` keeps it.
    pub window: crate::scene::view_window::ViewWindow,
}

/// A view's sun: on the screen and not covered (`race::place_sun` sets it, the polygons' hit test,
/// `scene::draw_polygon`, clears it), its size, its place on the screen as whole numbers and as
/// floats' bits, and the view's value fetched from its block (`race::fetch_view_value`), which
/// scales the fog with the size.
#[derive(rc2k_derive::Cells)]
pub struct ViewSun {
    pub on_screen: PlainDword,
    pub size: PlainDword,
    pub hit: [PlainDword; 2],
    pub place: [PlainDword; 2],
    pub view_out: PlainDword,
}

impl ViewSun {
    pub const fn new() -> Self {
        Self {
            on_screen: PlainDword::new(0),
            size: PlainDword::new(0),
            hit: [const { PlainDword::new(0) }; 2],
            place: [const { PlainDword::new(0) }; 2],
            view_out: PlainDword::new(0),
        }
    }
}

impl PolygonWork {
    pub const fn new() -> Self {
        Self {
            polygon_next_record: PlainDword::new(0),
            polygon_textures: [const { PlainDword::new(0) }; 2],
            texture_page: PlainDword::new(0),
            polygon_colour: PlainDword::new(0),
            polygon_texture: PlainDword::new(0),
            face_shine: PlainDword::new(0),
            face_word: PlainDword::new(0),
            polygon_sides: [const { PlainDword::new(0) }; 2],
            polygon_kept: [const { PlainDword::new(0) }; 3],
            polygon_outside_count: PlainDword::new(0),
            polygon_vertex_count: PlainDword::new(0),
            polygon_next_entry: PlainDword::new(0),
            polygon_clip_mask: PlainDword::new(0),
            clip_plane: PlainDword::new(0),
            screen_clipped: PlainDword::new(0),
            depth_clipped: PlainDword::new(0),
            entry_third: PlainDword::new(0),
            polygon_drawn: PlainDword::new(0),
            face_sort_key: PlainDword::new(0),
            clip_kept_mask: PlainDword::new(0),
            clip_work: [const { PlainFloat::new(0.0) }; 12],
            clip_out_words: [const { PlainDword::new(0) }; 5],
            quad_vertices: [const { PlainDword::new(0) }; 32],
            smooth: PlainDword::new(0),
            plain_first: PlainDword::new(0),
            plain_second: PlainDword::new(0),
            cleared_rectangle: [
                PlainDword::new(0),
                PlainDword::new(0),
                PlainDword::new(0x31F),
                PlainDword::new(0x257),
            ],
            crossing_amount: PlainDword::new(0),
            fade_from: PlainDword::new(0),
            view_mirrored: PlainDword::new(0),
            sun: ViewSun::new(),
            window: crate::scene::view_window::ViewWindow::new(),
        }
    }
}

/// The front end's slot: the polygon work of the HUD, the loading bar, the frame's set-up
/// and the effects, which draw outside any view's pipeline (each writes its own count before
/// its fan is drawn).
pub fn front_end() -> &'static PolygonWork {
    &crate::race::view_scratch::front_end().polygon
}
