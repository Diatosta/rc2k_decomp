//! `race::single_rally_race` (0x4843EC): a single rally's (and the other menus') race: the
//! track looked up, the settings and the players' records copied into the race's
//! variables, the race run (`race::race_from_menu`, the sound objects and the screen modes
//! round it), the results copied back. See `race/single_rally.rs`.
//!
//! In:  eax = 1 when the loading picture is not to be shown after, ebx = the kind (3:
//!      the rally's times copied first)
//! Out: every register preserved (`pusha` / `popa`)
//!
//! About 50 callers (the menus' races; each `cmp`s after, reading no flags). Raises fatal
//! 0x80739281 "Unknown Shaun TrakNum" (a track not in `g_trakPakHeader`) from Rust; refuses a
//! player's record missing (first). The registers above are the original's, for reference; a
//! call by address goes through `routes.rs`.

pub use crate::race::single_rally::single_rally_race;
