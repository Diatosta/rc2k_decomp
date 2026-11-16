# Module: strings

*Category: [Module reference](Modules)*

The game's texts. Every piece of text the game shows has a number; the language file gives the
words for each number in the chosen language. This module loads the language, looks texts up,
and builds the texts that are put together from pieces: times, speeds in the chosen units,
numbers, percentages. It also holds the keyboard tables that belong to a language.

## State

The owner is `Strings` (`src/strings/owner.rs`): the language's texts, the lexicon (the
compiled table of texts), the computer drivers' names and the shared text buffer
(`src/strings/text_buffer.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `load_language` | `src/strings/load_language.rs` | Reads the chosen language's files. |
| `own_texts` | `src/strings/own_texts.rs` | Reads the language's file of the game's own texts (`.rc2k.toml`). |
| `compile_lexicon` | `src/strings/compile_lexicon.rs` | Builds the table of texts from the language file. |
| `table`, `lexicon_bytes` | `src/strings/string_table.rs` | Where a text's bytes are found by its number. |
| `append_string_by_id` | `src/strings/append_string_by_id.rs` | Appends a text, by number, to a text being built. |
| `text_from_ids` | `src/strings/text_from_ids.rs` | Joins several texts into one. |
| `append_time` | `src/strings/append_time.rs` | A time as `m:ss.hh`. |
| `append_speed`, `units.rs` | `src/strings/append_speed.rs`, `src/strings/units.rs` | A speed in miles or kilometres an hour, by the player's choice. |
| `format_u32`, `hex_append`, `append_percent` | files of the same names | Numbers as text. |
| `text_length.rs`, `same_text.rs`, `copy_printable.rs` | `src/strings/` | Small helpers on texts: length, comparison, copying only printable characters. |

## How it is laid out

A small flat folder, one function per file. Texts are bytes in the game's own character set,
ending in a zero byte, not Rust strings. The rules without state are in
`rc2k_core/src/strings/`.

## Calls and callers

It reads through [`files`](Module-files) and `settings`. It is used wherever text is shown:
[`screens`](Module-screens), [`menu`](Module-menu), [`install`](Module-install),
[`net`](Module-net), [`race`](Module-race), [`text`](Module-text).

## Related pages

[Language files](Language-files), [The front end](The-front-end), [Glossary](Glossary).
