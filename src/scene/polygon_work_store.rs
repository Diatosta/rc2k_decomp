//! The polygon pipeline's working values, a view's own: the polygon being set up, clipped,
//! projected and drawn. One per view slot (`race::view_scratch::ViewScratch::polygon`); the HUD,
//! the loading bar and the menus use the front end's. Each part is a plain record in its own
//! `PlainSlot` (the single words, the clip's work, the quad vertices, the sun, the window), so
//! borrowing one never conflicts with borrowing another; a drawing copies the whole work for a
//! piece of it on another thread and merges what the piece changed (`copy_from`, `merge_changes`).
//! Not here: `polygon_alpha` (set by the HUD's strips only, read by every view's drawing).

use crate::race::plain_slot::{Merge, PlainSlot, merge_run, merge_value};

pub struct PolygonWork {
    /// The single words and short runs (everything but the work runs below and the sun and
    /// the window).
    pub scalars: PlainSlot<PolygonScalars>,
    /// `scene::clip`'s work runs.
    pub clip: PlainSlot<ClipWork>,
    /// The four quad vertices (`scene::point_records::of_quad`).
    pub quads: QuadSlot,
    /// The view's sun (`race::place_sun`; the hit test covers it).
    pub sun: ViewSun,
    /// The view's window, scale and depths (`scene::view_window`): set before the view's
    /// scene begins, so `ViewScratch::begin` keeps it.
    pub window: crate::scene::view_window::ViewWindow,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PolygonScalars {
    /// The next free record of the pool.
    pub polygon_next_record: u32,
    /// The polygon's two texture words (`scene::draw_polygon`).
    pub polygon_textures: [u32; 2],
    /// The texture page ("tpage"; `scene::look_up_texture`; the loading bar's switch).
    pub texture_page: u32,
    /// The polygon's colour.
    pub polygon_colour: u32,
    /// The polygon's texture.
    pub polygon_texture: u32,
    /// The polygon's two side switches (`scene::draw_polygon`, the quads).
    pub polygon_sides: [u32; 2],
    /// Kept by `scene::draw_polygon`.
    pub polygon_kept: [u32; 3],
    /// The polygon's count of points outside.
    pub polygon_outside_count: u32,
    /// The polygon's vertex count.
    pub polygon_vertex_count: u32,
    /// The next free entry of the work table.
    pub polygon_next_entry: u32,
    /// The polygon's clip mask.
    pub polygon_clip_mask: u32,
    /// The clip's plane (`scene::clip`).
    pub clip_plane: u32,
    /// Clipped by the screen ("donesuth").
    pub screen_clipped: u32,
    /// Clipped by depth ("donezcut").
    pub depth_clipped: u32,
    /// A list entry's third word (`scene::list_object_face`).
    pub entry_third: u32,
    /// The face's sort key (`scene::face_sort_key`).
    pub face_sort_key: u32,
    /// The texture filter's switches: `smooth` is the pass being drawn's (render state 3: 3
    /// smooth, 1 not), set from the pass's set's `plain`, 1 when its texture's mapping is
    /// 0. A view starts from `race::view_scratch::filter_input`; the HUD sets the front
    /// end's slot's `plain` 1, 1 (`hud::overlay_pass`).
    pub smooth: u32,
    /// The first texture set's `plain` switch.
    pub plain_first: u32,
    /// The second texture set's `plain` switch.
    pub plain_second: u32,
    /// The clear's rectangle (left, top, right, bottom), each view's own.
    pub cleared_rectangle: [u32; 4],
    /// The amount (alpha) byte of the vertex a clip crossing makes, blended between its two
    /// ends, as a dword (`race::car_shadow`), each view's own.
    pub crossing_amount: u32,
    /// The colour the faded polygons fade from (the fog colour `scene::set_fog_colour` was
    /// given), each view's own.
    pub fade_from: u32,
    /// 1 while the rear-view mirror is drawn (set by `race::mirror_view_window`; a main
    /// view's stays 0): its polygons' vertices turned round, its car lights and turn the
    /// mirror's.
    pub view_mirrored: u32,
}

impl PolygonScalars {
    pub const fn new() -> Self {
        Self {
            polygon_next_record: 0,
            polygon_textures: [0; 2],
            texture_page: 0,
            polygon_colour: 0,
            polygon_texture: 0,
            polygon_sides: [0; 2],
            polygon_kept: [0; 3],
            polygon_outside_count: 0,
            polygon_vertex_count: 0,
            polygon_next_entry: 0,
            polygon_clip_mask: 0,
            clip_plane: 0,
            screen_clipped: 0,
            depth_clipped: 0,
            entry_third: 0,
            face_sort_key: 0,
            smooth: 0,
            plain_first: 0,
            plain_second: 0,
            cleared_rectangle: [0, 0, 0x31F, 0x257],
            crossing_amount: 0,
            fade_from: 0,
            view_mirrored: 0,
        }
    }
}

impl Merge for PolygonScalars {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(
            &mut self.polygon_next_record,
            &now.polygon_next_record,
            &start.polygon_next_record,
        );
        merge_run(
            &mut self.polygon_textures,
            &now.polygon_textures,
            &start.polygon_textures,
        );
        merge_value(
            &mut self.texture_page,
            &now.texture_page,
            &start.texture_page,
        );
        merge_value(
            &mut self.polygon_colour,
            &now.polygon_colour,
            &start.polygon_colour,
        );
        merge_value(
            &mut self.polygon_texture,
            &now.polygon_texture,
            &start.polygon_texture,
        );
        merge_run(
            &mut self.polygon_sides,
            &now.polygon_sides,
            &start.polygon_sides,
        );
        merge_run(
            &mut self.polygon_kept,
            &now.polygon_kept,
            &start.polygon_kept,
        );
        merge_value(
            &mut self.polygon_outside_count,
            &now.polygon_outside_count,
            &start.polygon_outside_count,
        );
        merge_value(
            &mut self.polygon_vertex_count,
            &now.polygon_vertex_count,
            &start.polygon_vertex_count,
        );
        merge_value(
            &mut self.polygon_next_entry,
            &now.polygon_next_entry,
            &start.polygon_next_entry,
        );
        merge_value(
            &mut self.polygon_clip_mask,
            &now.polygon_clip_mask,
            &start.polygon_clip_mask,
        );
        merge_value(&mut self.clip_plane, &now.clip_plane, &start.clip_plane);
        merge_value(
            &mut self.screen_clipped,
            &now.screen_clipped,
            &start.screen_clipped,
        );
        merge_value(
            &mut self.depth_clipped,
            &now.depth_clipped,
            &start.depth_clipped,
        );
        merge_value(&mut self.entry_third, &now.entry_third, &start.entry_third);
        merge_value(
            &mut self.face_sort_key,
            &now.face_sort_key,
            &start.face_sort_key,
        );
        merge_value(&mut self.smooth, &now.smooth, &start.smooth);
        merge_value(&mut self.plain_first, &now.plain_first, &start.plain_first);
        merge_value(
            &mut self.plain_second,
            &now.plain_second,
            &start.plain_second,
        );
        merge_run(
            &mut self.cleared_rectangle,
            &now.cleared_rectangle,
            &start.cleared_rectangle,
        );
        merge_value(
            &mut self.crossing_amount,
            &now.crossing_amount,
            &start.crossing_amount,
        );
        merge_value(&mut self.fade_from, &now.fade_from, &start.fade_from);
        merge_value(
            &mut self.view_mirrored,
            &now.view_mirrored,
            &start.view_mirrored,
        );
    }
}

