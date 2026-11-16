# Module: rc2k_core and rc2k_derive

*Category: [Module reference](Modules)*

`rc2k_core/` is the game's logic as plain Rust: formulas, table lookups, file layouts, the
order of steps in a flow. It has no state of its own, does not know about the window, sound or
files on disk, and contains no unsafe code (the crate forbids it). Because of that its tests
run anywhere, without the game's files, and most of the project's unit tests are here.

## How `src/` uses it

A function in `src/` typically gathers values from an owner, calls a function in `rc2k_core`
to decide something, and stores the result. Where the logic is a longer sequence that needs
to call back into the game (run a screen, play a sound, read a file), `rc2k_core` defines a
trait for what it needs and `src/` implements it. For example the event flows are written in
`rc2k_core::menu::flow` against a trait, and `src/menu/flow_game.rs` (`Game`) is the game's
side of it; the screens' layout rules in `rc2k_core::screens` draw through the traits
implemented in `src/screens/game_screen.rs`.

When a comment in `src/` says "Logic in `rc2k_core::...`", the rule itself is there.

## Its folders

The folders mirror `src/`'s: the logic of `src/physics/` is in `rc2k_core/src/physics/`, and
so on.

| Folder | Holds |
| --- | --- |
| `audio`, `sound` | Voice and mixer rules, levels, command layouts; the race's sound decisions. |
| `d3d`, `scene`, `hud`, `effects` | Geometry, lighting, fog, texture levels, HUD shapes, particle and weather rules. |
| `drive`, `physics`, `race` | The simulation's steps, the car model's formulas, replay block sizes and packing (`drive/replay_frames.rs`, `drive/rle.rs`), course records, pace notes. |
| `files` | Path rules, wildcards, the packed-file format (`packed.rs`), settings lines. |
| `gfx` | PCX and PNG decoding (`pcx.rs`, `png.rs`), pixel conversion, glyph cutting, screen-mode choice. |
| `input` | Key tables, device and force-feedback rules. |
| `install` | The installer's pages, script and record rules. |
| `math` | Vectors, the random generator, portable sine / cosine and the reciprocal estimates. |
| `menu`, `screens`, `service` | The event flows, championship points, list menus, the launch switches (`menu/launch.rs`), screen layouts, repair and set-up rules. |
| `net` | Message layouts, the inbox, the lobby's lists. |
| `options`, `saves` | The options model and its two file forms (`options/toml_file.rs`, `options/old_file.rs`); best times, scores and the saved game's layout. |
| `strings` | Text handling, the language file and lexicon. |
| `system`, `threading`, `memory`, `fatal` | Command line, time packing, machine facts, and small rules of those areas. |

## rc2k_derive

`rc2k_derive/` is a small procedural-macro crate with three derives used on the game's
records in `src/`:

| Derive | What it gives a struct |
| --- | --- |
| `Cells` | Copying, merging and visiting its cells field by field. |
| `ByteView` | For a struct with a fixed layout: finding its cells by byte offset, and the field paths the debug window shows. |
| `ZeroCells` | A way to make it with every cell zero. |

The traits they implement are in `src/memory/cells.rs`.

## Running its tests

```sh
cargo test -p rc2k_core
```

## Related pages

[State and memory](State-and-memory), [Code conventions](Code-conventions),
[Testing](Testing), [Building from source](Building-from-source).
