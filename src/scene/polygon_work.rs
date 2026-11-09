//! The polygon pipeline's working values, a view's own (frame plan F4): the polygon being
//! set up, clipped, projected and drawn (its colour, textures, sides, vertex count,
//! entry and record numbers, clip planes and flags, the clip's work runs), the texture page,
//! the object face's word and shine, and the sort key. One per view slot
//! (`race::view_scratch::ViewScratch::polygon`); the HUD, the loading bar and the menus use
//! the front end's. In the original these were `SceneDraw`'s `polygon_*`, `texture_page`,
//! `face_word`, `face_shine`, `clip_work`, `clip_work_2`, `clip_kept_mask`, `clip_plane`,
//! `screen_clipped`, `depth_clipped`, `entry_third` and `face_sort_key`, `quad_vertices`, written and read by every view in turn. Not here:
//! `LevelGrid`'s `smooth`, `plain_first` and `plain_second` (the HUD sets the plain switches
//! and leaves them set into the next frame's views until their first face resets them) and
//! `polygon_alpha` (set by the HUD's strips only, read by every view's drawing).

use crate::session::{Dword, Float};

pub struct PolygonWork {
    /// `g_polygonNextRecord`.
    pub polygon_next_record: Dword,
    /// The polygon's two texture words (`scene::draw_polygon`).
    pub polygon_textures: [Dword; 2],
    /// The texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch).
    pub texture_page: Dword,
    /// `g_polygonColour`.
    pub polygon_colour: Dword,
    /// `g_polygonTexture`.
    pub polygon_texture: Dword,
    /// `g_faceShine` (`scene::draw_object_face`).
    pub face_shine: Dword,
    /// A face's word (`scene::draw_object_face`).
    pub face_word: Dword,
    /// The polygon's two side switches (`scene::draw_polygon`, the quads).
    pub polygon_sides: [Dword; 2],
    /// Kept by `scene::draw_polygon`.
    pub polygon_kept: [Dword; 3],
    /// `g_polygonOutsideCount`.
    pub polygon_outside_count: Dword,
    /// `g_polygonVertexCount`.
    pub polygon_vertex_count: Dword,
    /// `g_polygonNextEntry`.
    pub polygon_next_entry: Dword,
    /// `g_polygonClipMask` ("cuttest").
    pub polygon_clip_mask: Dword,
    /// The clip's plane (`scene::clip`).
    pub clip_plane: Dword,
    /// Clipped by the screen ("donesuth").
    pub screen_clipped: Dword,
    /// Clipped by depth ("donezcut").
    pub depth_clipped: Dword,
    /// A list entry's third word (`scene::list_object_face`).
    pub entry_third: Dword,
    /// `g_polygonDrawn`.
    pub polygon_drawn: Dword,
    /// `g_faceSortKey` (`scene::face_sort_key`).
    pub face_sort_key: Dword,
    /// The clip's kept mask (`scene::clip`).
    pub clip_kept_mask: Dword,
    /// `scene::clip`'s work (`flt_623F80` ..); `race::car_shadow` keeps a vertex's pieces in
    /// [2], [4], [10], [11].
    pub clip_work: [Float; 12],
    /// `scene::clip`'s second work run; `race::car_shadow` keeps a vertex's byte +0x20 in [0].
    pub clip_work_2: [Dword; 5],
    /// The four quad vertices (8 dwords each; `scene::point_records::of_quad`): the screen
    /// quads', the halfway vertices' and the skidmarks' own. In the original
    /// `RaceVars.quad_vertices`; every writer sets dwords 3, 6, 7 of the vertices it names
    /// just before the polygon is clipped and drawn.
    pub quad_vertices: [Dword; 32],
}

impl PolygonWork {
    pub const fn new() -> Self {
        Self {
            polygon_next_record: Dword::new(0),
            polygon_textures: [const { Dword::new(0) }; 2],
            texture_page: Dword::new(0),
            polygon_colour: Dword::new(0),
            polygon_texture: Dword::new(0),
            face_shine: Dword::new(0),
            face_word: Dword::new(0),
            polygon_sides: [const { Dword::new(0) }; 2],
            polygon_kept: [const { Dword::new(0) }; 3],
            polygon_outside_count: Dword::new(0),
            polygon_vertex_count: Dword::new(0),
            polygon_next_entry: Dword::new(0),
            polygon_clip_mask: Dword::new(0),
            clip_plane: Dword::new(0),
            screen_clipped: Dword::new(0),
            depth_clipped: Dword::new(0),
            entry_third: Dword::new(0),
            polygon_drawn: Dword::new(0),
            face_sort_key: Dword::new(0),
            clip_kept_mask: Dword::new(0),
            clip_work: [const { Float::from_bits(0) }; 12],
            clip_work_2: [const { Dword::new(0) }; 5],
            quad_vertices: [const { Dword::new(0) }; 32],
        }
    }
}

/// The front end's slot: the polygon work of the HUD, the loading bar, the frame's set-up
/// and the effects, which draw outside any view's pipeline (each writes its own count before
/// its fan is drawn).
pub fn front_end() -> &'static PolygonWork {
    &crate::race::view_scratch::front_end().polygon
}