/// `scene::clip`'s work: the first output run as float bits (twelve words; `race::car_shadow`
/// keeps a vertex's pieces in [2], [4], [10], [11]) and the second output run (five words written
/// back from its work; `race::car_shadow` keeps the new crossing vertex's fog byte in [0]).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ClipWork {
    pub work: [u32; 12],
    pub out_words: [u32; 5],
}

impl ClipWork {
    pub const fn new() -> Self {
        Self {
            work: [0; 12],
            out_words: [0; 5],
        }
    }
}

impl Merge for ClipWork {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.work, &now.work, &start.work);
        merge_run(&mut self.out_words, &now.out_words, &start.out_words);
    }
}

/// The four quad vertices (8 dwords each): the screen quads', the halfway vertices' and the
/// skidmarks' own. Every writer sets dwords 3, 6, 7 of the vertices it names just before the
/// polygon is clipped and drawn.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QuadVertices {
    pub words: [u32; 32],
}

impl QuadVertices {
    pub const fn new() -> Self {
        Self { words: [0; 32] }
    }
}

impl Merge for QuadVertices {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_run(&mut self.words, &now.words, &start.words);
    }
}

pub type QuadSlot = PlainSlot<QuadVertices>;

/// A view's sun: on the screen and not covered (`race::place_sun` sets it, the polygons' hit test,
/// `scene::draw_polygon`, clears it), its size, its place on the screen as whole numbers and as
/// floats' bits, and the view's value fetched from its block (`race::fetch_view_value`), which
/// scales the fog with the size.
pub type ViewSun = PlainSlot<SunValues>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SunValues {
    pub on_screen: u32,
    pub size: u32,
    pub hit: [u32; 2],
    pub place: [u32; 2],
    pub view_out: u32,
}

impl SunValues {
    pub const fn new() -> Self {
        Self {
            on_screen: 0,
            size: 0,
            hit: [0; 2],
            place: [0; 2],
            view_out: 0,
        }
    }
}

impl Merge for SunValues {
    fn merge_changes(&mut self, now: &Self, start: &Self) {
        merge_value(&mut self.on_screen, &now.on_screen, &start.on_screen);
        merge_value(&mut self.size, &now.size, &start.size);
        merge_run(&mut self.hit, &now.hit, &start.hit);
        merge_run(&mut self.place, &now.place, &start.place);
        merge_value(&mut self.view_out, &now.view_out, &start.view_out);
    }
}

impl PolygonWork {
    /// Every field set from `other`'s.
    pub fn copy_from(&self, other: &Self) {
        self.scalars.copy_from(&other.scalars);
        self.clip.copy_from(&other.clip);
        self.quads.copy_from(&other.quads);
        self.sun.copy_from(&other.sun);
        self.window.copy_from(&other.window);
    }

    /// What `now` changed since it was `start` put into this record, field by field.
    pub fn merge_changes(&self, now: &Self, start: &Self) {
        self.scalars.merge_changes(&now.scalars, &start.scalars);
        self.clip.merge_changes(&now.clip, &start.clip);
        self.quads.merge_changes(&now.quads, &start.quads);
        self.sun.merge_changes(&now.sun, &start.sun);
        self.window.merge_changes(&now.window, &start.window);
    }
}

impl PolygonWork {
    pub const fn new() -> Self {
        Self {
            scalars: PlainSlot::new(PolygonScalars::new()),
            clip: PlainSlot::new(ClipWork::new()),
            quads: PlainSlot::new(QuadVertices::new()),
            sun: PlainSlot::new(SunValues::new()),
            window: PlainSlot::new(crate::scene::view_window::WindowValues::new()),
        }
    }
}
