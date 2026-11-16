# Module: gfx

*Category: [Module reference](Modules)*

The game's 2D graphics. It owns the picture in memory that the menus are drawn into, reads the
picture files (PCX and PNG), builds the fonts' glyphs, copies and scales pictures, converts
between pixel formats, chooses screen modes and writes screenshots. Menus and text never go
through the 3D code: they are drawn here, pixel by pixel, and the platform puts the result on
the window.

## State

The owner is `Gfx` (`src/gfx/owner.rs`): the screen's state, the picture buffer, the fonts and
glyph tables, the pixel converters. It also holds the 3D layer's state (`Gfx::d3d`, see
[`d3d`](Module-d3d)). Parts are defined in the `*_store.rs` files (`fonts_store.rs`,
`glyph_store.rs`, `picture_store.rs`) and `screen_state.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `load_picture_with` | `src/gfx/load_picture.rs` | Loads a picture file into the picture buffer. |
| `read_pcx`, `read_png` | `src/gfx/read_pcx.rs`, `src/gfx/read_png.rs` | The two picture readers. |
| `show_picture_file` | `src/gfx/show_picture_file.rs` | Loads a picture and shows it (loading screens). |
| `read_picture_for_texture`, `upload_picture` | `src/gfx/load_picture_to_texture.rs` | A picture file made into a texture. |
| `build_fonts_in`, `build_game_fonts` | `src/gfx/build_fonts.rs`, `src/gfx/build_game_fonts.rs` | Cuts the font sheets into glyphs. |
| `blit_glyph` | `src/gfx/blit_glyph.rs` | Draws one glyph or sprite; every piece of text ends here. |
| `Blitter` | `src/gfx/blit.rs` | The copying of pixel rows with clipping. |
| `Target` | `src/gfx/surface.rs` | What is being drawn on: the screen, a texture, or the picture buffer. |
| `set_clip` | `src/gfx/set_clip.rs` | Sets the rectangle drawing is kept inside. |
| `try_screen_mode` | `src/gfx/try_screen_mode.rs` | Tries a screen mode and falls back if it cannot be set. |
| `take_screenshot`, `write_screen_pcx` | files of the same names | Writes the screen as a PCX file. |
| `load_jade` | `src/gfx/load_jade.rs` | Loads the small font the fatal-error screen uses. |

## How it is laid out

One function per file. Groups by name: `read_*`, `lay_out_*`, `decode_*` and `png*.rs` /
`pcx*.rs` (picture files), `build_*` and `glyph_*` (fonts), `*_to_dwords.rs`, `converters.rs`
and `components_*` (pixel formats), `*_modes.rs` (screen modes), `blit_*` (copying).
`draw2d/` records a frame's 2D drawing as typed commands, which the GPU renderers use to draw
menus as textured quads and which checks replay.

## Calls and callers

It reads through [`files`](Module-files) and reaches the window through
[`platform`](Module-platform). It is used by [`menu`](Module-menu), [`screens`](Module-screens),
[`text`](Module-text), [`race`](Module-race), [`net`](Module-net) and [`install`](Module-install).

## Related pages

[Drawing a frame](Drawing-a-frame), [Textures (PCX)](Textures-PCX), [Screens (PNG)](Screens-PNG),
[Fonts](Fonts), [Screenshots](Screenshots).
