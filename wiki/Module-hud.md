# Module: hud

*Category: [Module reference](Modules)*

The in-race display: the rev counter and speedometer, the times, gear and damage signs, the
stage progress bars and the two-player strips. The HUD is drawn as flat polygons in a virtual
screen of fixed size, scaled to the real one, gathered into a batch and drawn with one call per
texture after the 3D views.

## State

`Hud` (`src/hud/owner.rs`), a field of the race's owner (`Race::hud`). It is one block,
`HudBlock` in `src/hud/block_store.rs`: the batch's vertices and indices, the virtual screen's size,
the layouts' values. The numbers the HUD shows (speed, times, gear) are filled in by the race
each frame (`src/race/panel_store.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `overlay_pass` | `src/hud/overlay_pass.rs` | The entry: sets the states for 2D drawing and draws the HUD over the views. |
| `draw_hud` | `src/hud/draw_hud.rs` | Draws every HUD element for one player. |
| `layout_0`, `layout_1`, `layout_2` | `src/hud/layout_0.rs` ... | The HUD's arrangements (one player, two players, and so on). |
| `disc`, `needle` | `src/hud/dials.rs` | The dials. |
| `draw_stage_bars` | `src/hud/draw_stage_bars.rs` | The progress bars along the stage. |
| `draw_signs` | `src/hud/draw_signs.rs` | Warning and gear signs. |
| `add_quad`, `add_sprite`, `add_triangle` | files of the same names | Add a shape to the batch. |
| `flush_batch` | `src/hud/flush_batch.rs` | Draws the batch gathered so far. |

## How it is laid out

A small flat folder: `draw_*` (elements), `add_*` (shapes into the batch), `layout_*`.
The sizes and rules without state are in `rc2k_core/src/hud/`.

## Calls and callers

It draws through [`d3d`](Module-d3d) (`draw_hud_batch`) and uses textures set up by
[`scene`](Module-scene). It is called from the race's drawing (`race::draw_race_scene`).
The map and the pace-note arrows are drawn by [`race`](Module-race).

## Related pages

[Drawing a frame](Drawing-a-frame), [A race, step by step](A-race-step-by-step).
