//! `race::single_rally_race`: a single rally's (and the other menus') race: the track is looked
//! up, the settings and the players' records are copied into the race's variables, the race runs
//! (`race::race_from_menu`, with the sound objects and screen modes set round it) and the results
//! are copied back. See `race/single_rally.rs`.
//!
//! Raises fatal 0x80739281 "Unknown Shaun TrakNum" (a track not in the stage tables' `trak_pak`);
//! refuses first a player's record that is missing.

pub use crate::race::single_rally::single_rally_race;
