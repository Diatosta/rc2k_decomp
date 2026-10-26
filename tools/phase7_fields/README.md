# Phase 7 field tools

Scripts used to type pieces of the race block (2026-10-25). Run from the repository root.

- `gen_store.py SPEC OUT START END STRUCT STATIC MAP DOC`: a `#[repr(C)]` overlay store over
  the race block from a field list (`ida	name	kind	count	reading`; kinds `D`, `F`, and the
  record kinds defined in the script), plus an address map (JSON) for the converters.
  `gen_tables.py` is the first version (the physics tables, `ptab_spec.txt`).
- `to_fields.py ROOT MAP STATIC USE`: `g` / `p` / `a` / `hold_*` / `negate` / `ease_zero`
  calls on the piece's IDA literals -> field accesses. `car_fields.py`: its first version.
- `vars_lists.py ROOT DIR`: `const VARIABLES: [usize; N]` mapped once through
  `game_addr(rva(x))` -> typed addresses (reads `car_map.json`, `tables_map.json`,
  `p8_map.json` in DIR).
- `const_fields.py ROOT DIR`: named constants of typed places -> field accesses.

The maps are generated: `gen_store.py` writes them; the car's from `src/physics/car_store.rs`.
The converters read the list of maps from `DIR/maps.list` (`maps.py`: one line per piece,
`map file<TAB>static<TAB>path`, in the order they are tried); a new piece is one line there.
After each: `cargo fmt`, `cargo fix` for imports, clippy both ways (touch a file first).
- `closures.py ROOT DIR`: local address / value closures and helper fns on typed literals.
- `inline.py ROOT DIR`: inline `game_read` / `game_write` / `game_addr` on `rva(typed literal)`.
- `loop_arrays.py ROOT DIR`: `g` / `p(BASE + 4 * i)` inside `for i in 0..N`, when N fits.
- `methods.py ROOT DIR`: `Ground` / `Mirror` receivers' `get` / `put` / `a`.
- `survey.py [FORM N]`: what is left of the three pieces, by form and by file.

`src/race/work_store.rs` has a hand-added `impl RaceWork { car_block, car_record }`: keep it when
regenerating.
- `rva_consts.py ROOT DIR` / `rva_pub_consts.py ROOT DIR`: private / public
  `const NAME: usize = rva(0x..)` of typed scalars -> field accesses (`game_read`,
  `game_write`, `game_addr`, `game_ptr`); the public one follows the constant into the
  files that name it through its module.
- `indexed.py ROOT`: `g` / `k` / `game_read(rva(BASE + 4 * i))` -> element reads, per file,
  for bases whose index was checked bounded there by hand (the table in the script), and
  the car blocks through `RACE_WORK.car_block` (unchecked, as the original).
- `car_map.py OUT`: the work car's map from `src/physics/car_store.rs` (run after any
  field change; `blocks_map.json` is it replicated per car block, ten of 0xFE0 from
  0x70F3EC, plus `unknown_tail`).
- `block_offsets.py ROOT DIR`: `g` / `p(B [+ 0xFE0 n | NEXT n] + off)` with `B` the cars'
  blocks -> `CAR_BLOCKS.cars[n].field`; prints each index for review.
- `car_pointer.py FILE DIR READ WRITE VAR..`: `READ(VAR + off)` / `WRITE(VAR + off, v)` with
  VAR a car block's address (checked by hand) -> `Car::at(VAR).field`.
- `drop_unsafe.py`: reads clippy's short output on stdin and takes off each `unsafe` block it
  calls unnecessary (one expression: unwrapped; a block of statements: left as `{ .. }` for a
  hand tidy).
- `local_set.py FILE HELPER MAP STATIC USE [read]`: a file's own one-dword writer
  (`set(NAME, v)`, `p(NAME[i], v)`) or, with `read`, reader (`g(NAME)`) on named
  `rva` constants of typed scalars -> `STATIC.field.set_bits(v)` / `.bits()`. Check the
  helper's body first.
- `drop_dead.py`: reads clippy's short output and removes each item rustc calls never used,
  with its comments (brackets counted, a trailing comment allowed).
- `drop_unresolved.py`: reads clippy's short output and takes each unresolved import (a
  constant a converter removed) out of its `use` list.
- `rva_pub_consts.py` must be run with ROOT `src`.
- `fix_runs.py`: reads clippy's short output and turns the whole-run reads / writes the
  converters typed as one cell (`X[0].bits()` where `[u32; N]` is wanted, and back) into
  `session::read_bits` / `write_bits`.
