# Module: debug

*Category: [Module reference](Modules)*

The developers' debug pages, shown as a window over the game (Ctrl+F11 opens it, Ctrl+F12
freezes a race). The original game had these pages hidden; here they are a tabbed Dear ImGui
window that reads the game's variables while it runs. The window exists in the 64-bit desktop
builds only; on other builds the module's calls do nothing.

## State

No owner of its own. It reads the other modules' owners, and keeps a few counters and switches
in statics (the freeze flag in `src/debug/overlay.rs`, timings in `src/debug/timing.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `toggle`, `event`, `draw`, `is_open` | `src/debug/imgui_sdl.rs` (stand-ins in `src/debug/mod.rs` where the window is not built) | The doors the rest of the game uses: open or close, pass an input event, draw the window. |
| `take_keys`, `frozen` | `src/debug/overlay.rs` | The two key combinations, and whether the race is frozen. |
| `build` | `src/debug/ui.rs` | Builds the window's pages each frame. |
| `imgui_sdl` | `src/debug/imgui_sdl.rs` | Dear ImGui on SDL: the window, its input and drawing. |
| `Stage`, `start`, `stop` | `src/debug/timing.rs` | The profiler's named stages, timed by the code they wrap. |
| `tab` | `src/debug/switches.rs`, `kstar.rs`, `dyno.rs`, `gpu_cuts.rs`, `dev_front_end.rs` | One function per tab: switches, the developers' modes, the dyno, GPU diagnostics, the developers' front-end shortcuts. |
| `CheckHasher` | `src/debug/check_hash.rs` | A hash used by checks that compare two runs. |

## How it is laid out

One file per tab or page (`players.rs`, `effects_page.rs`, `screens.rs`, `wear_page.rs`), the
rows the display page shows (`display_rows.rs`, `watch_rows.rs`, `row_cells.rs`), and the
ImGui glue.

## Calls and callers

It reads [`race`](Module-race), [`scene`](Module-scene), [`physics`](Module-physics) and
`memory`. The race loop, the platform's event pump and the frame's hand-over call it.

## Related pages

[The debug window](The-debug-window).
