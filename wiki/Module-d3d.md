# Module: d3d

*Category: [Module reference](Modules)*

The thin 3D drawing layer. The original game used Direct3D only to fill triangles: every vertex
arrived already transformed, lit and clipped. This module keeps that boundary. The scene code
calls it to draw a fan of screen-space vertices, set a render state or a texture, or lock a
texture to write its pixels, and it passes that on to the platform, which notes it for a
renderer.

## State

`D3d` (`src/d3d/owner.rs`), a field of the graphics owner (`Gfx::d3d`). It holds the 3D
settings (`Cards3d`, `src/d3d/cards_store.rs`). Each drawing thread keeps its own cache of
render states (`src/d3d/render_cache.rs`) and its vertex scratch (`src/d3d/vertices_store.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `Vertex` | `src/d3d/mod.rs` | The vertex the layer takes: screen position, depth, colours, texture coordinates. |
| `draw_fan` | `src/d3d/draw_fan.rs` | Draws a triangle fan, clipped to the screen. |
| `set_render_state` | `src/d3d/set_render_state.rs` | Sets one render state (blending, depth test, fog ...), skipping repeats. |
| `set_texture` | `src/d3d/set_texture.rs` | Chooses the texture for what is drawn next. |
| `lock_texture_in` | `src/d3d/lock_texture.rs` | Gives access to a texture's pixels (for uploads and the mud). |
| `draw_hud_batch` | `src/d3d/draw_hud_batch.rs` | Draws the HUD's gathered triangles in one call per texture. |
| `copy_screen_to_texture` | `src/d3d/copy_screen_to_texture.rs` | Copies the drawn picture into a texture. |
| `Cache` | `src/d3d/render_cache.rs` | The states and texture currently set, per drawing thread. |
| `Call`, `note` | `src/d3d/recorder.rs` | A record of the calls made, for tests. |

## How it is laid out

A small flat folder, one function per file, plus the stores named above.

## Calls and callers

Called by [`scene`](Module-scene), [`effects`](Module-effects), [`race`](Module-race) and
[`hud`](Module-hud). It calls `platform::direct3d`, which builds the command list that
[`render`](Module-render) draws.

## Related pages

[Drawing a frame](Drawing-a-frame).
