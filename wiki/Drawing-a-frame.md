# Drawing a frame

*Category: [Game flow](Game-flow)*

This page follows one picture of a race from the scene's geometry to the screen: how the game
turns its world into finished triangles, how those reach one of three renderers, where the 2D
parts (menus, text, the HUD) come in, and how views are drawn on several threads at once.

The game side is `src/scene/` (objects, polygons, textures, lighting), `src/race/frame_draw/`
and `src/race/draw_race_scene.rs` (what a race draws), `src/effects/` and `src/hud/`. The thin
layer between game and renderer is `src/d3d/` and `src/d3d/device.rs`. The renderers
are in `src/render/`.

## The idea in one paragraph

The 1999 game used its graphics card only to fill triangles. It did all the geometry itself:
it turned every point into view space, lit it, clipped it against the screen and projected it,
then handed the card flat, screen-space triangles with a colour at each corner. This project
keeps that design. Everything up to "finished triangles" is the game's own code and gives the
same numbers as the original. Only the last stage, filling the triangles, is new, and it can
be done by a GPU or by the processor.

## A race frame from the top

Between the race's ticks (the simulation runs 100 times a second) a frame is drawn from a blend
of the last two ticks: `race::blend` keeps, after every tick, each car's place and turn matrices,
its wheels' turns and heights and its camera's place and angles, and `draw_race_scene` puts the
blend at the frame's time (`rc2k_core::race::blend`: the time since the last tick over 10 ms;
places and angles linearly, the shorter way round a turn, matrices entry by entry) into the cars'
blocks while it draws and puts the ticks' values back after. A car or camera that moved far in one
tick (a reset, a camera cut) is not blended across it. The advanced page's "smooth frames"
switch turns this off. Frames drawn without a tick step before them (a pause, a replay's still)
show the last tick exactly.

`drive::race_frame` (`src/drive/race_frame.rs`) draws one picture:

1. the platform's events are handled;
2. the 3D scene is begun (`scene::begin_scene`);
3. `race::draw_race_scene` draws everything in the world and over it;
4. the scene is ended (`scene::end_scene`);
5. the race's sounds get their turn for the frame;
6. `race::show_frame` adds the last 2D bits (the frame rate, the credits' text) and shows the
   picture.

`draw_race_scene` (`src/race/draw_race_scene.rs`) first prepares what all views share: which
cars are shown, the textures this frame needs, the cars' mud, the map's scroll. Then it draws
each **view**. A one-player race has one view; a split-screen race has two, side by side or one
above the other, or with three or four players one for each in the layout chosen with the F2 key
(`rc2k_core::race::view::Split`: quarters, rows, columns, or for three one wide view above two;
the HUD of those, `hud::layout_many`, is a small panel of place, stage time, progress bar, gear and speed
in each part, in screen pixels). A view may have a rear-view **mirror**, which is a small view of its own looking
backwards (not in the split screens of more than two players: their view slots are the players').

For each view, in order:

| Part | What is drawn | Code |
| --- | --- | --- |
| Set-up | The view's window on the screen, its camera matrices, draw distance, fog. | `src/race/frame_draw/view.rs`, `scene::set_view_window` |
| Sky | The sky behind everything. | `src/race/frame_draw/view.rs` |
| Course | The road and land near the camera, from the course's grid of faces. | `src/race/frame_draw/course_view.rs` |
| Objects and cars | Scenery, the cars, shadows, skid marks, car lights, lens flares, sorted by depth where they are see-through. | `src/race/frame_draw/scene_objects.rs`, `scene::draw_object` |
| Effects | Rain, snow, dust and spray particles, glows, drops on the windscreen. | `effects::run_frame` (`src/effects/`) |
| Mirror | The same again, smaller, looking back. | `draw_race_scene.rs` |

After the views comes the 2D pass over the whole screen (the HUD's 800 x 600 layout keeps its shape on a screen wider than 4:3, its pieces anchored left, middle or right by `rc2k_core::hud::anchor`; on a screen narrower than 4:3 the layout keeps its shape by the width's factor and its pieces keep to the top, bottom or middle (`hud::anchor::narrow_y`, `hud::place`); the 3D's scale is made from the screen's height on such a screen, `scene::view_window::design_width`, and the tests that cut the course to the sides of the view widen by `scene::view_window::side_widening`; the top progress bar spans the screen, `hud::anchor::bar_x`, and its marker stands at the bar's place for its progress, `hud::anchor::marker_edge_x`, so it meets the parts' ends and reaches the bar's end at the finish): the HUD (`hud::overlay_pass` →
`hud::draw_hud`: dials, times, the map, notices) and name plates.

## From an object to triangles

Take one scenery object. `scene::draw_object` runs a short list of steps
(`src/scene/draw_object.rs`):

1. **Light and material.** The object's light and its material's render settings are chosen
   (`scene::set_object_light`, `scene::set_object_material`).
2. **Fade.** A far object fades out, or is swapped for a simpler one
   (`scene::set_object_fade`).
3. **Points.** Every point of the object is turned by the object's and the view's matrices
   into view space (`scene::place_object_points`).
4. **Faces listed.** The faces that face the camera are listed
   (`scene::list_object_faces`; `scene::face_faces_away` drops the rest).
5. **Sorted** back to front where the material needs it (`scene::sort_listed_faces`).
6. **Drawn.** Each face goes through the polygon path below (`scene::draw_object_faces`).

The polygon path, for one face:

| Stage | What happens | File |
| --- | --- | --- |
| Project | Each point is projected to the screen; points are cached per frame so a shared point is done once. | `project_polygon.rs`, `project_point.rs`, `point_records.rs` |
| Clip | The polygon is cut against the near and far depths and the four screen edges. New corners get colours and texture coordinates by interpolation. | `clip_polygon_by_depth.rs`, `clip_polygon_to_screen.rs`, `clip.rs` |
| Light | Each corner gets its colour from the stage's light and nearby lamps. | `vertex_light.rs`, `light_tables.rs`, `add_point_lights.rs` |
| Fog | Each corner gets its fog amount from the view's fog table. | `fog_polygon.rs`, `fog.rs` |
| Shade and submit | The polygon is drawn in one or two passes by its material (a second pass adds shine or a reflection) and handed over as a triangle fan. | `draw_polygon.rs`, `submit_polygon.rs`, `draw_fan.rs` |

Course faces take the same path from the clip stage on. Textures are chosen per frame: each
face names a texture slot, and `scene::load_slot_textures` and its relatives make sure the
pictures this frame needs are loaded at a sensible size (`texture_levels.rs`,
`texture_needs.rs`).

## The thin layer: `d3d`

The game's 3D code ends in a handful of calls: set a render state
(`d3d::set_render_state`), set the texture (`d3d::set_texture`), draw a fan of finished
vertices (`d3d::draw_fan`), draw the HUD's batch of triangles (`d3d::draw_hud_batch`), clear
the depth buffer. A **render state** is a numbered setting: depth test on or off, how a
see-through pixel blends, fog on or off, texture filtering.

In this program these calls draw nothing. They **note** what was asked
(`src/d3d/recorder.rs`, `src/d3d/device.rs`) as a list of commands.

## The command list

`src/render/mod.rs` defines what the renderers must agree on (how a frame ends is `render/frame_end.rs` on the systems with a window and `render/frame_end_panel.rs` on the handheld, chosen once, with the same function names; the backend that runs, the GPU API's or the OpenGL one, is held behind one lock in `render/backend.rs`, never both):

- `Command::Fan`: a span of vertices, the render states in force, the texture, and the fan's
  bounds on the screen.
- `Command::Clear`: set the depth buffer back in a rectangle (a new view begins). The game does not
  clear the depth at every frame's start, so the GPU renderers (Vulkan/Metal and OpenGL ES) reset
  the whole depth buffer when a frame ends; otherwise the last frame's depths would hide patches
  of the rear-view mirror in the next.
- `CommandList`: the commands in the order they were noted, with their vertices and the
  textures they use.
- `Backend`: the trait a renderer implements. It has one job, `draw(target, commands)`.

A vertex is already on the screen: x, y, depth, a colour, a second colour for shine and fog,
and texture coordinates.

## The three renderers

| Renderer | File | Where | How |
| --- | --- | --- | --- |
| GPU | `src/render/gpu.rs` (its parts by concern in `src/render/gpu/`: device, pipelines, targets, uploads, draw (a flush planned, its textures and transfers staged, then the copy pass and the render passes), readback) | Desktop (Vulkan on Windows and Linux, Metal on macOS), through SDL's GPU API. | Fans become triangles in one buffer, grouped into batches that share states and texture (`src/render/batches.rs`), drawn with three small shaders (`src/render/shaders/`). |
| OpenGL / OpenGL ES | `src/render/gles.rs` (its parts in `src/render/gles/`: shader sources, programs, textures, targets, draw, present, and the GL thread) | The handheld (OpenGL ES 3), and on the desktop the renderer after the GPU API in the default order (an OpenGL 3.3 core context on a hidden SDL window). | The same batches, drawn through OpenGL ES 3. A thread of its own holds the GL context; another prepares each part's batches one part ahead (`src/render/gl_batching.rs`). A batch whose pixels are all opaque and cannot fail the alpha test is drawn with the blend off by a copy of the shader without the test: the picture is the same, and the GPU can skip hidden pixels before shading them. On the desktop the same shader text runs under desktop GLSL's version line (`#version 330 core` in place of `#version 300 es`, swapped when the shader is compiled), the context is made on the GL thread on a window the game's thread creates, and that window is the one the player sees while the GL runs (the game's own window, which SDL's renderer is made on, is hidden): each frame's end goes to it as the handheld's goes to its panel, the target shown with the canvas's written pixels over it, and a frame the GL does not hold shown from the canvas's picture, with no read-back (a screenshot still brings the frame back into the canvas). The debug UI, which SDL's renderer draws, is not over the GL's frames. For machines without Vulkan or Direct3D 12 (not on macOS: there an OpenGL context can only be made on the main thread, and a Mac without Metal goes to the software renderer). Each desktop backend can be stopped and started again while the game runs (the advanced page's renderer swap, `platform::swap_renderer`): the GPU API's resources are released and its device destroyed after the window's renderer on it; the GL thread's context and window are destroyed (the game's own window is shown again, where the GL's was) once both of its threads are joined; a started backend draws every texture and the canvas up again as the frames name them. The 3D's shader is built once per kind of batch (`batches::ShaderKind`: textured or not, fogged or not, which alpha test, a second pass or none), with those choices as constants and only the vertex values the kind reads. |
| Software | `src/render/software.rs`, `src/render/rasteriser.rs` (the four-lane row path in `rasteriser/simd.rs`, the texel reads in `rasteriser/texels.rs`) | Anywhere, as the fallback or by choice. | The processor fills every triangle, with a depth buffer, texturing, blending and fog as the states say. The picture is cut into tiles spread over the processor's cores; the result is identical to one thread's. With the graphics page's mipmapping on, a textured triangle that is smaller on the screen than its texture reads a smaller copy of the texture (each texel the average of four of the level above, made the first time the texture is drawn and again when it is written), chosen for the triangle from how its texels change across the screen; with it off, the first level only, as before. The OpenGL renderer does the same on the GPU (its texture arrays get their smaller levels the first time a drawn triangle asks for them, so a game that never turns mipmapping on pays nothing; the levels add a third to each array's memory); the GPU API renderer does the same (its textures are made with their levels, uploaded from the same averages, and a sampler that reads linearly inside the nearest smaller level is bound for a mipmapping batch; the levels are made the first time a batch asks, which makes the textures again, and add a third to each texture's memory). |

`src/render/choice.rs` picks one at start: the GPU API first, then OpenGL, then the software
rasteriser, each when the one before cannot start. `--renderer gl` (or `RC2K_RENDERER=gl`) asks for OpenGL alone (the software rasteriser if it cannot start). The switch `--renderer software` (or the environment variable `RC2K_RENDERER=software`) forces the rasteriser.
On the handheld the fallback rasteriser draws straight into the framebuffer.

The renderers are expected to draw the same picture. The project's checks compare them (see
[Testing](Testing)).

## The 2D layer

Menus, text and pictures are drawn by the game's own 2D code into a picture in memory, in the
screen's pixel format (always 32 bits a pixel, XRGB):

- pictures: `src/gfx/` loads PCX and PNG files ([Textures (PCX)](Textures-PCX),
  [Screens (PNG)](Screens-PNG)) and copies them to the screen
  (`gfx::copy_picture_to_surface`);
- text: `src/text/` draws strings glyph by glyph (`gfx::blit_glyph`; see [Fonts](Fonts));
- boxes and bars: `gfx::rect` and the bar blitters.

`src/platform/mod.rs` holds that picture (`Screen`) and `src/shell/frame.rs` puts it on the window when the game
shows a frame.

On the GPU renderers, 2D and 3D have to be combined, because the 3D is on the GPU and the 2D
is in memory. Three things make that cheap:

- the 2D memory (the "canvas") starts each layer marked as unwritten, and only the rows the
  game wrote are uploaded and laid over the 3D (`src/render/canvas_rows.rs`);
- glyphs are also noted as textured quads and drawn by the GPU directly
  (`src/gfx/draw2d/`, `src/render/gpu/quads.rs`, `src/render/gles/layer2d.rs`);
- a menu's background picture is kept as a GPU texture and drawn under the canvas, uploaded
  again only when it changes (`src/render/background.rs`).

A menu frame is mostly 2D. Menus with a 3D part (the car carousel) draw a small scene through
the same object path (`scene::draw_carousel`, `scene::prepare_menu_scene`) between the
background and the text. The frame is 800 x 600 and the window shows it enlarged, so on the GPU
renderers the models are drawn a second time at the size the frame has in the window (times the
3D resolution share): the picture so far enlarged, the 3D over it. The frame's own picture, with
the low-resolution 3D, stays what the 2D after the 3D is drawn over; at the frame's end a merge
takes the large picture wherever the frame is unchanged since the 3D and the frame's own enlarged
picture elsewhere (text and buttons over the models). The software rasteriser draws the models at
the frame's size.

## Views side by side on several threads

In the original, one thread drew the first view, then its mirror, then the second view. Here,
on a machine with two or more cores, those **parts** are drawn at the same time on helper
threads, and the picture is the same as before. The code is `src/scene/workers.rs`.

How it stays the same picture:

- Each part notes its drawing into its **own** command list. When all parts are done, the
  lists are joined in the fixed order one thread would have drawn them. The renderer never
  sees which thread made what.
- Each part works on its **own** working state: a `PartScratch` (the polygon being built, the
  point cache) and a `ViewScratch` (the view's matrices, fog, sort lists;
  `src/race/view_scratch.rs`). No part writes anything another part reads.
- What all parts only read (the course, the light tables) is lent to them read-only for the
  length of the drawing.
- What a view must hand on to the rest of the game (counters, which textures it needed) is
  applied afterwards on the game's thread, in the views' order.

Inside a view, the player's own car is the most expensive single thing, so it is drawn as a
piece "aside" on another thread while the view's thread draws the course, and its commands are
put in at the car's place in the order (`src/race/car_part.rs`).

The course's sorted faces, the next largest piece, are cut into runs of faces next to each other
in the list when drawing threads stand idle (`src/race/face_runs.rs`,
`src/scene/workers/list_runs.rs`): the view's thread draws the first run, each other run is drawn
on a drawing thread with a scratch and a copy of the view of its own, and the runs' commands,
render states, point records and changes to the view are put together in the list's order. A
face draws from the view's set-up and from what it makes itself, never from what the face before
it left, so the commands are the ones one thread notes.

Environment variables exist for checking this: `RC2K_MIRROR_BESIDE=0` draws the mirror
after its view again, `RC2K_DRAW_SERIAL` runs the parts one after another on the game's
thread while keeping their separate states, and `RC2K_FACE_RUNS=0` draws the course's faces on
the view's thread alone (a number cuts every list into that many runs; with `RC2K_DRAW_SERIAL`,
4096 makes every face a run of its own, the check that no face depends on the one before). The type-level rules that keep the threads apart
are described in [Threads and shared state](Threads-and-shared-state).

## Where to look

| To change or understand | Start in |
| --- | --- |
| What a race frame draws, and in what order | `src/race/draw_race_scene.rs` |
| The course's drawing | `src/race/frame_draw/course_view.rs` |
| An object's drawing | `src/scene/draw_object.rs`, `draw_object_faces.rs` |
| Clipping | `src/scene/clip.rs` |
| Lighting | `src/scene/vertex_light.rs`, `light_tables.rs` |
| Fog | `src/scene/fog.rs`, `fog_polygon.rs` |
| Textures per frame | `src/scene/textures.rs`, `load_slot_textures.rs` |
| Weather and particles | `src/effects/run_frame.rs` |
| The HUD | `src/hud/draw_hud.rs`, `src/hud/batch.rs` |
| What a render state means to a GPU | `src/render/batches.rs` (`batches/keys.rs`, `batches/build.rs`) |
| The software rasteriser | `src/render/rasteriser.rs` |
| Which renderer starts | `src/render/choice.rs` |
| Drawing on several threads | `src/scene/workers.rs`, `src/scene/workers/list_runs.rs`, `src/memory/part_cell.rs` |
| The screen's memory and the window | `src/platform/mod.rs` |

Next: [Sound and music](Sound-and-music). See also [Module-scene](Module-scene),
[Module-render](Module-render), [Module-d3d](Module-d3d), [Module-gfx](Module-gfx),
[Object files](Object-files), [Course files](Course-files).
