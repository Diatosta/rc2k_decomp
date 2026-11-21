# Module: text

*Category: [Module reference](Modules)*

Drawing and measuring text. A text is drawn glyph by glyph in one of the game's fonts, with
the font's spacing and kerning, at a position on the current drawing surface. The same code
measures a text without drawing it, which the screens use to centre and right-align.

## State

None of its own. The fonts and their glyphs are in the graphics owner (`Gfx`); the layout of a
font's record is described in `src/text/font.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `draw_text` | `src/text/draw_text.rs` | Draws a text at a position in a font. |
| `draw_string_by_id` | `src/text/draw_string_by_id.rs` | Draws the text with a given number in the current language. |
| `measure_text`, `measure_string_by_id` | files of the same names | A text's width in a font. |
| `draw_time_text` | `src/text/draw_time_text.rs` | Draws a time. |
| `draw_time_fixed_pitch`, `draw_clock_fixed_pitch` | files of the same names | Times with every digit the same width, so they do not jitter while they count. |
| `blend_font` | `src/text/blend_font.rs` | Makes a font's colours blend towards another colour (highlights, fades). |
| `glyph_kerning.rs` | `src/text/glyph_kerning.rs` | The spacing between two glyphs. |

## How it is laid out

A dozen files, one function each.

## Calls and callers

It draws glyphs with `gfx::blit_glyph` and looks texts up in [`strings`](Module-strings). It
is called mostly by [`screens`](Module-screens), and by [`menu`](Module-menu),
[`net`](Module-net) and [`race`](Module-race).

## Related pages

[Fonts](Fonts), [The front end](The-front-end), [Language files](Language-files).
