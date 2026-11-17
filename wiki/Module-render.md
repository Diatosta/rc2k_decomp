# Module: render

*Category: [Module reference](Modules)*

The renderers. By the time drawing reaches this module it is a list of simple commands: draw
this fan of screen-space vertices with this texture and these states, clear the depth buffer.
A **backend** turns the list into pixels. There are three, and they are arranged to give the
same picture.

## State

Its own statics in `src/render/mod.rs` (the spare command lists, the changed textures, whether
the GPU holds the frame) and each backend's own state.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `Backend` | `src/render/mod.rs` | The trait a renderer implements. |
| `Command`, `CommandList` | `src/render/mod.rs` | A frame's drawing as data. |
| `decide`, `Wish`, `wanted` | `src/render/choice.rs` | Chooses the backend (`--renderer` and `RC2K_RENDERER` can force one; the options file's renderer is the weakest source). |
| `Gpu` | `src/render/gpu.rs` | The desktop GPU backend, on SDL's GPU API (Vulkan, Metal). |
| `Gpu` | `src/render/gles.rs` | The OpenGL ES backend of the handheld, and the OpenGL 3.3 one on the desktop. |
| `batched`, `run` | `src/render/gl_batching.rs` | Prepares the OpenGL ES backend's batches on a helper thread. |
| `Software` | `src/render/software.rs` | The software backend: splits the picture into tiles over the processor's cores. |
| `render_scale`, `flush_scale` | `src/render/mod.rs` | The graphics page's share of the screen's size the race's 3D is drawn at; each backend draws the 3D into a smaller or larger frame and brings it back to the screen's size, enlarging it below 100 % and averaging it down above (`small_frame.rs` for the software one; the GPU ones blit between the screen's target and a second one). |
| `menu_high_percent`, `set_menu_high` | `src/render/mod.rs` | The menus' 3D at the window's size: the percent of the menu frame's size its place in the window has, times the 3D's share (set by the platform when it shows the frame). A GPU backend draws a menu flush's 3D into larger targets (`MenuHigh` in `gpu.rs`, `HighTarget` in `gles.rs`) as well as into the frame, keeps a copy of the frame as it was right after that 3D (`kept`), and at the frame's end shows, per pixel, the larger picture where the frame still equals that copy and the frame's own enlarged picture where text or buttons were drawn over the models since (`shaders/merge.frag`, `MERGE_FS`). |
| `draw_fan` | `src/render/rasteriser.rs` | The rasteriser itself: fills a fan's triangles into a tile. |
| `Batches` | `src/render/batches.rs` | Groups the commands into runs that share a pipeline and texture. |
| `background.rs` | `src/render/background.rs` | The 2D picture under and over the 3D when the GPU holds the frame. |
| `dump_if_asked`, `hash_if_asked` | `src/render/dump.rs` | Writes or hashes frames for the picture checks. |

## How it is laid out

`mod.rs` is what every backend agrees on. `gpu.rs` + `gpu/` and `gles.rs` + `gles/` are the
two GPU backends (`quads.rs` / `layer2d.rs` draw the 2D layer as textured quads).
`software.rs`, `rasteriser.rs` and `rasteriser_neon.rs` (the ARM version of its inner loops)
are the software one. `shaders/` holds the shader sources and their compiled forms
(`tools/compile_shaders.sh` rebuilds them). `gpu_cut.rs`, `gpu_time.rs` and `seethru.rs` are
diagnostics.

## Calls and callers

It is fed by `platform::direct3d` and uses [`gfx`](Module-gfx) for pixel formats and the 2D
picture. Nothing in the game's logic calls it directly.

## Related pages

[Drawing a frame](Drawing-a-frame), [Porting to a new platform](Porting-to-a-new-platform),
[Running and options](Running-and-options).
