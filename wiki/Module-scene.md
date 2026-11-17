# Module: scene

*Category: [Module reference](Modules)*

The game's own 3D. The game does all its geometry itself: it loads objects, places them, turns
their points by the view's matrix, lights them, sorts and clips their faces, projects them onto
the screen and applies fog, and only then hands finished triangles to the drawing layer
([`d3d`](Module-d3d)). This module is that code, together with the textures' loading and
bookkeeping and the 3D scenes of the menus (the car carousel).

## State

The owner is `Scene` (`src/scene/owner.rs`), in groups (`Drawing`, `Lights`, `Objects`) whose
parts are the `*_store.rs` files: objects and their faces and points, the texture tables, the
light tables, the drawing's working memory. Each drawing thread has its own scratch
(`src/scene/workers.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `load_object` | `src/scene/load_object.rs` | Reads an object file into the object tables. |
| `read_kind_records` | `src/scene/read_kind_records.rs` | Reads the table that describes each kind of object. |
| `set_up_textures`, `load_slot_textures` | files of the same names | Sets up the texture tables and loads the textures a stage needs. |
| `build_texture_catalogue` | `src/scene/build_texture_catalogue.rs` | Lists which picture belongs to which texture. |
| `begin_scene`, `end_scene`, `present_frame` | files of the same names | A frame's start, end and showing. |
| `draw_object` | `src/scene/draw_object.rs` | Draws one object: its points placed and lit, its faces listed. |
| `draw_polygon` | `src/scene/draw_polygon.rs` | One face: projected, clipped, fogged, sent on. |
| `clip_polygon_to_screen`, `clip_polygon_by_depth` | files of the same names | The clippers. |
| `project_point` | `src/scene/project_point.rs` | A point from the view's space to the screen. |
| `sort_listed_faces` | `src/scene/sort_listed_faces.rs` | Orders the faces far to near. |
| `Fog` | `src/scene/fog.rs` | The fog's levels. |
| `draw_carousel` | `src/scene/draw_carousel.rs` | The turning cars of the car choice screen. |
| `set_up_engine_in` | `src/scene/set_up_engine.rs` | Prepares the 3D code's tables. |
| `workers.rs` | `src/scene/workers.rs` | Draws the players' views and mirrors on several threads at once; `threads` is how many parts may draw at once (the advanced page's row), read at each frame. |

## How it is laid out

Flat, about 165 files. Groups by name: `draw_*` (objects, polygons, the menus' scenes),
`load_*` and `*texture*` (textures), `*light*` (lighting), `clip*` and `project_*`
(geometry), `matrix*.rs`, `vector3.rs` and `view_*` (the view), `*_store.rs` (state).

## Calls and callers

It draws through [`d3d`](Module-d3d), uses `math` and the course data in
[`race`](Module-race), and reads pictures through [`gfx`](Module-gfx). It is called by `race`,
[`effects`](Module-effects), [`physics`](Module-physics) (matrices and the course's points),
[`drive`](Module-drive) and [`menu`](Module-menu).

## Related pages

[Drawing a frame](Drawing-a-frame), [Object files](Object-files),
[Object and texture tables](Object-and-texture-tables), [Textures (PCX)](Textures-PCX).
