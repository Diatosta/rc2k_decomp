# M7 step 5: the tests off old addresses (survey, API, tool, pilot)

Counts from `python3 tools/m7_tests/survey.py` (test text only: a file minus
`count_unsafe.strip_tests`, test-only files whole, comments out). "idioms" leaves out the
literal IDA addresses, which are counted apart ("ida literal": a hex literal 0x40xxxx ..
0x74xxxx).

## The idioms and what they are used for

By the test families they sit in (the test function's name):

| family | idioms | literals | files | what it does | what replaces it |
| --- | --- | --- | --- | --- | --- |
| first values (`first_values_are_the_games`, `*_starts_as_the_games`, ~100 tests + 99 calls of `gfx::first_values_are_the_games`) | 357 | 367 | 151 | a store's or constant's bytes against `moved::image_bytes` | `test_support::first_values` (golden), and one test `every_store_starts_as_the_games` over the registry |
| layer self-tests (`moved_knows_the_block` 35, `piece_has_its_home` 50, `home()` 59) | 286 | 410 | 135 | `moved::place(IDA) == field.addr()`; `offset_of!(T, f) == 0xIDA - START` | the place checks go with `MOVED` (delete); the offsets stay as plain offsets (R1) |
| asm / x87 reference (`asm_reference`, `*_matches_the_assembly`) | 54 | 225 | 18 | copied asm run against `asm_base()` / `OldPlaces` / `game_addr(rva(..))` | vectors recorded once on i686, replayed (plan step 5) |
| other (state set before a call, snapshot / restore, raw reads through `host_at`, `pointers::host`) | 208 | 322 | 102 | names a field to write or read by address | a typed field access, or `test_support::state` (`Snapshot::of`, `store(name)` + `Store::load` / `store`) |

Per kind (all modules): `image_bytes` 243, `moved::place` 152, `host_at` 137, `rva(` 78,
`game_addr` 64, `fn home()` 58, `pointers::host` 49, `asm_base` / `OldPlaces` 34,
`GamePtr::new` 26, `IMAGE_BASE` 12, `moved::named` 9, `game_read` / `game_write` 8,
`home_of_rva` 8, `moved::snapshot` / `restore` 6; literals 1,327.

## The API (`src/test_support/`, test builds only)

- `data/first_values.golden`: one line per key, `<key> <size> <fnv1a64>[ <hex bytes>]`.
  Every registry store (`store:<registry name>`, `#2` .. when a name repeats) as size and
  hash of the image's bytes; named reads (the constants and fields tests read) with their
  bytes. Made from today's image by `tools/m7_tests/freeze.sh`; keys of named reads are
  listed in `tools/m7_tests/first_values.manifest` (`<key> <IDA> <len>`).
  Key rule: a store's address -> `<registry name>[<field path>]/<len>` (no field path from
  a store's first byte); an image constant -> the Rust item it is compared with
  (`logfile.write_line.LOG_PATH`).
- `first_values::{bytes, dword, assert_matches, golden, hash}`: read / check by key.
- `first_values::assert_block_starts_as_the_games(&block)`: the store starting at the
  block's first byte against the golden hash (the `gfx::first_values_are_the_games`
  replacement; takes `test_lock`).
- `first_values::{store_keys, store_bytes, assert_store_starts_as_the_games}`.
- Test `every_store_starts_as_the_games`: every registry store but 18 (`NOT_THE_GAMES`:
  handles where the image has addresses, checked by their own patched tests) and the three
  run-time ones (`CHANGED_BY_TESTS`: allocator counts, file search, the picture) starts as
  the image. It covers what the per-store first-value tests check: the codemod lanes may
  delete those once they are on the golden.
- `state::{Snapshot::of(&[names]), Snapshot::restore, store(name)}`: snapshot / restore
  through the registry, by store name. Only the stores a test writes: restoring all of them
  undid unlocked tests' writes (seen: picture and memory tests failed).
- `freeze.rs` (one-off, goes with the layer): `RC2K_FREEZE_FIRST_VALUES` writes the golden;
  `RC2K_NAME_ADDRESSES` names IDA addresses by store key, offset and field path.

## The tool (`tools/m7_tests/`)

