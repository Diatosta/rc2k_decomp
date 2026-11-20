# Module: memory

*Category: [Module reference](Modules)*

The building blocks the game's data is made of. The game has several threads and a great many
variables that more than one of them reads. Instead of globals, every variable is a **cell**:
a small typed container that can be read and written through a shared reference, with the
rules for who may touch it built into its type. This module defines those cells, the traits
that let whole records of cells be copied or compared, and a few containers for larger data.

## State

It defines types rather than holding state. The one piece of state is the bookkeeping of the
game's memory lists (`src/memory/mod.rs`), which keeps the figures the game logs about its
memory use.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `Dword`, `Float`, `Int`, `Word`, `Flag` | `src/memory/session.rs` | Cells any thread may read and write (atomics underneath). |
| `GameThread<T>` and the plain cells | `src/memory/session.rs` | Cells and wrappers for data only the game thread reaches. |
| `Cells`, `ByteView` | `src/memory/cells.rs` | Traits (derived) that let a record of cells be copied, merged and visited field by field, or found by byte offset. |
| `Owned<T>` | `src/memory/owned.rs` | A typed table that is taken when needed and given back, like a block of heap. |
| `PoolBytes` | `src/platform/pool.rs` | A block of bytes made at run time (the screen's and the textures' pixels). |
| `PartCell<T>` | `src/memory/part_cell.rs` | State owned by one drawing thread at a time. |
| `SharedTable` | `src/memory/shared_table.rs` | A table several threads read. |
| `BytePlace` | `src/memory/place.rs` | A position inside a text or data buffer, passed in place of a pointer. |
| `ListMenu`, `ListItem` | `src/memory/list_handle.rs` | The replay viewer's list menu, by handle. |
| `shared_memory.rs` | `src/memory/shared_memory.rs` | The few raw reads and writes, each with its safety rule. |

## How it is laid out

A handful of files, each a kind of container. The derive macros for `Cells` and `ByteView`
are in the crate `rc2k_derive`.

## Calls and callers

Every module uses it; it uses almost nothing.

## Related pages

[State and memory](State-and-memory), [Threads and shared state](Threads-and-shared-state),
[Code conventions](Code-conventions), [Glossary](Glossary).
