# Fixed bugs

Bugs of the original `RAL.EXE` that the Rust version fixes. Up to the parity milestone the
Rust code kept every one of the original's slips, so that it could be compared with the
original function by function; since then each bug is fixed as its code is converted.
Addresses are those of the original executable (as IDA shows them).

Each entry says:
- **What**: the mistake in the original's code.
- **In game**: what a player could see or hear (or that nothing shows).
- **Seen**: whether it was seen in play or found by reading the code.
- **Fix**: what the Rust version does instead, and where.

---

## Text lookup with a bad string id reads anywhere

*Original*: `strings::expand_entry` (0x401FE1). *Fixed*: 2026-10-20.

- **What**: the game's texts (menus, messages) live in one table of entries that may name
  other entries. The lookup checks neither that an id is inside the table's index, nor
  that an entry's data stays inside the table, nor how deep entries nest. A bad id reads
  whatever memory follows, and an entry naming itself recurses until the stack runs out.
- **In game**: nothing with the game's own text files, whose ids are all valid; a damaged
  or edited text file could show garbage text or crash the game.
- **Seen**: no; found by reading the code.
- **Fix**: `rc2k_core::strings::StringTable::expand` checks the id, the entry's bounds and
  the nesting depth; a bad entry is refused (nothing written) and logged.

## Logged floating-point numbers carry leftover digits

*Original*: `logfile::log_float` (0x4014D8). *Fixed*: 2026-10-22.

- **What**: to log a number with a given count of decimals, the original writes its text
  into a small scratch buffer and puts the end marker where that count of decimals would
  end. When the text is shorter (more decimals asked for than it has, or "infinity" /
  "nan"), the marker lands past it, so the logged line also takes whatever earlier numbers
  left in the buffer.
- **In game**: nothing on screen; some lines of the game's log (`var\logfile.txt`) show
  stray characters after a number.
- **Seen**: no; found by reading the code.
- **Fix**: the Rust logger (`logfile::numeric::log_number`) logs the text only up to what
  was written: the marker's place, but never past the text's end.
