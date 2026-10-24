//! RAL.EXE's read-only data (`.rdata`), out of its image (phase 5, `notes/conversion.md`):
//! IDA 0x4DA000 .. 0x4DF1D8, the game's log texts (the settings copy's labels, the race's
//! steps, the cheats, the front end's objects, ...) and float constants. One block with
//! the image's layout (`crate::store`), first values the image's (`rdata_data.rs`,
//! generated); the code names the texts by IDA's address and gets them through
//! `framework::game_addr`. The pointers to them in the image's data are made homes at start.

crate::block!(pub mod texts {
    start: 0x4DA000,
    end: 0x4DF1D8,
    runs: crate::rest::rdata_data::RUNS,
});
