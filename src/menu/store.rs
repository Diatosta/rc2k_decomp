//! The front end's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`):
//! IDA 0x755144 .. 0x755224 (the savegame preview's and details' figures, the controller
//! screen's history and binding learner) and 0x75523C .. 0x755A4C (the menus' state: the
//! yes / no answer, the event's mode, kind, players and sections, the screen lists, the
//! sprite table, the options' work copies, the best-times tables' pointers, the player
//! names, the work variables of the drawing code, the Escape flags), around the progress
//! bar's variables (`gfx::bar_store`, 0x755224 .. 0x75523C); and 0x755A4C .. 0x758B72 (the
//! players' records of 0x28C bytes and `g_players`, the stage and event figures, the
//! arcade's totals, the cheats, the captions' lists of string ids), up to the draw lists
//! (0x758B72 .. 0x7594B0, still the image's: their addresses are compared as IDA's).
//!
//! Each region is one block with the image's layout (`crate::store`): gaps and unreferenced
//! bytes kept, first values the image's (`store_data.rs`, generated). The code still names
//! them by IDA's address; `framework::game_addr` takes such an address to the block.

use crate::block;

block!(pub mod controls {
    start: 0x755144,
    end: 0x755224,
    runs: crate::menu::store_data::CONTROLS_RUNS,
});

block!(pub mod state {
    start: 0x75523C,
    end: 0x755A4C,
    runs: crate::menu::store_data::STATE_RUNS,
});

block!(pub mod players {
    start: 0x755A4C,
    end: 0x758B72,
    runs: crate::menu::store_data::PLAYERS_RUNS,
});
