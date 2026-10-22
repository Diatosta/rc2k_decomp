//! `settings::find_key` (0x41157D): finds a key in a settings text.
//!
//! In:  eax = the key (lower case, NUL-terminated), esi = the text (NUL-terminated)
//! Out: eax = where the key's value starts, carry clear; or eax = 0, carry set;
//!      everything else preserved
//! A line is looked at from its first character above a space. The key must be there in
//! any case (the text is lowered through `byte_4F0BC8`) and be followed by `=` or by a
//! character that is a space or less. The value starts at the first character above a
//! space after that, an `=` between them being skipped once; a line that ends first has
//! its value at the line feed (so: empty), and a text that ends first has no such key.
//! A text that ends with the key itself makes the original compare on past both ends:
//! refused. Seven callers.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::rva;

// ---- game data ----

/// byte_4F0BC8: a character's lower case, 256 entries (A .. Z and the accented capitals).
pub const LOWER_RVA: usize = rva(0x4F0BC8);

// ---- Rust version ----