1. `python3 tools/m7_tests/codemod.py collect <modules> > addrs` (literals in test text and
   the files' address constants).
2. `CARGO_TARGET_DIR=... tools/m7_tests/freeze.sh names addrs names` (Rust: `MOVED` gives
   IDA -> home, `stores::find` home -> store and field path). 1,105 addresses: 1,061 in a
   store, 44 `image` (code, rdata constants).
3. `codemod.py report names <module>`: every test line with an idiom, its literals named.
4. `codemod.py rewrite names <module>`: the exact rewrites (R1 offsets, R2 block checks,
   R3 literal `image_bytes` reads -> golden keys, appending to the manifest), then the report
   of what is left. Then `freeze.sh` (regenerates the golden with the new keys), `cargo fmt`.
   Merges: the golden and manifest are sorted / grouped by key, lanes add disjoint lines.

Dry run over every module (a copy of the tree, not committed): 166 files rewritten, 86 R2
calls, 25 manifest keys; idioms 877 -> 852, literals 1,317 -> 820; the Linux lib suite gave
the same result as the untouched tree (716 passed, the same 5 failures, below).

## Pilot (math, logfile)

math 3 idioms / 5 literals -> 0 / 0; logfile 5 / 5 -> 0 / 0 (whole tree 885 / 1,327 ->
877 / 1,317). Linux `--lib math:: logfile:: test_support::` 151 passed; i686 under Wine 184
passed; the whole Linux lib suite 716 passed, 5 failed both before and after (they also fail
alone: `audio::load_sounds::lists_end_inside_their_store`, `gfx::picture` pcx / png (no
game files in the worktree), `gfx::picture_store::first_values_are_the_games`,
`memory::variables_start_as_the_games`).

## Work list for the codemod lanes (after `rewrite`, what is left by hand)

| module | files | idioms | literals | idioms after rewrite | literals after rewrite |
| --- | --- | --- | --- | --- | --- |
| total | 283 | 877 | 1317 | 852 | 820 |
| race | 59 | 232 | 424 | 230 | 230 |
| effects | 60 | 170 | 171 | 170 | 164 |
| gfx | 37 | 92 | 79 | 92 | 42 |
| sound | 7 | 65 | 44 | 58 | 37 |
| net | 1 | 2 | 107 | 2 | 10 |
| physics | 17 | 29 | 75 | 29 | 14 |
| audio | 3 | 7 | 86 | 6 | 85 |
| strings | 11 | 22 | 68 | 16 | 62 |
| scene | 16 | 31 | 48 | 31 | 0 |
| saves | 5 | 16 | 56 | 15 | 49 |
| menu | 25 | 52 | 12 | 52 | 5 |
| install | 5 | 30 | 28 | 30 | 27 |
| LAYER | 5 | 45 | 3 | 45 | 3 |
| input | 3 | 10 | 35 | 10 | 35 |
| session | 1 | 4 | 26 | 4 | 26 |
| drive | 3 | 7 | 14 | 7 | 0 |
| records | 1 | 5 | 14 | 5 | 14 |
| settings | 2 | 11 | 7 | 8 | 4 |
| fatal | 1 | 6 | 7 | 5 | 6 |
| system | 4 | 10 | 0 | 10 | 0 |
| hud | 2 | 7 | 0 | 7 | 0 |
| service | 1 | 3 | 3 | 3 | 1 |
| text | 1 | 3 | 3 | 0 | 0 |
| screens | 3 | 5 | 0 | 5 | 0 |
| files | 2 | 4 | 1 | 4 | 1 |
| d3d | 2 | 4 | 0 | 4 | 0 |
| debug | 2 | 2 | 1 | 2 | 1 |
| platform_3d | 1 | 0 | 3 | 0 | 3 |
| memory | 1 | 1 | 1 | 0 | 0 |
| threading | 1 | 1 | 1 | 1 | 1 |
| place | 1 | 1 | 0 | 1 | 0 |

Suggested lanes (by size after the rewrite): race (230 / 230, with `asm_base` / `OldPlaces`
34: the asm reference vectors), effects (170 / 164: `fn home()` 21, `host_at` 55,
`image_bytes` 77 with `T::START`), gfx + menu + screens + hud + d3d (`moved::place` 64,
`pointers::host` 20), sound + audio + input (`host_at` 32), strings + saves + settings +
install + session + records + fatal + the rest. LAYER (45) goes with step 6.
